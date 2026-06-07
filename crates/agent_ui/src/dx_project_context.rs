use std::{
    collections::HashSet,
    path::{Component, MAIN_SEPARATOR, Path, PathBuf},
};

use crate::dx_deploy_root_key::deploy_root_key;

pub const DX_PROJECT_CONTEXT_ANCESTOR_LIMIT: usize = 8;
pub const DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT: usize = 4;
pub const DX_SHARED_FALLBACK_ROOT: &str = r"G:\Dx";

const DX_CONFIG_FILE_NAME: &str = "dx";
const DX_METADATA_DIR_NAME: &str = ".dx";
const DX_DIAGNOSTICS_LATEST_PATH: &[&str] = &[".dx", "diagnostics", "latest.json"];
const DX_CHECK_LATEST_RECEIPT_PATH: &[&str] = &[".dx", "receipts", "check", "check-latest.json"];
const DX_FORGE_RECEIPT_ROOT: &[&str] = &[".dx", "receipts", "forge"];
const DX_WWW_ROUTE_MANIFEST_PATH: &[&str] = &[".dx", "www", "routes-latest.json"];
const DX_STYLE_RECEIPT_ROOT: &[&str] = &[".dx", "receipts", "style"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DxProjectContext {
    pub workspace_root: PathBuf,
    pub root_exists: bool,
    pub dx_config_path: PathBuf,
    pub dx_metadata_root: PathBuf,
    pub diagnostics_receipt_path: PathBuf,
    pub check_receipt_path: PathBuf,
    pub forge_receipt_root: PathBuf,
    pub www_route_manifest_path: PathBuf,
    pub style_receipt_root: PathBuf,
}

impl DxProjectContext {
    pub fn detect(root: impl AsRef<Path>) -> Option<Self> {
        let workspace_root = normalize_project_root(root.as_ref())?;
        let dx_metadata_root = workspace_root.join(DX_METADATA_DIR_NAME);

        Some(Self {
            root_exists: workspace_root.is_dir(),
            dx_config_path: workspace_root.join(DX_CONFIG_FILE_NAME),
            diagnostics_receipt_path: join_relative_path(
                &workspace_root,
                DX_DIAGNOSTICS_LATEST_PATH,
            ),
            check_receipt_path: join_relative_path(&workspace_root, DX_CHECK_LATEST_RECEIPT_PATH),
            forge_receipt_root: join_relative_path(&workspace_root, DX_FORGE_RECEIPT_ROOT),
            www_route_manifest_path: join_relative_path(
                &workspace_root,
                DX_WWW_ROUTE_MANIFEST_PATH,
            ),
            style_receipt_root: join_relative_path(&workspace_root, DX_STYLE_RECEIPT_ROOT),
            dx_metadata_root,
            workspace_root,
        })
    }

    pub fn contexts_for_workspace_roots(workspace_roots: &[String]) -> Vec<Self> {
        let mut seen = HashSet::new();
        workspace_roots
            .iter()
            .filter_map(|root| workspace_root_candidate(root))
            .filter(|root| push_unique_path_key(&mut seen, root))
            .take(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT)
            .filter_map(|root| Self::detect(&root))
            .collect()
    }

    pub fn check_receipt_candidates(
        workspace_roots: &[String],
        fallback_receipt: impl AsRef<Path>,
    ) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut candidates = Vec::new();
        for context in Self::contexts_for_workspace_roots(workspace_roots) {
            push_unique_path(&mut candidates, &mut seen, context.check_receipt_path);
        }

        push_unique_path(
            &mut candidates,
            &mut seen,
            normalize_project_root(fallback_receipt.as_ref())
                .unwrap_or_else(|| fallback_receipt.as_ref().to_path_buf()),
        );
        candidates
    }

    pub fn source_scoped_receipt_roots(
        source_path: Option<&str>,
        workspace_root: Option<&str>,
        receipt_kind: &str,
    ) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut roots = Vec::new();
        let Some(source_path) =
            source_path.and_then(|path| normalize_absolute_project_path(Path::new(path)))
        else {
            return roots;
        };
        let Some(workspace_root) =
            workspace_root.and_then(|path| normalize_absolute_project_path(Path::new(path)))
        else {
            return roots;
        };

        if !path_is_same_or_child(&source_path, &workspace_root) {
            return roots;
        }

        let Some(parent) = source_path.parent() else {
            return roots;
        };

        for ancestor in parent.ancestors().take(DX_PROJECT_CONTEXT_ANCESTOR_LIMIT) {
            if !path_is_same_or_child(ancestor, &workspace_root) {
                break;
            }

            push_unique_path(
                &mut roots,
                &mut seen,
                ancestor
                    .join(DX_METADATA_DIR_NAME)
                    .join("receipts")
                    .join(receipt_kind),
            );

            if same_path_key(ancestor, &workspace_root) {
                break;
            }
        }

        roots
    }

    pub fn receipts_root(&self) -> PathBuf {
        self.dx_metadata_root.join("receipts")
    }

    pub fn receipts_root_for(root: impl AsRef<Path>) -> Option<PathBuf> {
        Self::detect(root).map(|context| context.receipts_root())
    }

    pub fn receipt_root(&self, receipt_kind: &str) -> PathBuf {
        self.dx_metadata_root.join("receipts").join(receipt_kind)
    }

    pub fn receipt_root_for(root: impl AsRef<Path>, receipt_kind: &str) -> Option<PathBuf> {
        Self::detect(root).map(|context| context.receipt_root(receipt_kind))
    }

    pub fn audit_root(&self, audit_kind: &str) -> PathBuf {
        self.dx_metadata_root.join("audit").join(audit_kind)
    }

    pub fn audit_root_for(root: impl AsRef<Path>, audit_kind: &str) -> Option<PathBuf> {
        Self::detect(root).map(|context| context.audit_root(audit_kind))
    }

    pub fn workspace_receipt_roots(
        workspace_roots: &[PathBuf],
        receipt_kind: &str,
    ) -> Vec<PathBuf> {
        let mut seen_roots = HashSet::new();
        let mut seen_receipts = HashSet::new();
        let mut roots = Vec::new();

        for context in workspace_roots
            .iter()
            .filter_map(|root| workspace_path_candidate(root))
            .filter(|root| push_unique_path_key(&mut seen_roots, root))
            .take(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT)
            .filter_map(|root| Self::detect(&root))
        {
            push_unique_path(
                &mut roots,
                &mut seen_receipts,
                context.receipt_root(receipt_kind),
            );
        }

        roots
    }

    pub fn receipts_root_candidates(
        workspace_roots: &[String],
        fallback_workspace_root: impl AsRef<Path>,
    ) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut roots = Vec::new();

        for context in Self::contexts_for_workspace_roots(workspace_roots)
            .into_iter()
            .take(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT)
        {
            push_unique_path(&mut roots, &mut seen, context.receipts_root());
        }

        let fallback =
            Self::receipts_root_for(fallback_workspace_root.as_ref()).unwrap_or_else(|| {
                fallback_workspace_root
                    .as_ref()
                    .join(DX_METADATA_DIR_NAME)
                    .join("receipts")
            });
        push_unique_path(&mut roots, &mut seen, fallback);

        roots
    }

    pub fn audit_root_candidates(
        workspace_roots: &[String],
        audit_kind: &str,
        fallback_workspace_root: impl AsRef<Path>,
    ) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut roots = Vec::new();

        for context in Self::contexts_for_workspace_roots(workspace_roots)
            .into_iter()
            .take(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT)
        {
            push_unique_path(&mut roots, &mut seen, context.audit_root(audit_kind));
        }

        let fallback = Self::audit_root_for(fallback_workspace_root.as_ref(), audit_kind)
            .unwrap_or_else(|| {
                fallback_workspace_root
                    .as_ref()
                    .join(DX_METADATA_DIR_NAME)
                    .join("audit")
                    .join(audit_kind)
            });
        push_unique_path(&mut roots, &mut seen, fallback);

        roots
    }

    pub fn receipt_root_candidates(
        workspace_roots: &[String],
        receipt_kind: &str,
        fallback_workspace_root: impl AsRef<Path>,
    ) -> Vec<PathBuf> {
        let mut seen = HashSet::new();
        let mut roots = Vec::new();

        for context in Self::contexts_for_workspace_roots(workspace_roots)
            .into_iter()
            .take(DX_PROJECT_CONTEXT_WORKSPACE_ROOT_LIMIT)
        {
            push_unique_path(&mut roots, &mut seen, context.receipt_root(receipt_kind));
        }

        let fallback = Self::receipt_root_for(fallback_workspace_root.as_ref(), receipt_kind)
            .unwrap_or_else(|| {
                fallback_workspace_root
                    .as_ref()
                    .join(DX_METADATA_DIR_NAME)
                    .join("receipts")
                    .join(receipt_kind)
            });
        push_unique_path(&mut roots, &mut seen, fallback);

        roots
    }

    pub fn shared_fallback_root() -> PathBuf {
        PathBuf::from(DX_SHARED_FALLBACK_ROOT)
    }

    pub fn shared_launch_examples_root() -> PathBuf {
        Self::shared_fallback_root()
            .join("cli")
            .join("fixtures")
            .join("launch-examples")
    }

    pub fn shared_receipt_cache_artifact_path() -> PathBuf {
        Self::shared_fallback_root()
            .join(DX_METADATA_DIR_NAME)
            .join("receipts")
            .join("receipt-cache.dxrc")
    }
}

