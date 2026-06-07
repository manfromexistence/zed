use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const CHECK_RECEIPT_SCHEMA: &str = "dx.check.receipt.v1";
const ZED_PANEL_SCHEMA: &str = "dx.check.zed_panel.v1";
const VIEW_MODEL_SCHEMA: &str = "dx.www.check_panel_view_model.v1";
const CHECK_PANEL_CACHE_TTL: Duration = Duration::from_secs(5);
const MAX_RECEIPT_BYTES: u64 = 256 * 1024;

mod parser;
mod reader;
mod web_audits;

#[derive(Clone)]
pub(crate) struct DxCheckPanelSnapshot {
    pub status: String,
    pub title: String,
    pub score_value: Option<u32>,
    pub score_max: Option<u32>,
    pub score_percent: Option<u8>,
    pub score_estimated: bool,
    pub weight_profile: String,
    pub receipt_path: PathBuf,
    pub receipt_present: bool,
    pub receipt_error: Option<String>,
    pub generated_at_unix_ms: Option<u64>,
    pub last_run_label: String,
    pub pass_count: Option<u32>,
    pub fail_count: Option<u32>,
    pub warn_count: Option<u32>,
    pub skipped_count: Option<u32>,
    pub duration_ms: Option<u64>,
    pub checked_paths: Vec<String>,
    pub skipped_expensive_checks: Vec<String>,
    pub refresh_command: String,
    pub detail_command: Option<String>,
    pub scoring_config_status: String,
    pub scoring_config_applies_to_score: bool,
    pub scoring_config_summary: String,
    pub sections: Vec<DxCheckPanelSection>,
    pub blockers: Vec<DxCheckPanelNotice>,
    pub warnings: Vec<DxCheckPanelNotice>,
    pub quick_fixes: Vec<DxCheckPanelQuickFix>,
    pub adapter_plans: Vec<DxCheckPanelAdapterPlan>,
    pub web_audits: Vec<DxCheckPanelWebAudit>,
    pub next_action: String,
    pub source_schema: String,
}

#[derive(Clone)]
pub(crate) struct DxCheckPanelSection {
    pub title: String,
    pub score: Option<u32>,
    pub max_score: Option<u32>,
    pub estimated: bool,
    pub status: String,
}

#[derive(Clone)]
pub(crate) struct DxCheckPanelNotice {
    pub code: String,
    pub message: String,
    pub next_action: Option<String>,
}

#[derive(Clone)]
pub(crate) struct DxCheckPanelQuickFix {
    pub label: String,
    pub next_action: String,
    pub risk_level: String,
    pub requires_user_approval: bool,
    pub writes_receipts: bool,
    pub command: Option<String>,
}

#[derive(Clone)]
pub(crate) struct DxCheckPanelAdapterPlan {
    pub label: String,
    pub target: String,
    pub command: String,
    pub parser: String,
    pub configured_from: Vec<String>,
    pub run_command: Option<String>,
}

#[derive(Clone)]
pub(crate) struct DxCheckPanelWebAudit {
    pub label: String,
    pub status: String,
    pub detail: String,
    pub url: String,
    pub source: Option<String>,
}

struct DxCheckPanelCache {
    cached_at: Instant,
    workspace_roots: Vec<String>,
    snapshot: DxCheckPanelSnapshot,
}

static CHECK_PANEL_CACHE: OnceLock<Mutex<Option<DxCheckPanelCache>>> = OnceLock::new();

pub(crate) fn dx_check_panel_snapshot(workspace_roots: &[String]) -> DxCheckPanelSnapshot {
    let normalized_roots = workspace_roots
        .iter()
        .map(|root| root.trim().to_string())
        .filter(|root| !root.is_empty())
        .collect::<Vec<_>>();

    let cache = CHECK_PANEL_CACHE.get_or_init(|| Mutex::new(None));
    let now = Instant::now();
    if let Ok(mut cache) = cache.lock() {
        if let Some(cached) = cache.as_ref() {
            if cached.workspace_roots == normalized_roots
                && now.duration_since(cached.cached_at) <= CHECK_PANEL_CACHE_TTL
            {
                return cached.snapshot.clone();
            }
        }

        let snapshot = reader::read_latest_check_panel(&normalized_roots);
        *cache = Some(DxCheckPanelCache {
            cached_at: now,
            workspace_roots: normalized_roots,
            snapshot: snapshot.clone(),
        });
        return snapshot;
    }

    reader::read_latest_check_panel(&normalized_roots)
}

pub(crate) fn invalidate_dx_check_panel_snapshot_cache() {
    let Some(cache) = CHECK_PANEL_CACHE.get() else {
        return;
    };
    if let Ok(mut cache) = cache.lock() {
        *cache = None;
    }
}

impl DxCheckPanelSnapshot {
    pub(crate) fn score_label(&self) -> String {
        match (self.score_value, self.score_max, self.score_percent) {
            (Some(score), Some(max_score), Some(percent)) => {
                let estimated = if self.score_estimated {
                    ", estimated"
                } else {
                    ""
                };
                format!("{score}/{max_score} ({percent}%{estimated})")
            }
            (Some(score), Some(max_score), None) => {
                let estimated = if self.score_estimated {
                    " estimated"
                } else {
                    ""
                };
                format!("{score}/{max_score}{estimated}")
            }
            _ => "No score claimed".to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
