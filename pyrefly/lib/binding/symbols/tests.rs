/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use ruff_text_size::TextRange;
use ruff_text_size::TextSize;

use crate::binding::symbols::BindingSymbolId;
use crate::binding::symbols::DefinitionKind;
use crate::binding::symbols::ScopeId;
use crate::binding::symbols::SymbolInventory;
use crate::binding::symbols::SymbolScopeKind;
use crate::state::require::Require;
use crate::test::util::mk_multi_file_state;

fn inventory(code: &str) -> SymbolInventory {
    let (handles, state) = mk_multi_file_state(&[("m", code)], Require::Everything, false);
    state
        .transaction()
        .get_answers(&handles["m"])
        .unwrap()
        .bindings()
        .symbol_inventory()
        .expect("Everything retains lexical symbols")
        .clone()
}

/// Locate a name within an exact, unique source snippet, independently of IDs.
fn range(code: &str, snippet: &str, name: &str) -> TextRange {
    assert_eq!(
        code.matches(snippet).count(),
        1,
        "ambiguous snippet: {snippet}"
    );
    let start = code.find(snippet).unwrap() + snippet.find(name).unwrap();
    TextRange::at(
        TextSize::try_from(start).unwrap(),
        TextSize::try_from(name.len()).unwrap(),
    )
}

fn symbol(
    inventory: &SymbolInventory,
    code: &str,
    snippet: &str,
    name: &str,
    kind: DefinitionKind,
) -> BindingSymbolId {
    let range = range(code, snippet, name);
    let definition = inventory.definition_at(range).expect(snippet);
    assert_eq!(definition.range, range);
    assert_eq!(definition.kind, kind, "{snippet}");
    let id = *definition.symbol.as_ref().expect(snippet);
    assert_eq!(inventory.symbol(id).name.as_str(), name);
    id
}

fn declared_scope(inventory: &SymbolInventory, name: &str) -> ScopeId {
    let mut scopes = inventory.scopes().filter(|(_, scope)| {
        scope
            .declaration
            .as_ref()
            .is_some_and(|declaration| declaration.id.as_str() == name)
    });
    let (id, _) = scopes.next().expect(name);
    assert!(scopes.next().is_none(), "ambiguous declaration: {name}");
    id
}

/// The binder retains annotation scopes even for declarations without type parameters.
fn declaration_parent(inventory: &SymbolInventory, id: ScopeId) -> Option<ScopeId> {
    let mut parent = inventory.scope(id).parent;
    while let Some(id) = parent {
        if inventory.scope(id).kind != SymbolScopeKind::Annotation {
            break;
        }
        parent = inventory.scope(id).parent;
    }
    parent
}

#[test]
fn repeated_assignments_shadowing_and_occurrence_kinds() {
    let code = r#"
from typing import NamedTuple
Generated = NamedTuple("Generated", [])
item = 1
item = 2
def outer():
    item = 3
    item += 4
    return item
def item():
    pass
class item:
    pass
"#;
    let inventory = inventory(code);
    symbol(
        &inventory,
        code,
        "Generated =",
        "Generated",
        DefinitionKind::Assignment,
    );
    let first = symbol(
        &inventory,
        code,
        "item = 1",
        "item",
        DefinitionKind::Assignment,
    );
    for (snippet, kind) in [
        ("item = 2", DefinitionKind::Assignment),
        ("def item():", DefinitionKind::Function),
        ("class item:", DefinitionKind::Class),
    ] {
        assert_eq!(first, symbol(&inventory, code, snippet, "item", kind));
    }
    let local = symbol(
        &inventory,
        code,
        "item = 3",
        "item",
        DefinitionKind::Assignment,
    );
    assert_eq!(
        local,
        symbol(
            &inventory,
            code,
            "item += 4",
            "item",
            DefinitionKind::AugmentedAssignment
        )
    );
    assert_ne!(first, local);
    let module = inventory.symbol(first).owner;
    let outer = declared_scope(&inventory, "outer");
    assert_eq!(inventory.scope(module).kind, SymbolScopeKind::Module);
    assert_eq!(inventory.scope(module).parent, None);
    assert_eq!(inventory.symbol(local).owner, outer);
    assert_eq!(declaration_parent(&inventory, outer), Some(module));
    assert_eq!(
        inventory
            .symbols()
            .filter(|(_, s)| s.name == "item")
            .count(),
        2
    );
    let definitions: Vec<_> = inventory.definitions().collect();
    assert!(
        definitions
            .windows(2)
            .all(|pair| pair[0].range.start() < pair[1].range.start())
    );
    assert!(
        inventory
            .definition_at(range(code, "return item", "item"))
            .is_none()
    );
}

