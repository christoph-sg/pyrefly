/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use pyrefly_build::handle::Handle;
pub use pyrefly_glean_schema::report::glean::facts;
pub use pyrefly_glean_schema::report::glean::schema;

use crate::report::glean::facts::Glean;
use crate::state::state::Transaction;

pub mod convert;

pub fn glean(transaction: &Transaction, handle: &Handle, ownership: bool) -> String {
    fn f(transaction: &Transaction, handle: &Handle, ownership: bool) -> Option<Glean> {
        Some(convert::glean(transaction, handle, ownership))
    }

    let data = f(transaction, handle, ownership)
        .expect("Glean data be ready")
        .entries;
    serde_json::to_string_pretty(&data).unwrap()
}

#[test]
fn snapshot_test() {
    use std::env;
    use std::fs;

    use crate::state::require::Require;
    use crate::test::util::mk_multi_file_state_assert_no_errors;

    // Helper function to sort JSON for deterministic comparison
    fn normalize_json(json_str: &str) -> String {
        let mut value: serde_json::Value =
            serde_json::from_str(json_str).expect("Failed to parse JSON");

        fn sort_arrays(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Array(arr) => {
                    for item in arr.iter_mut() {
                        sort_arrays(item);
                    }
                    arr.sort_by(|a, b| {
                        serde_json::to_string(a)
                            .unwrap_or_default()
                            .cmp(&serde_json::to_string(b).unwrap_or_default())
                    });
                }
                serde_json::Value::Object(obj) => {
                    for (_, v) in obj.iter_mut() {
                        sort_arrays(v);
                    }
                }
                _ => {}
            }
        }

        sort_arrays(&mut value);
        serde_json::to_string_pretty(&value).unwrap()
    }

    // Simple line-by-line diff like git/source control
    fn create_simple_diff(expected: &str, actual: &str) -> String {
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();

        let mut diff_lines = Vec::new();
        let max_lines = expected_lines.len().max(actual_lines.len());

        for i in 0..max_lines {
            match (expected_lines.get(i), actual_lines.get(i)) {
                (Some(exp_line), Some(act_line)) if exp_line != act_line => {
                    diff_lines.push(format!("- {exp_line}"));
                    diff_lines.push(format!("+ {act_line}"));
                }
                (Some(exp_line), None) => {
                    diff_lines.push(format!("- {exp_line}"));
                }
                (None, Some(act_line)) => {
                    diff_lines.push(format!("+ {act_line}"));
                }
                _ => {} // Lines are the same or both None
            }
        }

        if diff_lines.is_empty() {
            "No differences found".to_owned()
        } else {
            format!("Diff (- expected, + actual):\n{}", diff_lines.join("\n"))
        }
    }

    let simple_code = r#"def hello_world():
    return "Hello, World!"

x = 42
"""Answer to everything"""
y = hello_world()"#;

    let classes_code = r#"
import typing

T = typing.TypeVar('T')

class Animal(typing.Generic[T]):
    def __init__(self, name: str):
        self.name = name

    def speak(self) -> str:
        return f"{self.name} makes a sound"

class Dog(Animal):
    def speak(self) -> str:
        return f"{self.name} barks"

dog = Dog("Buddy")
sound = dog.speak()"#;

    let imports_code = r#"import os
from typing import List
from .simple import hello_world
from os import *

def process_files(paths: List[str]) -> int:
    """Count files in given paths."""
    count = 0
    for path in paths:
        if os.path.exists(path):
            count += 1
    return count

# Use relative import
greeting = hello_world()
result = process_files(["/tmp", "/home"])"#;

    let try_except_code = r#"try:
    pass
except Exception as ex:
    pass"#;

    let return_types = r#"from typing import Callable, Dict, List, Optional, Set, Union

class FooClass:
    pass

def complex_return() -> (
    Union[
        Union[FooClass, Callable[[int], bool], int | str | None],
        List[int | str],
        Dict[str, int | str],
        Optional[int | str],
        Set[List[str]],
    ]
):
    return None
    "#;

    let calls = r#"from dataclasses import dataclass
@dataclass(frozen=True)
class A:
    pass

print("hello")
print(b"world")

"#;

    let type_lit_str = r#"from typing import Union
import typing

class B:
    class A:
        pass

    def other(self, x: "B") -> "Union[list[B], A, bool]":
        return False

    def dotted(self, x: "B.A") -> "typing.List[int]":
        return [0]
"#;

    let exports_all = r#"from typing import Union as U
x = 1
y = 2

__all__ = ["x", "U"]

__all__ += ["y"]
"#;

    let generated_file = format!(
        r#" # {}generated SignedSource<<abcde123456>>
 # @codegen-command : scripts/foo/bar
 # @codegen-source FooConfig: this/is/the/Way.php
 # @codegen-class : BarCodeGen

def helloworld():
    pass
"#,
        "@",
    );

    let attr_annotation = r#"from typing import Optional

