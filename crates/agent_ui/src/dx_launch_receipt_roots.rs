use std::path::PathBuf;

use crate::dx_project_context::DxProjectContext;

pub(crate) fn active_launch_receipt_root(workspace_roots: &[String]) -> PathBuf {
    let roots = launch_receipt_roots(workspace_roots);
    roots
        .iter()
        .find(|root| root.is_dir())
        .cloned()
        .or_else(|| roots.last().cloned())
        .unwrap_or_else(fallback_launch_receipt_root)
}

pub(crate) fn launch_receipt_roots(workspace_roots: &[String]) -> Vec<PathBuf> {
    DxProjectContext::receipt_root_candidates(
        workspace_roots,
        "launch",
        DxProjectContext::shared_fallback_root(),
    )
}

fn fallback_launch_receipt_root() -> PathBuf {
    DxProjectContext::receipt_root_for(DxProjectContext::shared_fallback_root(), "launch")
        .unwrap_or_else(|| {
            DxProjectContext::shared_fallback_root()
                .join(".dx")
                .join("receipts")
                .join("launch")
        })
}
