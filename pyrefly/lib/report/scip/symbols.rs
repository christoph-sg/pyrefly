/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use pyrefly_python::module_name::ModuleName;
use ruff_text_size::TextRange;
use scip::types::Descriptor;
use scip::types::descriptor::Suffix;

use crate::binding::symbols::DefinitionKind;
use crate::binding::symbols::ScopeId;
use crate::binding::symbols::SymbolInventory;
use crate::binding::symbols::SymbolScopeKind;

#[derive(Debug, Clone, PartialEq)]
pub enum ScipSymbol {
    Global(Vec<Descriptor>),
    Local(usize),
}

/// Assign SCIP descriptors to the binder's definition sites in source order. Lexical IDs
/// only join occurrences within this analysis; persistent names contain source names only.
pub fn definitions(
    module: ModuleName,
    inventory: &SymbolInventory,
) -> Vec<(TextRange, ScipSymbol)> {
    let module_path = vec![descriptor(module.as_str(), Suffix::Namespace)];
    let mut names: HashMap<_, ScipSymbol> = HashMap::new();
    let mut next_local = 0;
    let mut definitions = Vec::new();
    for site in inventory.definitions() {
        // Imports are emitted by reference collection. Preserve the existing policy that
        // augmented assignments are references and type parameters have no SCIP definition.
        if matches!(
            site.kind,
            DefinitionKind::Import
                | DefinitionKind::AugmentedAssignment
                | DefinitionKind::TypeParameter
        ) {
            continue;
        }
        let Ok(id) = site.symbol else { continue };
        let binding = inventory.symbol(id);
        let scope = inventory.scope(binding.owner);
        let suffix = match site.kind {
            DefinitionKind::Function => Some(Suffix::Method),
            DefinitionKind::Class => Some(Suffix::Type),
            DefinitionKind::Parameter if scope.kind == SymbolScopeKind::Function => {
                Some(Suffix::Parameter)
            }
            _ => None,
        };
        let symbol = if let Some(suffix) = suffix {
            // A declaration can reclassify an existing name. Later assignments retain that
            // descriptor, while earlier occurrences keep the descriptor they already received.
            let Some(path) = scope_path(inventory, binding.owner, &module_path) else {
                continue;
            };
            ScipSymbol::Global(child(&path, &binding.name, suffix))
        } else if let Some(symbol) = names.get(&id) {
            symbol.clone()
        } else if matches!(
            scope.kind,
            SymbolScopeKind::Module | SymbolScopeKind::Class(_)
        ) {
            let Some(path) = scope_path(inventory, binding.owner, &module_path) else {
                continue;
            };
            ScipSymbol::Global(child(&path, &binding.name, Suffix::Term))
        } else {
            let symbol = ScipSymbol::Local(next_local);
            next_local += 1;
            symbol
        };
        names.insert(id, symbol.clone());
        definitions.push((site.range, symbol));
    }
    definitions
}

pub fn descriptor(name: &str, suffix: Suffix) -> Descriptor {
    Descriptor {
        name: name.to_owned(),
        suffix: suffix.into(),
        ..Default::default()
    }
}

fn child(path: &[Descriptor], name: &str, suffix: Suffix) -> Vec<Descriptor> {
    let mut path = path.to_vec();
    path.push(descriptor(name, suffix));
    path
}

