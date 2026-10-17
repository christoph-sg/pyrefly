/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::Path;

use lsp_types::Uri;
use pyrefly_util::absolutize::Absolutize;
use scip::types::Index;
use scip::types::Metadata;
use scip::types::TextEncoding;
use scip::types::ToolInfo;

pub fn scip(project_root: &Path, version: &str) -> Index {
    let project_root = Uri::from_directory_path(project_root.absolutize())
        .map(|uri| uri.to_string())
        .unwrap_or_default();
    Index {
        metadata: Some(Metadata {
            project_root,
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
        ..Default::default()
    }
}
