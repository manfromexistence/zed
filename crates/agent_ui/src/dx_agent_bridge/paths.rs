use std::{collections::HashSet, path::PathBuf};

use crate::dx_project_context::{DxProjectContext, project_root_key};

const DEFAULT_PROVIDER_CATALOG_PATH: &[&str] =
    &[".dx", "catalog", "agents", "provider-model-catalog.rkyv"];

pub(super) fn active_agent_receipt_root(workspace_roots: &[String]) -> PathBuf {
    let roots = DxProjectContext::receipt_root_candidates(
        workspace_roots,
        "agents",
        DxProjectContext::shared_fallback_root(),
    );
    roots
        .iter()
        .find(|root| root.is_dir())
        .cloned()
        .or_else(|| roots.last().cloned())
        .unwrap_or_else(default_agent_receipt_root)
}

pub(super) fn active_provider_catalog_path(workspace_roots: &[String]) -> PathBuf {
    let candidates = provider_catalog_path_candidates(workspace_roots);
    candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .or_else(|| candidates.last().cloned())
        .unwrap_or_else(default_provider_catalog_path)
}

pub(super) fn default_agent_receipt_root() -> PathBuf {
    DxProjectContext::receipt_root_for(DxProjectContext::shared_fallback_root(), "agents")
        .unwrap_or_else(|| {
            DxProjectContext::shared_fallback_root()
                .join(".dx")
                .join("receipts")
                .join("agents")
        })
}

pub(super) fn default_provider_catalog_path() -> PathBuf {
    provider_catalog_path_for_root(DxProjectContext::shared_fallback_root())
}

fn provider_catalog_path_candidates(workspace_roots: &[String]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    for context in DxProjectContext::contexts_for_workspace_roots(workspace_roots) {
        push_unique_path(
            &mut candidates,
            &mut seen,
            provider_catalog_path_for_root(context.workspace_root),
        );
    }

    push_unique_path(&mut candidates, &mut seen, default_provider_catalog_path());
    candidates
}

fn provider_catalog_path_for_root(root: PathBuf) -> PathBuf {
    let mut path = root;
    for component in DEFAULT_PROVIDER_CATALOG_PATH {
        path.push(component);
    }
    path
}

fn push_unique_path(paths: &mut Vec<PathBuf>, seen: &mut HashSet<String>, path: PathBuf) {
    if seen.insert(project_root_key(&path)) {
        paths.push(path);
    }
}
