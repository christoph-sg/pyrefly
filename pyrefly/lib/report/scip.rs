/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::Path;

use lsp_types::Uri;
use pyrefly_build::handle::Handle;
use pyrefly_python::module_name::ModuleName;
use pyrefly_util::absolutize::Absolutize;
use pyrefly_util::unix_path::path_to_unix_string;
use scip::symbol::format_symbol;
use scip::types::Descriptor;
use scip::types::Document;
use scip::types::Index;
use scip::types::Metadata;
use scip::types::Occurrence;
use scip::types::PositionEncoding;
use scip::types::Symbol;
use scip::types::SymbolRole;
use scip::types::TextEncoding;
use scip::types::ToolInfo;
use scip::types::descriptor::Suffix;

use crate::state::state::Transaction;

#[cfg(test)]
mod snapshot;

const SCHEME: &str = "scip-python";
const MANAGER: &str = "python";

#[derive(Debug, Clone, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
}

pub fn scip(
    transaction: &Transaction,
    handles: &[Handle],
    project_root: &Path,
    project: &Package,
    version: &str,
) -> Index {
    let project_root = project_root.absolutize();
    let mut documents: Vec<Document> = handles
        .iter()
        .filter_map(|handle| document(transaction, handle, &project_root, project))
        .collect();
    documents.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Index {
        metadata: Some(Metadata {
            project_root: Uri::from_directory_path(&project_root)
                .map(|uri| uri.to_string())
                .unwrap_or_default(),
            tool_info: Some(ToolInfo {
                name: "pyrefly".to_owned(),
                version: version.to_owned(),
                ..Default::default()
            })
            .into(),
            text_document_encoding: TextEncoding::UTF8.into(),
            ..Default::default()
        })
        .into(),
        documents,
        ..Default::default()
    }
}

fn document(
    transaction: &Transaction,
    handle: &Handle,
    project_root: &Path,
    project: &Package,
) -> Option<Document> {
    let module = transaction.get_module_info(handle)?;
    let relative_path = module.path().as_path().strip_prefix(project_root).ok()?;
    Some(Document {
        language: "python".to_owned(),
        relative_path: path_to_unix_string(relative_path),
        position_encoding: PositionEncoding::UTF8CodeUnitOffsetFromLineStart.into(),
        occurrences: vec![Occurrence {
            range: vec![0, 0, 0],
            symbol: module_symbol(project, module.name()),
            symbol_roles: SymbolRole::Definition as i32,
            ..Default::default()
        }],
        ..Default::default()
    })
}

fn descriptor(name: &str, suffix: Suffix) -> Descriptor {
    Descriptor {
        name: name.to_owned(),
        suffix: suffix.into(),
        ..Default::default()
    }
}

fn global_symbol(package: &Package, descriptors: Vec<Descriptor>) -> String {
    // The SCIP spec escapes spaces in package fields by doubling them.
    let escape = |x: &str| x.replace(' ', "  ");
    format_symbol(Symbol {
        scheme: SCHEME.to_owned(),
        package: Some(scip::types::Package {
            manager: MANAGER.to_owned(),
            name: escape(&package.name),
            version: escape(&package.version),
            ..Default::default()
        })
        .into(),
        descriptors,
        ..Default::default()
    })
}

fn module_symbol(package: &Package, module: ModuleName) -> String {
    global_symbol(
        package,
        vec![
            descriptor(module.as_str(), Suffix::Namespace),
            descriptor("__init__", Suffix::Meta),
        ],
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pyrefly_python::module_path::ModulePath;
    use pyrefly_util::fs_anyhow;

    use super::*;
    use crate::report::scip::snapshot::format_snapshot;
    use crate::state::require::Require;
    use crate::test::util::TestEnv;

    fn snapshots_dir() -> PathBuf {
        let path = std::env::var("SCIP_SNAPSHOTS_PATH")
            .expect("SCIP_SNAPSHOTS_PATH env var not set: buck should set this automatically");
        std::env::current_dir().unwrap().join(path)
    }

    fn project_package(snapshots: &Path, case: &str) -> Package {
        let info: serde_json::Value = serde_json::from_str(
            &fs_anyhow::read_to_string(&snapshots.join("packageInfo.json")).unwrap(),
        )
        .unwrap();
        let package = info["special"].get(case).unwrap_or(&info["default"]);
        Package {
            name: package["name"].as_str().unwrap().to_owned(),
            version: package["version"].as_str().unwrap().to_owned(),
        }
    }

    fn assert_snapshot(case: &str) {
        let snapshots = snapshots_dir();
        let input_dir = snapshots.join("input").join(case);
        let output_dir = snapshots.join("output").join(case);

        let mut env = TestEnv::new();
        let mut handles = Vec::new();
        for path in glob::glob(&format!("{}/**/*.py", input_dir.display())).unwrap() {
            let path = path.unwrap();
            let name =
                ModuleName::from_relative_path(path.strip_prefix(&input_dir).unwrap()).unwrap();
            env.add_real_path(name.as_str(), path.clone());
            handles.push(Handle::new(
                name,
                ModulePath::filesystem(path),
                env.sys_info(),
            ));
        }
        let (state, _) = env
            .with_default_require_level(Require::Everything)
            .to_state();
        let index = scip(
            &state.transaction(),
            &handles,
            &input_dir,
            &project_package(&snapshots, case),
            "test",
        );

        let update = std::env::var("SCIP_UPDATE_SNAPSHOTS").is_ok();
        for doc in &index.documents {
            let input = fs_anyhow::read_to_string(&input_dir.join(&doc.relative_path)).unwrap();
            let actual = format_snapshot(&input, doc, &index.external_symbols);
            let output_path = output_dir.join(&doc.relative_path);
            if update {
                fs_anyhow::create_dir_all(output_path.parent().unwrap()).unwrap();
                fs_anyhow::write(&output_path, actual).unwrap();
                continue;
            }
            let expected = fs_anyhow::read_to_string(&output_path).unwrap_or_else(|_| {
                panic!(
                    "{} does not exist. Run with SCIP_UPDATE_SNAPSHOTS=1 to generate it.",
                    output_path.display()
                )
            });
            pretty_assertions::assert_eq!(
                expected,
                actual,
                "Snapshot mismatch for {}. Run with SCIP_UPDATE_SNAPSHOTS=1 to update.",
                output_path.display()
            );
        }
    }

    macro_rules! snapshot_tests {
        ($($case:ident),* $(,)?) => {
            $(
                #[test]
                fn $case() {
                    assert_snapshot(stringify!($case));
                }
            )*
        };
    }

    snapshot_tests!(
        aliased_import,
        builtin_imports,
        class_nohint,
        comprehensions,
        dunder_vars,
        empty_project_name_version,
        enclosing_range_class,
        enclosing_range_function,
        f_string,
        file_from_module_import,
        nameparts_import,
        nested_items,
        odd_pkg_name_1,
        pyproject_1,
        relative_import,
        single_class,
        single_function,
        unique,
        unresolved_import,
    );
}