pub fn normalize_project_root(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    let mut saw_component = false;

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
                saw_component = true;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
                saw_component = true;
            }
        }
    }

    (saw_component && !normalized.as_os_str().is_empty()).then_some(normalized)
}

pub fn project_root_key(path: &Path) -> String {
    let normalized = normalize_project_root(path).unwrap_or_else(|| path.to_path_buf());
    deploy_root_key(&normalized)
}

fn workspace_root_candidate(root: &str) -> Option<PathBuf> {
    let root = root.trim();
    if root.is_empty() {
        return None;
    }
    let normalized = normalize_project_root(Path::new(root))?;
    normalized.is_absolute().then_some(normalized)
}

fn workspace_path_candidate(root: &Path) -> Option<PathBuf> {
    let normalized = normalize_project_root(root)?;
    normalized.is_absolute().then_some(normalized)
}

fn normalize_absolute_project_path(path: &Path) -> Option<PathBuf> {
    path.is_absolute()
        .then(|| normalize_project_root(path))
        .flatten()
}

fn join_relative_path(root: &Path, components: &[&str]) -> PathBuf {
    let mut path = root.to_path_buf();
    for component in components {
        path.push(component);
    }
    path
}

fn push_unique_path(paths: &mut Vec<PathBuf>, seen: &mut HashSet<String>, path: PathBuf) {
    if push_unique_path_key(seen, &path) {
        paths.push(path);
    }
}

fn push_unique_path_key(seen: &mut HashSet<String>, path: &Path) -> bool {
    seen.insert(project_root_key(path))
}

fn same_path_key(left: &Path, right: &Path) -> bool {
    project_root_key(left) == project_root_key(right)
}

fn path_is_same_or_child(path: &Path, root: &Path) -> bool {
    let path_key = project_root_key(path);
    let root_key = project_root_key(root);
    if path_key == root_key {
        return true;
    }

    let child_prefix = if root_key.ends_with(MAIN_SEPARATOR) {
        root_key
    } else {
        format!("{root_key}{MAIN_SEPARATOR}")
    };
    path_key.starts_with(&child_prefix)
}
