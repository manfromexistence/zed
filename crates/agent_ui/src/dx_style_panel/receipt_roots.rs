use std::path::PathBuf;

use crate::dx_project_context::DxProjectContext;

pub(super) fn active_style_receipt_roots(
    source_path: Option<&str>,
    workspace_root: Option<&str>,
) -> Vec<PathBuf> {
    DxProjectContext::source_scoped_receipt_roots(source_path, workspace_root, "style")
}