#[test]
fn global_writes_share_module_ownership() {
    let code = r#"
existing = 0
def writer():
    global existing, introduced
    existing = 1
    (existing := 4)
    introduced = 2
def another():
    global introduced
    introduced = 3
"#;
    let inventory = inventory(code);
    let existing = symbol(
        &inventory,
        code,
        "existing = 0",
        "existing",
        DefinitionKind::Assignment,
    );
    assert_eq!(
        existing,
        symbol(
            &inventory,
            code,
            "existing = 1",
            "existing",
            DefinitionKind::Assignment
        )
    );
    assert_eq!(
        existing,
        symbol(
            &inventory,
            code,
            "existing := 4",
            "existing",
            DefinitionKind::Assignment
        )
    );
    assert!(
        inventory
            .definition_at(range(code, "global existing", "existing"))
            .is_none()
    );
    let introduced = symbol(
        &inventory,
        code,
        "introduced = 2",
        "introduced",
        DefinitionKind::Assignment,
    );
    assert_eq!(
        introduced,
        symbol(
            &inventory,
            code,
            "introduced = 3",
            "introduced",
            DefinitionKind::Assignment
        )
    );
    let module = inventory.symbol(existing).owner;
    assert_eq!(inventory.scope(module).kind, SymbolScopeKind::Module);
    assert_eq!(inventory.symbol(introduced).owner, module);
    for name in ["writer", "another"] {
        assert_eq!(
            declaration_parent(&inventory, declared_scope(&inventory, name)),
            Some(module)
        );
    }
}

#[test]
fn nonlocal_skips_nonbinding_scope_and_follows_capture_chain() {
    let code = r#"
def outer():
    value = 0
    def middle():
        def writer():
            nonlocal value
            value = 1
    def relay():
        nonlocal value
        value = 2
        def leaf():
            nonlocal value
            value = 3
            (value := 4)
"#;
    let inventory = inventory(code);
    let value = symbol(
        &inventory,
        code,
        "value = 0",
        "value",
        DefinitionKind::Assignment,
    );
    for snippet in ["value = 1", "value = 2", "value = 3", "value := 4"] {
        assert_eq!(
            value,
            symbol(
                &inventory,
                code,
                snippet,
                "value",
                DefinitionKind::Assignment
            )
        );
    }
    assert_eq!(
        inventory.symbol(value).owner,
        declared_scope(&inventory, "outer")
    );
    for (child, parent) in [("writer", "middle"), ("leaf", "relay"), ("relay", "outer")] {
        assert_eq!(
            declaration_parent(&inventory, declared_scope(&inventory, child)),
            Some(declared_scope(&inventory, parent))
        );
    }
}

#[test]
fn unresolved_nonlocal_has_no_symbol() {
    let code = r#"
missing = 0
def invalid():
    nonlocal missing
    missing = 1
"#;
    let inventory = inventory(code);
    let definition = inventory
        .definition_at(range(code, "missing = 1", "missing"))
        .unwrap();
    assert_eq!(definition.kind, DefinitionKind::Assignment);
    assert!(definition.symbol.is_err());
    assert_eq!(
        inventory
            .symbols()
            .filter(|(_, s)| s.name == "missing")
            .count(),
        1
    );
}

#[test]
fn comprehensions_and_lambdas_bound_walrus_ownership() {
    let code = r#"
def outer():
    values = [(captured := element) for element in range(3)]
    callbacks = [lambda argument: (local := argument) for index in range(3)]
    callback = lambda parameter: (standalone := parameter)
"#;
    let inventory = inventory(code);
    let outer = declared_scope(&inventory, "outer");
    let captured = symbol(
        &inventory,
        code,
        "captured := element",
        "captured",
        DefinitionKind::Assignment,
    );
    assert_eq!(inventory.symbol(captured).owner, outer);
    let element = symbol(
        &inventory,
        code,
        "for element in",
        "element",
        DefinitionKind::Assignment,
    );
    let comprehension = inventory.symbol(element).owner;
    assert_eq!(
        inventory.scope(comprehension).kind,
        SymbolScopeKind::Comprehension
    );
    assert_eq!(inventory.scope(comprehension).parent, Some(outer));

    let index = symbol(
        &inventory,
        code,
        "for index in",
        "index",
        DefinitionKind::Assignment,
    );
    let nested_comprehension = inventory.symbol(index).owner;
    assert_ne!(comprehension, nested_comprehension);
    assert_eq!(
        inventory.scope(nested_comprehension).kind,
        SymbolScopeKind::Comprehension
    );
    for (parameter, walrus, snippet, parent) in [
        (
            "argument",
            "local",
            "lambda argument:",
            nested_comprehension,
        ),
        ("parameter", "standalone", "lambda parameter:", outer),
    ] {
        let parameter = symbol(
            &inventory,
            code,
            snippet,
            parameter,
            DefinitionKind::Parameter,
        );
        let walrus = symbol(
            &inventory,
            code,
            &format!("{walrus} :="),
            walrus,
            DefinitionKind::Assignment,
        );
        let lambda = inventory.symbol(parameter).owner;
        assert_eq!(inventory.symbol(walrus).owner, lambda);
        assert_eq!(inventory.scope(lambda).kind, SymbolScopeKind::Lambda);
        assert_eq!(inventory.scope(lambda).parent, Some(parent));
        assert!(inventory.scope(lambda).declaration.is_none());
    }
}

