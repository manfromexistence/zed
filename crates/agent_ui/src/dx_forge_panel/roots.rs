use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::dx_project_context::DxProjectContext;
use crate::dx_project_context::project_root_key;

#[derive(Clone)]
pub(super) struct ForgeRootContext {
    workspace_root: PathBuf,
    dx_metadata_root: PathBuf,
    canonical_forge_root: PathBuf,
    legacy_forge_root: PathBuf,
    history_forge_root: PathBuf,
    shared_fallback: bool,
}

impl ForgeRootContext {
    pub(super) fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub(super) fn forge_package_status_candidates(&self) -> Vec<PathBuf> {
        vec![
            self.workspace_root
                .join(".forge")
                .join("receipts")
                .join("package-status.json"),
            self.workspace_root
                .join(".dx")
                .join("forge")
                .join("package-status.json"),
        ]
    }

    pub(super) fn forge_remote_registry_path(&self) -> PathBuf {
        self.workspace_root.join(".forge").join("remotes.json")
    }

    pub(super) fn forge_machine_cache_root(&self) -> PathBuf {
        self.dx_metadata_root.clone()
    }

    fn is_configured(&self) -> bool {
        self.canonical_forge_root.is_dir()
            || self.legacy_forge_root.is_dir()
            || self.history_forge_root.is_dir()
    }

    fn is_workspace_configured(&self) -> bool {
        !self.shared_fallback && self.is_configured()
    }
}

pub(super) fn forge_root_contexts(workspace_roots: &[String]) -> Vec<ForgeRootContext> {
    if workspace_roots.is_empty() {
        return Vec::new();
    }

    let mut seen = HashSet::new();
    let mut contexts = Vec::new();

    for context in DxProjectContext::contexts_for_workspace_roots(workspace_roots) {
        push_context(&mut contexts, &mut seen, forge_root_context(context));
    }

    if let Some(context) = shared_fallback_context() {
        push_context(&mut contexts, &mut seen, context);
    }

    contexts
}

pub(super) fn configured_forge_root_count(workspace_roots: &[String]) -> usize {
    forge_root_contexts(workspace_roots)
        .into_iter()
        .filter(ForgeRootContext::is_workspace_configured)
        .count()
}

fn forge_root_context(context: DxProjectContext) -> ForgeRootContext {
    let canonical_forge_root = context.workspace_root.join(".forge");
    let legacy_forge_root = context.dx_metadata_root.join("forge");
    let history_forge_root = context.workspace_root.join("tools").join("dx-forge");

    ForgeRootContext {
        workspace_root: context.workspace_root,
        dx_metadata_root: context.dx_metadata_root,
        canonical_forge_root,
        legacy_forge_root,
        history_forge_root,
        shared_fallback: false,
    }
}

fn shared_fallback_context() -> Option<ForgeRootContext> {
    let context = DxProjectContext::detect(DxProjectContext::shared_fallback_root())?;

    Some(ForgeRootContext {
        history_forge_root: context.workspace_root.join("tools").join("dx-forge"),
        workspace_root: context.workspace_root,
        dx_metadata_root: context.dx_metadata_root,
        canonical_forge_root: shared_fallback_forge_root(),
        legacy_forge_root: shared_fallback_metadata_forge_root(),
        shared_fallback: true,
    })
}

fn shared_fallback_forge_root() -> PathBuf {
    DxProjectContext::shared_fallback_root().join(".forge")
}

fn shared_fallback_metadata_forge_root() -> PathBuf {
    DxProjectContext::shared_fallback_root()
        .join(".dx")
        .join("forge")
}

fn push_context(
    contexts: &mut Vec<ForgeRootContext>,
    seen: &mut HashSet<String>,
    context: ForgeRootContext,
) {
    if seen.insert(project_root_key(context.workspace_root())) {
        contexts.push(context);
    }
}
