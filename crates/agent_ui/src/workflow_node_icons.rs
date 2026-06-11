use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::OnceLock;

use gpui::{AnyElement, IntoElement, SharedString};
use serde::Deserialize;
use ui::{Color, DxUiIcon, Icon, IconName, IconSize, dx_icon, dx_icon_data_dir};

const WORKFLOW_NODE_ICON_PREVIEW_CACHE_VERSION: &str = "v1";
const WORKFLOW_NODE_ICON_ID_PREVIEW_CHARS: usize = 36;
const WORKFLOW_NODE_ICON_CANDIDATE_LIMIT: usize = 8;
const WORKFLOW_NODE_SVGL_ICON_NAMES: &[&str] = &[
    "airtable",
    "anthropic_black",
    "discord",
    "dropbox",
    "figma",
    "github_dark",
    "gitlab",
    "gmail",
    "google",
    "drive",
    "hubspot",
    "jira",
    "linear",
    "mailchimp",
    "microsoft-teams",
    "mongodb",
    "mysql",
    "notion",
    "openai",
    "postgresql",
    "redis",
    "salesforce",
    "shopify",
    "slack",
    "stripe",
    "telegram",
    "trello",
    "twilio",
    "whatsapp",
    "youtube",
    "zoom",
];

static WORKFLOW_NODE_ICON_PREVIEW_CACHE: OnceLock<HashMap<String, SharedString>> = OnceLock::new();
static WORKFLOW_NODE_SVGL_ICON_PACK: OnceLock<Option<SvglIconPack>> = OnceLock::new();

#[derive(Clone)]
pub(crate) enum WorkflowNodeIconAsset {
    Embedded(IconName),
    ExternalSvg(SharedString),
}

impl WorkflowNodeIconAsset {
    pub(crate) fn render(&self, size: IconSize, color: Color) -> AnyElement {
        match self {
            WorkflowNodeIconAsset::Embedded(icon) => Icon::new(*icon).size(size).color(color),
            WorkflowNodeIconAsset::ExternalSvg(svg) => {
                Icon::from_external_svg_with_original_colors(svg.clone()).size(size)
            }
        }
        .into_any_element()
    }
}

pub(crate) fn workflow_node_icon_asset_for(
    icon_hint: Option<&str>,
    category_hint: Option<&str>,
    display_name: &str,
) -> WorkflowNodeIconAsset {
    for candidate in workflow_node_svg_candidates(icon_hint, category_hint, display_name) {
        if let Some(path) = workflow_node_svg_preview_cache().get(candidate.as_str()) {
            return WorkflowNodeIconAsset::ExternalSvg(path.clone());
        }
        if let Some(path) = write_workflow_node_icon_preview_for_candidate(&candidate) {
            return WorkflowNodeIconAsset::ExternalSvg(path.into());
        }
    }

    WorkflowNodeIconAsset::Embedded(workflow_node_fallback_icon_for(
        icon_hint,
        category_hint,
        display_name,
    ))
}

pub(crate) fn workflow_node_element_id(prefix: &str, raw: &str) -> SharedString {
    let mut sanitized = String::with_capacity(prefix.len() + raw.len().min(48) + 18);
    sanitized.push_str(prefix);
    sanitized.push('-');
    for character in raw
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        .take(WORKFLOW_NODE_ICON_ID_PREVIEW_CHARS)
    {
        sanitized.push(character.to_ascii_lowercase());
    }
    if sanitized.ends_with('-') {
        sanitized.push_str("item");
    }
    let _ = write!(&mut sanitized, "-{:016x}", fnv1a64(raw));
    sanitized.into()
}