/// Declaration nesting supplies descriptors, not name resolution. In particular, the
/// declaration's lexical owner may differ from its syntactic containing scope.
fn scope_path(
    inventory: &SymbolInventory,
    id: ScopeId,
    module: &[Descriptor],
) -> Option<Vec<Descriptor>> {
    let scope = inventory.scope(id);
    if let Some(declaration) = &scope.declaration {
        let site = inventory.definition_at(declaration.range)?;
        // Invalid captures do not acquire a valid descriptor through their containing scope.
        let owner = inventory.symbol(*site.symbol.as_ref().ok()?).owner;
        return Some(child(
            &scope_path(inventory, owner, module)?,
            &declaration.id,
            match site.kind {
                DefinitionKind::Class => Suffix::Type,
                DefinitionKind::Function => Suffix::Method,
                _ => unreachable!("a scope is introduced by a class or function declaration"),
            },
        ));
    }
    match scope.parent {
        Some(parent) => scope_path(inventory, parent, module),
        None => Some(module.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use scip::symbol::format_symbol;
    use scip::types::Symbol;

    use super::*;
    use crate::state::require::Require;
    use crate::test::util::mk_multi_file_state;

    fn symbols(code: &str) -> Vec<(String, String)> {
        let (handles, state) = mk_multi_file_state(&[("m", code)], Require::Everything, false);
        let transaction = state.transaction();
        let answers = transaction.get_answers(&handles["m"]).unwrap();
        definitions(
            ModuleName::from_str("m"),
            answers.bindings().symbol_inventory().unwrap(),
        )
        .iter()
        .map(|(range, symbol)| {
            let symbol = match symbol {
                ScipSymbol::Global(descriptors) => format_symbol(Symbol {
                    scheme: "s".to_owned(),
                    descriptors: descriptors.clone(),
                    ..Default::default()
                }),
                ScipSymbol::Local(i) => format!("local {i}"),
            };
            (code[*range].to_owned(), symbol)
        })
        .collect()
    }

    fn assert_symbols(code: &str, expected: &[(&str, &str)]) {
        let expected: Vec<_> = expected
            .iter()
            .map(|(name, symbol)| (name.to_string(), format!("s . . . {symbol}")))
            .collect();
        assert_eq!(symbols(code), expected);
    }

    #[test]
    fn test_classes_and_functions() {
        assert_symbols(
            r#"
class C:
    x: int
    class D: ...
    def f(self, a, *args, **kwargs):
        def g(): ...
"#,
            &[
                ("C", "m/C#"),
                ("x", "m/C#x."),
                ("D", "m/C#D#"),
                ("f", "m/C#f()."),
                ("self", "m/C#f().(self)"),
                ("a", "m/C#f().(a)"),
                ("args", "m/C#f().(args)"),
                ("kwargs", "m/C#f().(kwargs)"),
                ("g", "m/C#f().g()."),
            ],
        );
    }

    #[test]
    fn test_self_attributes() {
        assert_symbols(
            r#"
class C:
    y: int
    def __init__(self, v):
        self.x = v
        self.x = v
        self.y = v
"#,
            &[
                ("C", "m/C#"),
                ("y", "m/C#y."),
                ("__init__", "m/C#__init__()."),
                ("self", "m/C#__init__().(self)"),
                ("v", "m/C#__init__().(v)"),
                ("x", "m/C#x."),
            ],
        );
    }

    #[test]
    fn test_capture_and_lambda_ownership() {
        let actual = symbols(
            r#"
x = 0
def outer():
    value = 1
    def middle():
        def writer():
            nonlocal value
            value = 2
callback = lambda: (x := 3)
"#,
        );
        let values: Vec<_> = actual.iter().filter(|(name, _)| name == "value").collect();
        // The intervening function does not bind `value`; both writes name one local.
        assert_eq!(values.len(), 2);
        assert_eq!(values[0].1, "local 0");
        assert_eq!(values[1].1, values[0].1);
        let xs: Vec<_> = actual.iter().filter(|(name, _)| name == "x").collect();
        // A lambda is a binding scope for walrus assignments, unlike a comprehension.
        assert_eq!(xs.len(), 2);
        assert_eq!(xs[0].1, "s . . . m/x.");
        assert_eq!(xs[1].1, "local 1");
    }

    #[test]
    fn test_reclassified_declarations() {
        assert_symbols(
            "x = 0\ndef x(): ...\nx = 1\nclass x: ...\nx = 2\n",
            &[
                ("x", "m/x."),
                ("x", "m/x()."),
                ("x", "m/x()."),
                ("x", "m/x#"),
                ("x", "m/x#"),
            ],
        );
    }

    #[test]
    fn test_locals() {
        let actual = symbols(
            r#"
x = 1
for y in []: pass
def f(p):
    p = 1
    z = [w for w in []]
    z = 2
    global x
    x = 3
squares = [w * w for w in []]
"#,
        );
        let expected = [
            ("x", "s . . . m/x."),
            ("y", "s . . . m/y."),
            ("f", "s . . . m/f()."),
            ("p", "s . . . m/f().(p)"),
            ("p", "s . . . m/f().(p)"),
            ("z", "local 0"),
            ("w", "local 1"),
            ("z", "local 0"),
            ("x", "s . . . m/x."),
            ("squares", "s . . . m/squares."),
            ("w", "local 2"),
        ];
        assert_eq!(
            actual,
            expected
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect::<Vec<_>>()
        );
    }
}