class Node:
    pass

class Container:
    def setup(self) -> None:
        self.x: Node = Node()
        self.y: Optional[Node] = None
"#;

    let files = [
        ("simple", simple_code),
        ("classes", classes_code),
        ("imports", imports_code),
        ("try_except", try_except_code),
        ("return_types", return_types),
        ("calls", calls),
        ("type_lit_str", type_lit_str),
        ("exports_all", exports_all),
        ("generated_file", &generated_file),
        ("attr_annotation", attr_annotation),
    ];
    let (handles, state) = mk_multi_file_state_assert_no_errors(&files, Require::Everything);
    let transaction = state.transaction();

    let update_snapshots = env::var("GLEAN_SNAPSHOTS_WRITE_PATH").is_ok();

    let snapshots_path = if !update_snapshots {
        env::var("GLEAN_SNAPSHOTS_PATH")
            .expect("GLEAN_SNAPSHOTS_PATH env var not set: buck should set this automatically")
    } else {
        env::var("GLEAN_SNAPSHOTS_WRITE_PATH").unwrap()
    };

    let mut snapshot_dir = std::env::current_dir().expect("Failed to get current directory");
    snapshot_dir.push(snapshots_path);

    for (filename, _) in &files {
        let handle = handles
            .get(filename)
            .expect("Handle should exist for test file");
        let glean_output = glean(&transaction, handle, false);
        let output_path = snapshot_dir.join(format!("{filename}.json"));

        if update_snapshots {
            let normalized_output = normalize_json(&glean_output);
            let output_with_header = &normalized_output;
            fs::write(&output_path, output_with_header).expect("Failed to write JSON output");
            println!(
                "Updated snapshot for {} -> {}",
                filename,
                output_path.display()
            );
            continue;
        }

        assert!(
            output_path.exists(),
            "Snapshot file does not exist: {}\nRun with GLEAN_SNAPSHOTS_WRITE_PATH set to update snapshots.",
            output_path.display()
        );

        let expected_content =
            fs::read_to_string(&output_path).expect("Failed to read existing snapshot");

        let normalized_actual = normalize_json(&glean_output);
        let normalized_expected = normalize_json(&expected_content);

        assert!(
            normalized_actual.trim() == normalized_expected.trim(),
            "Snapshot mismatch for {filename}!\n\n{}\n\nTo update snapshots, run with GLEAN_SNAPSHOTS_WRITE_PATH set. If using buck, add -- --env GLEAN_SNAPSHOTS_WRITE_PATH=\"path\" to the command.",
            create_simple_diff(&normalized_expected, &normalized_actual)
        );

        println!("✓ Snapshot matches for {filename}");
    }

    if update_snapshots {
        println!("All snapshots updated successfully!");
    } else {
        println!("All snapshots match!");
    }
}

#[test]
fn ownership_test() {
    use crate::report::glean::facts::GleanEntry;
    use crate::state::require::Require;
    use crate::test::util::mk_multi_file_state_assert_no_errors;

    let (handles, state) =
        mk_multi_file_state_assert_no_errors(&[("simple", "x = 1")], Require::Everything);
    let handle = handles
        .get("simple")
        .expect("Handle should exist for test file");
    let entries = convert::glean(&state.transaction(), handle, true).entries;

    for entry in entries {
        if let GleanEntry::Predicate {
            predicate, unit, ..
        } = entry
        {
            assert_eq!(
                unit.as_deref(),
                Some("simple.py"),
                "{predicate} should be owned by its file"
            );
        }
    }
}

#[cfg(test)]
mod symbol_tests {
    use serde_json::Value;

    use super::glean;
    use crate::state::require::Require;
    use crate::test::util::mk_multi_file_state;

    fn index(code: &str) -> Value {
        let (handles, state) = mk_multi_file_state(&[("m", code)], Require::Everything, false);
        serde_json::from_str(&glean(&state.transaction(), &handles["m"], false)).unwrap()
    }

