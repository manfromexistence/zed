use super::{SOURCE_AUDIT_LATEST, SOURCE_AUDIT_MARKDOWN};
use crate::dx_project_context::DxProjectContext;
use std::path::PathBuf;
const SOURCE_AUDIT_KIND: &str = "launch-source";
const DX_STUDIO_QA_AUDIT_KIND: &str = "dx-studio-www-qa";
pub(super) struct SourceAuditPaths {
    pub root: PathBuf,
    pub latest_path: PathBuf,
    pub markdown_path: PathBuf,
    pub dx_studio_qa_path: PathBuf,
    pub root_exists: bool,
    pub latest_present: bool,
    pub markdown_present: bool,
    pub dx_studio_qa_present: bool,
}

pub(super) fn source_audit_paths(workspace_roots: &[String]) -> SourceAuditPaths {
    let root = active_audit_root(workspace_roots, SOURCE_AUDIT_KIND);
    let latest_path = root.join(SOURCE_AUDIT_LATEST);
    let markdown_path = root.join(SOURCE_AUDIT_MARKDOWN);
    let dx_studio_qa_path =
        active_audit_root(workspace_roots, DX_STUDIO_QA_AUDIT_KIND).join(SOURCE_AUDIT_LATEST);

    SourceAuditPaths {
        root_exists: root.is_dir(),
        latest_present: latest_path.is_file(),
        markdown_present: markdown_path.is_file(),
        dx_studio_qa_present: dx_studio_qa_path.is_file(),
        root,
        latest_path,
        markdown_path,
        dx_studio_qa_path,
    }
}

fn active_audit_root(workspace_roots: &[String], audit_kind: &str) -> PathBuf {
    let roots = DxProjectContext::audit_root_candidates(
        workspace_roots,
        audit_kind,
        DxProjectContext::shared_fallback_root(),
    );
    roots
        .iter()
        .find(|root| root.is_dir())
        .cloned()
        .or_else(|| roots.last().cloned())
        .unwrap_or_else(|| fallback_audit_root(audit_kind))
}

fn fallback_audit_root(audit_kind: &str) -> PathBuf {
    DxProjectContext::audit_root_for(DxProjectContext::shared_fallback_root(), audit_kind)
        .unwrap_or_else(|| {
            DxProjectContext::shared_fallback_root()
                .join(".dx")
                .join("audit")
                .join(audit_kind)
        })
}
