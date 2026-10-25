/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! Lexical bindings and their definition occurrences, independent of inferred types.
//! IDs are local to one module analysis and must not be persisted across rebuilds.

use ruff_python_ast::Identifier;
use ruff_python_ast::name::Name;
use ruff_text_size::TextRange;
use starlark_map::small_map::SmallMap;

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

#[derive(Clone, Debug, Default)]
pub struct SymbolInventory {
    scopes: Vec<ScopeInfo>,
    symbols: Vec<BindingSymbol>,
    by_owner: SmallMap<(ScopeId, Name), BindingSymbolId>,
    /// Multiple names can share a source range, for example in a wildcard import.
    by_definition: SmallMap<(TextRange, Name), usize>,
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
        let symbol = owner.map(|owner| {
            *self
                .by_owner
                .entry((owner, name.clone()))
                .or_insert_with(|| {
                    let id = BindingSymbolId(self.symbols.len());
                    self.symbols.push(BindingSymbol {
                        name: name.clone(),
                        owner,
                    });
                    id
                })
        });
        let site = DefinitionSite {
            symbol,
            range,
            kind,
        };
        if let Some(index) = self.by_definition.get(&(range, name.clone())) {
            self.definitions[*index] = site;
        } else {
            self.by_definition
                .insert((range, name.clone()), self.definitions.len());
            self.definitions.push(site);
        }
    }

    pub(super) fn finish(&mut self, metadata: &BindingsMetadata) {
        // ClassFields selects the declaration site, including receiver assignments. Do not
        // rediscover fields or confuse this binding-time information with inherited attributes.
        for i in 0..self.scopes.len() {
            if let SymbolScopeKind::Class(class) = self.scopes[i].kind {
                let fields = &metadata.get_class(class).fields;
                for name in fields.fields() {
                    let range = fields
                        .field_decl_range(name)
                        .expect("a field has a declaration");
                    if !self.by_definition.contains_key(&(range, name.clone())) {
                        self.record(name, Ok(ScopeId(i)), range, DefinitionKind::Field);
                    }
                }
            }
        }
        self.definitions
            .sort_by_key(|site| (site.range.start(), site.range.end()));
        // Walrus binding first updates the comprehension flow, then records the enclosing
        // owner. Retain only lexical symbols referenced by the final definition sites.
        let mut remap = SmallMap::new();
        let mut symbols = Vec::new();
        for site in &mut self.definitions {
            if let Ok(id) = &mut site.symbol {
                *id = *remap.entry(*id).or_insert_with(|| {
                    let new_id = BindingSymbolId(symbols.len());
                    symbols.push(self.symbols[id.0].clone());
                    new_id
                });
            }
        }
        self.symbols = symbols;
        // These lookups are only needed while building the inventory.
        self.by_owner = SmallMap::new();
        self.by_definition = SmallMap::new();
    }
}