fn workflow_node_svg_candidates(
    icon_hint: Option<&str>,
    category_hint: Option<&str>,
    display_name: &str,
) -> Vec<String> {
    let hints = workflow_node_hints(icon_hint, category_hint, display_name);
    let mut candidates = Vec::with_capacity(WORKFLOW_NODE_ICON_CANDIDATE_LIMIT);

    push_explicit_source_candidate(icon_hint, &mut candidates);
    push_named_candidate(&hints, &mut candidates, "google drive", "drive");
    push_named_candidate(&hints, &mut candidates, "google sheets", "google");
    push_named_candidate(&hints, &mut candidates, "google calendar", "google");
    push_named_candidate(&hints, &mut candidates, "google analytics", "google");
    push_named_candidate(&hints, &mut candidates, "microsoft teams", "microsoft");
    push_named_candidate(&hints, &mut candidates, "postgres", "postgresql");
    push_named_candidate(&hints, &mut candidates, "email", "gmail");
    push_named_candidate(&hints, &mut candidates, "mail", "gmail");
    push_named_candidate(&hints, &mut candidates, "github", "github_dark");
    push_named_candidate(&hints, &mut candidates, "anthropic", "anthropic_black");

    for name in WORKFLOW_NODE_SVGL_ICON_NAMES {
        push_named_candidate(&hints, &mut candidates, name, name);
        if candidates.len() >= WORKFLOW_NODE_ICON_CANDIDATE_LIMIT {
            break;
        }
    }

    candidates
}

fn push_explicit_source_candidate(icon_hint: Option<&str>, candidates: &mut Vec<String>) {
    let Some(icon_hint) = icon_hint else {
        return;
    };
    let Some((pack, slug)) = icon_hint.split_once(':') else {
        return;
    };
    if pack.trim().eq_ignore_ascii_case("svgl") {
        push_unique_candidate(candidates, normalized_icon_slug(slug));
    }
}

fn push_named_candidate(hints: &str, candidates: &mut Vec<String>, needle: &str, candidate: &str) {
    if hints.contains(needle) {
        push_unique_candidate(candidates, candidate.to_string());
    }
}

fn push_unique_candidate(candidates: &mut Vec<String>, candidate: String) {
    if candidate.is_empty() || candidates.iter().any(|item| item == &candidate) {
        return;
    }
    candidates.push(candidate);
}

fn workflow_node_fallback_icon_for(
    icon_hint: Option<&str>,
    category_hint: Option<&str>,
    display_name: &str,
) -> IconName {
    let hints = workflow_node_hints(icon_hint, category_hint, display_name);

    if hints.contains("github") {
        IconName::Github
    } else if hints.contains("gitlab") {
        IconName::Gitlab
    } else if hints.contains("bitbucket") {
        IconName::Bitbucket
    } else if hints.contains("gitea") {
        IconName::Gitea
    } else if hints.contains("git") {
        IconName::FileGit
    } else if hints.contains("database")
        || hints.contains("postgres")
        || hints.contains("mysql")
        || hints.contains("sql")
        || hints.contains("redis")
    {
        IconName::DatabaseZap
    } else if hints.contains("mail") || hints.contains("email") || hints.contains("gmail") {
        IconName::Envelope
    } else if hints.contains("chat")
        || hints.contains("message")
        || hints.contains("slack")
        || hints.contains("discord")
        || hints.contains("teams")
    {
        IconName::QueueMessage
    } else if hints.contains("cloud")
        || hints.contains("drive")
        || hints.contains("s3")
        || hints.contains("dropbox")
    {
        IconName::CloudDownload
    } else if hints.contains("calendar") || hints.contains("schedule") || hints.contains("cron") {
        IconName::Clock
    } else if hints.contains("terminal")
        || hints.contains("command")
        || hints.contains("shell")
        || hints.contains("script")
    {
        IconName::Terminal
    } else if hints.contains("file")
        || hints.contains("document")
        || hints.contains("sheet")
        || hints.contains("csv")
        || hints.contains("json")
    {
        IconName::FileTextOutlined
    } else if hints.contains("image")
        || hints.contains("video")
        || hints.contains("media")
        || hints.contains("photo")
    {
        IconName::Image
    } else if hints.contains("trigger") {
        IconName::PlayOutlined
    } else if hints.contains("http") || hints.contains("web") || hints.contains("api") {
        IconName::ToolWeb
    } else {
        dx_icon(DxUiIcon::Plugins)
    }
}

fn workflow_node_hints(
    icon_hint: Option<&str>,
    category_hint: Option<&str>,
    display_name: &str,
) -> String {
    [
        icon_hint.unwrap_or_default(),
        category_hint.unwrap_or_default(),
        display_name,
    ]
    .join(" ")
    .replace(['_', '-', '.', ':', '/'], " ")
    .to_ascii_lowercase()
}

