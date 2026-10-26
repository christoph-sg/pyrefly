/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Lexical bindings and their definition occurrences, independent of inferred types.
//! IDs are local to one module analysis and must not be persisted across rebuilds.

use pyrefly_python::short_identifier::ShortIdentifier;
use ruff_python_ast::Identifier;
use ruff_python_ast::name::Name;
use ruff_text_size::TextRange;
use starlark_map::small_map::SmallMap;

use crate::binding::binding::Binding;
use crate::binding::binding::Key;
use crate::binding::bindings::BindingTable;
use crate::binding::metadata::BindingsMetadata;
use crate::binding::scope::MutableCaptureError;
use crate::types::class::ClassDefIndex;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScopeId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BindingSymbolId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolScopeKind {
    Module,
    Class(ClassDefIndex),
    Function,
    Lambda,
    Comprehension,
    Annotation,
    TypeAlias,
}

#[derive(Clone, Debug)]
pub struct ScopeInfo {
    pub kind: SymbolScopeKind,
    /// Syntactic containment, not the Python name-resolution search order.
    pub parent: Option<ScopeId>,
    pub range: TextRange,
    pub declaration: Option<Identifier>,
}

#[derive(Clone, Debug)]
pub struct BindingSymbol {
    pub name: Name,
    pub owner: ScopeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    Assignment,
    AugmentedAssignment,
    Parameter,
    Function,
    Class,
    Import,
    TypeParameter,
    /// A field discovered during binding, not an inherited attribute resolved by the solver.
    Field,
}

#[derive(Clone, Debug)]
pub struct DefinitionSite {
    /// Invalid captures have no lexical owner. They are not fresh local bindings.
    pub symbol: Result<BindingSymbolId, MutableCaptureError>,
    pub range: TextRange,
    pub kind: DefinitionKind,
}

#[derive(Clone, Debug)]
pub struct SymbolInventory {
    scopes: Vec<ScopeInfo>,
    symbols: Vec<BindingSymbol>,
    definitions: Vec<DefinitionSite>,
}

impl SymbolInventory {
    pub fn scopes(&self) -> impl Iterator<Item = (ScopeId, &ScopeInfo)> {
        self.scopes.iter().enumerate().map(|(i, s)| (ScopeId(i), s))
    }

    pub fn symbols(&self) -> impl Iterator<Item = (BindingSymbolId, &BindingSymbol)> {
        self.symbols
            .iter()
            .enumerate()
            .map(|(i, s)| (BindingSymbolId(i), s))
    }

    /// Definition occurrences in source order.
    pub fn definitions(&self) -> impl Iterator<Item = &DefinitionSite> {
        self.definitions.iter()
    }

    /// Return the first definition at this range. Use `definitions` to enumerate all names
    /// introduced by an occurrence such as a wildcard import.
    pub fn definition_at(&self, range: TextRange) -> Option<&DefinitionSite> {
        let index = self.definitions.partition_point(|site| {
            (site.range.start(), site.range.end()) < (range.start(), range.end())
        });
        self.definitions
            .get(index)
            .filter(|site| site.range == range)
    }

    pub fn scope(&self, id: ScopeId) -> &ScopeInfo {
        &self.scopes[id.0]
    }

    pub fn symbol(&self, id: BindingSymbolId) -> &BindingSymbol {
        &self.symbols[id.0]
    }
}

/// Record ownership while binding; assign lexical IDs only after all writes are known.
#[derive(Clone, Debug, Default)]
pub(super) struct SymbolInventoryBuilder {
    scopes: Vec<ScopeInfo>,
    /// Multiple names can share a source range, for example in a wildcard import.
    definitions:
        SmallMap<(TextRange, Name), (Result<ScopeId, MutableCaptureError>, DefinitionKind)>,
}

impl SymbolInventoryBuilder {
    pub(super) fn push_scope(&mut self, info: ScopeInfo) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(info);
        id
    }

    pub(super) fn record(
        &mut self,
        name: &Name,
        owner: Result<ScopeId, MutableCaptureError>,
        range: TextRange,
        kind: DefinitionKind,
    ) {
        if range.is_empty() || name.is_empty() {
            return;
        }
        // A comprehension walrus first binds in the comprehension, then records its
        // enclosing owner. Replacing the occurrence does not create an orphan symbol.
        self.definitions
            .insert((range, name.clone()), (owner, kind));
    }

    pub(super) fn finish(
        mut self,
        metadata: &BindingsMetadata,
        table: &BindingTable,
    ) -> SymbolInventory {
        // ClassFields selects the declaration site, including receiver assignments. Do not
        // rediscover fields or confuse this binding-time information with inherited attributes.
        for (i, scope) in self.scopes.iter().enumerate() {
            if let SymbolScopeKind::Class(class) = scope.kind {
                // Only a syntactic class declaration introduces a class scope. Functional
                // factories also produce Binding::ClassDef, but remain assignments.
                if let Some(declaration) = &scope.declaration
                    && let Some((_, kind)) = self
                        .definitions
                        .get_mut(&(declaration.range, declaration.id.clone()))
                {
                    *kind = DefinitionKind::Class;
                }
                let fields = &metadata.get_class(class).fields;
                for name in fields.fields() {
                    let range = fields
                        .field_decl_range(name)
                        .expect("a field has a declaration");
                    if !range.is_empty() && !name.is_empty() {
                        self.definitions
                            .entry((range, name.clone()))
                            .or_insert((Ok(ScopeId(i)), DefinitionKind::Field));
                    }
                }
            }
        }
        let mut occurrences: Vec<_> = self.definitions.into_iter().collect();
        occurrences.sort_by_key(|((range, _), _)| (range.start(), range.end()));
        let mut by_owner = SmallMap::new();
        let mut symbols = Vec::new();
        let definitions = occurrences
            .into_iter()
            .map(|((range, name), (owner, kind))| {
                // Some bindings, notably augmented assignments, are populated after the write.
                // Read their final kind without coupling lexical identity to a binding key.
                let kind = if kind == DefinitionKind::Assignment {
                    let key = Key::Definition(ShortIdentifier::from_text_range(range));
                    match table
                        .types
                        .0
                        .key_to_idx(&key)
                        .and_then(|idx| table.types.1.get(idx))
                    {
                        Some(Binding::Function { .. }) => DefinitionKind::Function,
                        Some(Binding::TypeParameter(..)) => DefinitionKind::TypeParameter,
                        Some(
                            Binding::FunctionParameter(..)
                            | Binding::LambdaParameter(..)
                            | Binding::TypeLevelLambdaParameter(..),
                        ) => DefinitionKind::Parameter,
                        Some(Binding::AugAssign(..)) => DefinitionKind::AugmentedAssignment,
                        _ => kind,
                    }
                } else {
                    kind
                };
                let symbol = owner.map(|owner| {
                    *by_owner.entry((owner, name.clone())).or_insert_with(|| {
                        let id = BindingSymbolId(symbols.len());
                        symbols.push(BindingSymbol { name, owner });
                        id
                    })
                });
                DefinitionSite {
                    symbol,
                    range,
                    kind,
                }
            })
            .collect();
        SymbolInventory {
            scopes: self.scopes,
            symbols,
            definitions,
        }
    }
}
