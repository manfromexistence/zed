use std::path::{Path, PathBuf};

use crate::dx_deploy_hub_roots::dx_hub_root;
use crate::dx_deploy_root_key::deploy_root_key;
use crate::dx_project_context::DxProjectContext;

pub(crate) struct DxDeployCheckReceiptRoot {
    pub path: PathBuf,
    pub label: String,
    pub root_rank: u8,
}

pub(crate) fn check_receipt_roots(workspace_roots: &[PathBuf]) -> Vec<DxDeployCheckReceiptRoot> {
    let mut roots = Vec::new();

    for path in DxProjectContext::workspace_receipt_roots(workspace_roots, "check") {
        let label = path.display().to_string();
        push_check_root(&mut roots, path, label, 0);
    }

    let hub_root = dx_hub_root();
    push_check_root(
        &mut roots,
        check_receipt_path(&hub_root),
        format!("{}\\.dx\\receipts\\check", hub_root.display()),
        1,
    );
    push_check_root(
        &mut roots,
        check_receipt_path(hub_root.join("www")),
        format!("{}\\www\\.dx\\receipts\\check", hub_root.display()),
        2,
    );

    roots
}

fn check_receipt_path(root: impl AsRef<Path>) -> PathBuf {
    DxProjectContext::receipt_root_for(root.as_ref(), "check")
        .unwrap_or_else(|| root.as_ref().join(".dx").join("receipts").join("check"))
}

fn push_check_root(
    roots: &mut Vec<DxDeployCheckReceiptRoot>,
    path: PathBuf,
    label: String,
    root_rank: u8,
) {
    if path.as_os_str().is_empty() {
        return;
    }

    let path_key = deploy_root_key(&path);
    if roots
        .iter()
        .any(|root| deploy_root_key(&root.path) == path_key)
    {
        return;
    }

    roots.push(DxDeployCheckReceiptRoot {
        path,
        label,
        root_rank,
    });
}