fn workflow_node_svg_preview_cache() -> &'static HashMap<String, SharedString> {
    WORKFLOW_NODE_ICON_PREVIEW_CACHE.get_or_init(load_workflow_node_svg_preview_cache)
}

fn load_workflow_node_svg_preview_cache() -> HashMap<String, SharedString> {
    let Some(pack) = svgl_icon_pack() else {
        return HashMap::new();
    };

    let mut cache = HashMap::with_capacity(WORKFLOW_NODE_SVGL_ICON_NAMES.len());
    for name in WORKFLOW_NODE_SVGL_ICON_NAMES {
        let Some(icon) = pack.icons.get(*name) else {
            continue;
        };
        if let Ok(path) = write_workflow_node_icon_preview(name, icon) {
            cache.insert((*name).to_string(), path.into());
        }
    }
    cache
}

fn svgl_icon_pack() -> Option<&'static SvglIconPack> {
    WORKFLOW_NODE_SVGL_ICON_PACK
        .get_or_init(load_svgl_icon_pack)
        .as_ref()
}

fn load_svgl_icon_pack() -> Option<SvglIconPack> {
    let path = dx_icon_data_dir().join("svgl.json");
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<SvglIconPack>(&text).ok()
}

fn write_workflow_node_icon_preview_for_candidate(candidate: &str) -> Option<String> {
    let pack = svgl_icon_pack()?;
    for alias in svgl_candidate_aliases(candidate) {
        if let Some(icon) = pack.icons.get(alias.as_str())
            && let Ok(path) = write_workflow_node_icon_preview(&alias, icon)
        {
            return Some(path);
        }
    }
    None
}

fn write_workflow_node_icon_preview(name: &str, icon: &SvglIconBody) -> std::io::Result<String> {
    let path = workflow_node_icon_preview_path(name);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &path,
            wrap_workflow_node_icon_body(&icon.body, icon.width(), icon.height()),
        )?;
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

fn workflow_node_icon_preview_path(name: &str) -> PathBuf {
    repo_root()
        .join("target")
        .join("workflow-node-icons")
        .join(WORKFLOW_NODE_ICON_PREVIEW_CACHE_VERSION)
        .join("svgl")
        .join(format!("{}.svg", sanitize_file_component(name)))
}

fn wrap_workflow_node_icon_body(body: &str, width: u32, height: u32) -> String {
    let body = body.trim();
    if body.starts_with("<svg") {
        body.to_string()
    } else {
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}"><g fill="currentColor">{body}</g></svg>"#
        )
    }
}

fn repo_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("G:/Dx/code"))
}

fn sanitize_file_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
            sanitized.push(character);
        } else {
            sanitized.push('-');
        }
    }
    sanitized
}

fn normalized_icon_slug(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .replace([' ', '.'], "-")
        .to_ascii_lowercase()
}

fn svgl_candidate_aliases(candidate: &str) -> Vec<String> {
    let normalized = normalized_icon_slug(candidate);
    let mut aliases = Vec::with_capacity(4);
    push_unique_candidate(&mut aliases, normalized.clone());
    push_unique_candidate(&mut aliases, normalized.replace('-', "_"));
    match normalized.as_str() {
        "github" => push_unique_candidate(&mut aliases, "github_dark".to_string()),
        "anthropic" => push_unique_candidate(&mut aliases, "anthropic_black".to_string()),
        "google-drive" => push_unique_candidate(&mut aliases, "drive".to_string()),
        "google-sheets" | "google-calendar" | "google-analytics" => {
            push_unique_candidate(&mut aliases, "google".to_string())
        }
        "microsoft-teams" => push_unique_candidate(&mut aliases, "microsoft".to_string()),
        _ => {}
    }
    aliases
}

fn fnv1a64(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Deserialize)]
struct SvglIconPack {
    icons: HashMap<String, SvglIconBody>,
}

#[derive(Deserialize)]
struct SvglIconBody {
    body: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
}

impl SvglIconBody {
    fn width(&self) -> u32 {
        self.width.unwrap_or(24).max(1)
    }

    fn height(&self) -> u32 {
        self.height.unwrap_or(24).max(1)
    }
}