    fn facts<'a>(index: &'a Value, predicate: &str) -> &'a [Value] {
        index
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["predicate"] == predicate)
            .unwrap()["facts"]
            .as_array()
            .unwrap()
    }

    fn variables(index: &Value) -> Vec<(&str, u64)> {
        facts(index, "python.DeclarationLocation.4")
            .iter()
            .filter_map(|fact| {
                Some((
                    fact["key"]["declaration"]["variable"]["key"]["name"]["key"].as_str()?,
                    fact["key"]["span"]["start"].as_u64().unwrap(),
                ))
            })
            .collect()
    }

    fn xrefs(index: &Value) -> Vec<(&str, u64)> {
        facts(index, "python.XRefsViaNameByTarget.4")
            .iter()
            .flat_map(|fact| {
                let target = fact["key"]["target"]["key"].as_str().unwrap();
                fact["key"]["spans"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(move |span| (target, span["start"].as_u64().unwrap()))
            })
            .collect()
    }

    #[test]
    fn captures_use_the_binding_owner() {
        let code = r#"
shared = 0
def outer():
    value = 1
    def middle():
        def writer(flag):
            if flag:
                global shared
                nonlocal value
            shared = 2
            value = 3
            return value, shared
    return later()
def invalid():
    nonlocal absent
    absent = 4
def later():
    return 5
"#;
        let result = index(code);
        let vars = variables(&result);
        for (source, expected) in [
            ("shared = 0", "m.shared"),
            ("shared = 2", "m.shared"),
            ("value = 1", "m.outer.<locals>.value"),
            ("value = 3", "m.outer.<locals>.value"),
        ] {
            assert!(
                vars.contains(&(expected, code.find(source).unwrap() as u64)),
                "{source}: {vars:?}"
            );
        }
        assert!(!vars.iter().any(|(name, _)| name.ends_with(".absent")));
        let refs = xrefs(&result);
        for (source, expected) in [
            ("value, shared", "m.outer.<locals>.value"),
            ("shared\n", "m.shared"),
            ("later()\ndef invalid", "m.later"),
        ] {
            assert!(
                refs.contains(&(expected, code.rfind(source).unwrap() as u64)),
                "{refs:?}"
            );
        }
    }

    #[test]
    fn anonymous_scopes_and_receiver_fields_have_definitions() {
        let code = r#"
x = 0
values = [(saved := x) for x in range(3)]
callback = lambda x: (local := x)
class Base:
    inherited = 1
class Child(Base):
    def method(self, value):
        self.owned = value
        return self.inherited
"#;
        let result = index(code);
        let vars = variables(&result);
        let comp = code.find("[(saved").unwrap();
        let lambda = code.find("lambda x").unwrap();
        for (name, position) in [
            ("m.saved".to_owned(), code.find("saved :=").unwrap()),
            (
                format!("m.<comprehension@{comp}>.x"),
                code.find("x in range").unwrap(),
            ),
            (format!("m.<lambda@{lambda}>.x"), lambda + "lambda ".len()),
            (
                format!("m.<lambda@{lambda}>.local"),
                code.find("local :=").unwrap(),
            ),
            ("m.Child.owned".to_owned(), code.find("owned =").unwrap()),
        ] {
            assert!(
                vars.contains(&(name.as_str(), position as u64)),
                "{name}: {vars:?}"
            );
        }
        assert!(!vars.iter().any(|(name, _)| *name == "m.Child.inherited"));
        let refs = xrefs(&result);
        for (name, position) in [
            (
                format!("m.<comprehension@{comp}>.x"),
                code.find("x) for").unwrap(),
            ),
            (
                "m.Base.inherited".to_owned(),
                code.find("inherited\n").unwrap(),
            ),
        ] {
            assert!(refs.contains(&(name.as_str(), position as u64)), "{refs:?}");
        }
        assert!(!refs.iter().any(|(name, _)| *name == "m.Child.inherited"));
        // Lambda parameter definitions are indexed, but their reads still depend on
        // the existing definition resolver, which does not resolve them yet.
    }

    #[test]
    fn local_imports_and_cross_module_fields() {
        let code = r#"
def first():
    from dependency import Base as alias
    return alias()
def second():
    from dependency import Other as alias
    import dependency.nested
    return alias(), dependency.nested
from dependency import Child
value = Child().field
"#;
        let (handles, state) = mk_multi_file_state(
            &[
                ("m", code),
                (
                    "dependency",
                    "class Base:\n    field = 1\nclass Child(Base): pass\nclass Other: pass\n",
                ),
                ("dependency.nested", "value = 1\n"),
            ],
            Require::Everything,
            false,
        );
        let result: Value =
            serde_json::from_str(&glean(&state.transaction(), &handles["m"], false)).unwrap();
        let mut imports: Vec<_> = facts(&result, "python.DeclarationLocation.4")
            .iter()
            .filter_map(|fact| fact["key"]["declaration"]["imp"]["key"]["as_name"]["key"].as_str())
            .collect();
        imports.sort_unstable();
        assert_eq!(
            imports,
            [
                "m.Child",
                "m.first.<locals>.alias",
                "m.second.<locals>.alias",
                "m.second.<locals>.dependency",
                "m.second.<locals>.dependency.nested"
            ]
        );
        let refs = xrefs(&result);
        for (target, position) in [
            ("dependency.Base", code.find("alias()").unwrap()),
            ("m.first.<locals>.alias", code.find("alias()").unwrap()),
            ("dependency.Other", code.rfind("alias()").unwrap()),
            ("m.second.<locals>.alias", code.rfind("alias()").unwrap()),
            ("dependency.Base.field", code.find("field\n").unwrap()),
        ] {
            assert!(refs.contains(&(target, position as u64)), "{refs:?}");
        }
        assert!(
            !refs
                .iter()
                .any(|(name, _)| *name == "dependency.Child.field")
        );
    }
}
