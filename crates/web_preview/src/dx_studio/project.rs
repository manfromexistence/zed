use std::{
    collections::HashSet,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use agent_ui::dx_project_context::DxProjectContext;

use super::{DxStudioProjectDetection, MAX_DX_MARKER_SCAN_BYTES, MAX_DX_MARKER_SCAN_FILES};

const MAX_DX_CARGO_TOML_SCAN_BYTES: u64 = 256 * 1024;
const MAX_DX_MARKER_SCAN_DIRS: usize = 128;
const MAX_DX_MARKER_SCAN_VISITED_DIRS: usize = 256;

struct DxMarkerScanBudget {
    files_left: usize,
    dirs_left: usize,
    visited_dirs: HashSet<PathBuf>,
}

impl DxMarkerScanBudget {
    fn new() -> Self {
        Self {
            files_left: MAX_DX_MARKER_SCAN_FILES,
            dirs_left: MAX_DX_MARKER_SCAN_DIRS,
            visited_dirs: HashSet::new(),
        }
    }

    fn take_file(&mut self) -> bool {
        if self.files_left == 0 {
            return false;
        }

        self.files_left -= 1;
        true
    }

    fn try_visit_dir(&mut self, dir: &Path) -> bool {
        if self.dirs_left == 0 || self.visited_dirs.len() >= MAX_DX_MARKER_SCAN_VISITED_DIRS {
            return false;
        }

        let dir_key = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        if !self.visited_dirs.insert(dir_key) {
            return false;
        }

        self.dirs_left -= 1;
        true
    }
}

pub fn detect_project(root: &Path) -> Option<DxStudioProjectDetection> {
    if !root.is_dir() {
        return None;
    }

    let project_context = DxProjectContext::detect(root);
    let mut confidence = 0u8;
    let mut reasons = Vec::new();
    let dx_file = project_context
        .as_ref()
        .map(|context| context.dx_config_path.clone())
        .unwrap_or_else(|| root.join("dx"));
    let legacy_toml = root.join("dx.config.toml");
    let app_dir = root.join("app");
    let components_dir = root.join("components");
    let dx_dir = project_context
        .as_ref()
        .map(|context| context.dx_metadata_root.clone())
        .unwrap_or_else(|| root.join(".dx"));
    let forge_receipt_root = project_context
        .as_ref()
        .map(|context| context.forge_receipt_root.clone())
        .unwrap_or_else(|| root.join(".dx").join("receipts").join("forge"));
    let visible_forge_dir = root.join("forge");
    let public_preview_manifest = root.join("public").join("preview-manifest.json");
    let www_route_manifest = project_context
        .as_ref()
        .map(|context| context.www_route_manifest_path.clone())
        .unwrap_or_else(|| root.join(".dx").join("www").join("routes-latest.json"));
    let launch_template = root.join("examples").join("launch-template");
    let node_modules = root.join("node_modules");

    if dx_file.is_file() {
        confidence = confidence.saturating_add(45);
        reasons.push("root dx config file".to_string());
    }

    if legacy_toml.is_file() {
        confidence = confidence.saturating_add(20);
        reasons.push("legacy dx.config.toml fallback".to_string());
    }

    if dx_dir.is_dir() {
        confidence = confidence.saturating_add(35);
        reasons.push(".dx project metadata".to_string());
    }

    if public_preview_manifest.is_file() {
        confidence = confidence.saturating_add(45);
        reasons.push("public preview-manifest.json".to_string());
    }

    if www_route_manifest.is_file() {
        confidence = confidence.saturating_add(45);
        reasons.push("DX WWW route manifest".to_string());
    }

    if app_dir.is_dir() {
        confidence = confidence.saturating_add(18);
        reasons.push("Next-familiar app route folder".to_string());
    }

    if components_dir.is_dir() {
        confidence = confidence.saturating_add(10);
        reasons.push("local components folder".to_string());
    }

    if forge_receipt_root.is_dir() || visible_forge_dir.is_dir() {
        confidence = confidence.saturating_add(18);
        reasons.push("Forge package boundary".to_string());
    }

    if launch_template.is_dir() {
        confidence = confidence.saturating_add(20);
        reasons.push("DX launch template sources".to_string());
    }

    if contains_dx_marker_in_project_sources(root) {
        confidence = confidence.saturating_add(45);
        reasons.push("source data-dx markers".to_string());
    }

    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.is_file() && cargo_toml_contains_dx_www_marker(&cargo_toml) {
        confidence = confidence.saturating_add(18);
        reasons.push("DX-WWW Rust workspace".to_string());
    }

    if confidence < 40 {
        return None;
    }

    Some(DxStudioProjectDetection {
        root: root.to_path_buf(),
        confidence: confidence.min(100),
        reasons,
        strict_dx_file: dx_file.is_file(),
        legacy_toml_present: legacy_toml.is_file(),
        node_modules_present: node_modules.exists(),
    })
}

fn cargo_toml_contains_dx_www_marker(path: &Path) -> bool {
    read_bounded_utf8_file(path, MAX_DX_CARGO_TOML_SCAN_BYTES)
        .map(|contents| contents.contains("dx-www") || contents.contains("dx_www"))
        .unwrap_or(false)
}

fn contains_dx_marker_in_project_sources(root: &Path) -> bool {
    let mut budget = DxMarkerScanBudget::new();
    for source_root in [
        root.join("app"),
        root.join("pages"),
        root.join("components"),
        root.join("src"),
        root.join("examples").join("launch-template"),
    ] {
        if source_root.is_file() {
            if !budget.take_file() {
                return false;
            }
            if dx_marker_source_file_contains_marker(&source_root) {
                return true;
            }
        } else if source_root.is_dir()
            && dx_marker_source_dir_contains_marker(&source_root, &mut budget)
        {
            return true;
        }
    }

    false
}

fn dx_marker_source_dir_contains_marker(root: &Path, budget: &mut DxMarkerScanBudget) -> bool {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if !budget.try_visit_dir(&dir) {
            continue;
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if !should_skip_dx_marker_scan_dir(&path) {
                    stack.push(path);
                }
            } else if file_type.is_file() && is_dx_marker_source_file(&path) {
                if !budget.take_file() {
                    return false;
                }
                if dx_marker_source_file_contains_marker(&path) {
                    return true;
                }
            }
        }
    }

    false
}

fn should_skip_dx_marker_scan_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | ".next" | ".turbo" | "build" | "dist" | "node_modules" | "out" | "target"
            )
        })
}

fn is_dx_marker_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension,
                "astro" | "html" | "js" | "jsx" | "mdx" | "svelte" | "ts" | "tsx" | "vue"
            )
        })
}

fn dx_marker_source_file_contains_marker(path: &Path) -> bool {
    read_bounded_utf8_file(path, MAX_DX_MARKER_SCAN_BYTES)
        .map(|contents| {
            [
                "data-dx-route",
                "data-dx-source",
                "data-dx-source-file",
                "data-dx-component",
                "data-dx-section",
                "data-dx-edit-id",
                "data-dx-edit-ops",
                "data-dx-hot-reload-target",
            ]
            .into_iter()
            .any(|marker| contents.contains(marker))
        })
        .unwrap_or(false)
}

fn read_bounded_utf8_file(path: &Path, max_bytes: u64) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    let mut limited = file.take(max_bytes + 1);
    limited.read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > max_bytes {
        return None;
    }

    String::from_utf8(bytes).ok()
}