#[test]
fn parameters_nested_declarations_and_receiver_fields() {
    let code = r#"
class Base:
    inherited = 1
class Child(Base):
    def method(self, parameter):
        self.owned = parameter
        print(self.inherited)
        def nested(argument):
            return argument
        class Local:
            pass
"#;
    let inventory = inventory(code);
    let child = declared_scope(&inventory, "Child");
    let method = declared_scope(&inventory, "method");
    assert!(matches!(
        inventory.scope(child).kind,
        SymbolScopeKind::Class(_)
    ));
    assert_eq!(inventory.scope(method).kind, SymbolScopeKind::Function);
    assert_eq!(declaration_parent(&inventory, method), Some(child));
    for name in ["self", "parameter"] {
        let parameter = symbol(
            &inventory,
            code,
            "def method(self, parameter):",
            name,
            DefinitionKind::Parameter,
        );
        assert_eq!(inventory.symbol(parameter).owner, method);
    }
    for (name, snippet, kind, owner) in [
        ("method", "def method(", DefinitionKind::Function, child),
        ("nested", "def nested(", DefinitionKind::Function, method),
        ("Local", "class Local:", DefinitionKind::Class, method),
        (
            "argument",
            "def nested(argument):",
            DefinitionKind::Parameter,
            declared_scope(&inventory, "nested"),
        ),
        ("owned", "self.owned =", DefinitionKind::Field, child),
    ] {
        let symbol = symbol(&inventory, code, snippet, name, kind);
        assert_eq!(inventory.symbol(symbol).owner, owner);
        assert!(
            inventory
                .scope(owner)
                .range
                .contains_range(range(code, snippet, name))
        );
    }
    assert_eq!(
        declaration_parent(&inventory, declared_scope(&inventory, "Local")),
        Some(method)
    );
    assert!(
        inventory
            .definition_at(range(code, "self.inherited", "inherited"))
            .is_none()
    );
    assert!(
        !inventory
            .symbols()
            .any(|(_, symbol)| symbol.owner == child && symbol.name == "inherited")
    );
}

#[test]
fn imports_and_type_parameters() {
    let code = "from dependency import *\nimport dependency as alias\nfrom ...bad import missing\nfirst = 3\ndef generic[T](value: T): ...\n";
    let (handles, state) = mk_multi_file_state(
        &[("m", code), ("dependency", "first = 1\nsecond = 2\n")],
        Require::Everything,
        false,
    );
    let transaction = state.transaction();
    let answers = transaction.get_answers(&handles["m"]).unwrap();
    let inventory = answers.bindings().symbol_inventory().unwrap();
    symbol(
        inventory,
        code,
        "import missing",
        "missing",
        DefinitionKind::Import,
    );
    let wildcard = range(code, "import *", "*");
    let mut imported: Vec<_> = inventory
        .definitions()
        .filter(|site| site.range == wildcard)
        .map(|site| {
            assert_eq!(site.kind, DefinitionKind::Import);
            let id = *site.symbol.as_ref().unwrap();
            (inventory.symbol(id).name.as_str(), id)
        })
        .collect();
    imported.sort_by_key(|(name, _)| *name);
    assert_eq!(
        imported.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(
        imported[0].1,
        symbol(
            inventory,
            code,
            "first = 3",
            "first",
            DefinitionKind::Assignment
        )
    );
    let alias = symbol(inventory, code, "as alias", "alias", DefinitionKind::Import);
    assert_eq!(
        inventory.scope(inventory.symbol(alias).owner).kind,
        SymbolScopeKind::Module
    );
    let parameter = symbol(
        inventory,
        code,
        "generic[T]",
        "T",
        DefinitionKind::TypeParameter,
    );
    let annotation_scope = inventory.symbol(parameter).owner;
    assert_eq!(
        inventory.scope(annotation_scope).kind,
        SymbolScopeKind::Annotation
    );
    assert_eq!(
        inventory.scope(declared_scope(inventory, "generic")).parent,
        Some(annotation_scope)
    );
}
