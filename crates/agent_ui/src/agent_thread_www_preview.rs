#[allow(unused_imports)]
use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use gpui::{AnyElement, AnyEntity, App, WeakEntity, Window};
use workspace::Workspace;

pub const AGENT_WWW_TOOL_WHITEBOARD: &str = "whiteboard";
pub const AGENT_WWW_TOOL_SHADER: &str = "shader";

#[allow(dead_code)]
const DEFAULT_DEV_HOST: &str = "127.0.0.1";
#[allow(dead_code)]
const DEFAULT_DEV_PORT: u16 = 3000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentThreadCenterSurface {
    Messages,
    WwwPreview { tool_id: &'static str },
}

pub struct AgentThreadWwwPreviewHooks {
    pub open_url: fn(
        workspace: WeakEntity<Workspace>,
        url: String,
        existing: Option<AnyEntity>,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<AnyEntity>,
    pub deactivate: fn(preview: &AnyEntity, window: &mut Window, cx: &mut App),
    pub render: fn(preview: &AnyEntity, window: &mut Window, cx: &mut App) -> AnyElement,
}

static HOOKS: OnceLock<AgentThreadWwwPreviewHooks> = OnceLock::new();

pub fn register_agent_thread_www_preview_hooks(hooks: AgentThreadWwwPreviewHooks) {
    let _ = HOOKS.set(hooks);
}

pub fn agent_thread_www_preview_hooks() -> Option<&'static AgentThreadWwwPreviewHooks> {
    HOOKS.get()
}

pub fn is_agent_www_tool_preview_enabled(tool_id: &str) -> bool {
    // All web tool buttons are enabled for hardcoded HTML preview
    matches!(
        tool_id,
        "design"
            | "graphics"
            | "presentations"
            | "spreadsheets"
            | "video"
            | "music"
            | AGENT_WWW_TOOL_WHITEBOARD
            | "3d"
            | AGENT_WWW_TOOL_SHADER
            | "dx-web"
    ) && agent_thread_www_preview_hooks().is_some()
}

pub fn project_root_for_agent_www_tool(tool_id: &str) -> Option<PathBuf> {
    let www_root = find_dx_www_root()?;
    let sub = match tool_id {
        AGENT_WWW_TOOL_WHITEBOARD => Path::new("examples").join("whiteboard"),
        AGENT_WWW_TOOL_SHADER => Path::new("examples").join("shader"),
        _ => return None,
    };
    let project_root = www_root.join(sub);
    if project_root.is_dir() && project_root.join("dx").is_file() {
        Some(project_root)
    } else {
        None
    }
}

pub fn preview_url_for_agent_www_tool(tool_id: &str) -> Option<String> {
    // Serve a hardcoded HTML page with the tool name centered for all web tool buttons.
    let display_name = match tool_id {
        "design" => "Design",
        "graphics" => "Graphics",
        "presentations" => "Presentations",
        "spreadsheets" => "Spreadsheets",
        "video" => "Video",
        "music" => "Music",
        AGENT_WWW_TOOL_WHITEBOARD => "Whiteboard",
        "3d" => "3D",
        AGENT_WWW_TOOL_SHADER => "Shader",
        "dx-web" => "DX Web",
        _ => return None,
    };

    let html = format!(
        "<html><head><style>*{{margin:0;padding:0;box-sizing:border-box}}body{{display:flex;justify-content:center;align-items:center;height:100vh;width:100vw;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#1e1e2e;color:#cdd6f4}}h1{{font-size:3rem;font-weight:700;letter-spacing:0.02em}}</style></head><body><h1>{}</h1></body></html>",
        display_name
    );

    Some(format!("data:text/html,{}", html))
}

#[allow(dead_code)]
fn dev_server_origin(root: &Path) -> String {
    let mut host = DEFAULT_DEV_HOST.to_string();
    let mut port = DEFAULT_DEV_PORT;

    for config_path in [root.join("dx"), root.join("dx.config.toml")] {
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Some(dev_clause) = contents.lines().find_map(parse_dev_clause) {
                if let Some(parsed_host) = dev_clause.host {
                    host = parsed_host;
                }
                if let Some(parsed_port) = dev_clause.port {
                    port = parsed_port;
                }
            }
            break;
        }
    }

    // Best-effort: if dx dev auto-selected a different port (e.g. 3001 on conflict),
    // the project .log or .dx/*-dev-*.log will contain the "Development server running at http://..."
    // announcement. Prefer that so the embedded preview connects to the live instance.
    if let Some((actual_host, actual_port)) = scan_dev_server_port_from_logs(root) {
        host = actual_host;
        port = actual_port;
    }

    format!("http://{host}:{port}")
}

#[allow(dead_code)]
struct DevClauseValues {
    host: Option<String>,
    port: Option<u16>,
}

#[allow(dead_code)]
fn parse_dev_clause(line: &str) -> Option<DevClauseValues> {
    let line = line.split('#').next()?.trim();
    let args = line.strip_prefix("dev(")?.strip_suffix(')')?;
    let mut host = None;
    let mut port = None;

    for token in args.split_whitespace() {
        if let Some(value) = token.strip_prefix("host=") {
            host = Some(strip_quotes(value).to_string());
        } else if let Some(value) = token.strip_prefix("port=")
            && let Ok(parsed) = value.parse::<u16>()
        {
            port = Some(parsed);
        }
    }

    Some(DevClauseValues { host, port })
}

#[allow(dead_code)]
fn strip_quotes(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

#[allow(dead_code)]
fn find_dx_www_root() -> Option<PathBuf> {
    // Env override takes precedence
    if let Ok(env) = std::env::var("DX_WWW_ROOT") {
        let p = PathBuf::from(env);
        if is_dx_www_root(&p) {
            return Some(p);
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
        }
    }

    for start in candidates {
        let mut dir = Some(start.as_path());
        while let Some(d) = dir {
            if is_dx_www_root(d) {
                return Some(d.to_path_buf());
            }
            // Check sibling "www" next to this dir (e.g. when d points at G:\Dx\code or G:\Dx)
            let sibling = d.join("www");
            if is_dx_www_root(&sibling) {
                return Some(sibling);
            }
            dir = d.parent();
        }
    }

    // Environment-specific fallback (G drive DX layout)
    let fallback = PathBuf::from(r"G:\Dx\www");
    if is_dx_www_root(&fallback) {
        return Some(fallback);
    }

    None
}

#[allow(dead_code)]
fn is_dx_www_root(p: &Path) -> bool {
    p.join("examples").join("whiteboard").join("dx").is_file()
        && p.join("examples").join("shader").join("dx").is_file()
}

/// Scan recent dev server logs (at project root and .dx/) for the "Development server running at http://host:port"
/// line (emitted by `dx dev`). Returns the last seen host/port so the preview can target the live instance
/// even if dx chose a free port other than the one declared in the project's `dx` manifest.
#[allow(dead_code)]
fn scan_dev_server_port_from_logs(root: &Path) -> Option<(String, u16)> {
    let mut log_files: Vec<PathBuf> = Vec::new();

    // Collect *.log and *dev*.log candidates from root
    if let Ok(rd) = fs::read_dir(root) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if p.extension()
                    .map_or(false, |e| e.eq_ignore_ascii_case("log"))
                    || name.contains("dev")
                    || name.contains("server")
                {
                    log_files.push(p);
                }
            }
        }
    }

    // Also from .dx/ (where some dx runtimes and receipts put logs)
    let dxd = root.join(".dx");
    if let Ok(rd) = fs::read_dir(&dxd) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if p.extension()
                    .map_or(false, |e| e.eq_ignore_ascii_case("log"))
                    || name.contains("dev")
                {
                    log_files.push(p);
                }
            }
        }
    }

    for log_path in log_files {
        if let Ok(text) = fs::read_to_string(&log_path) {
            for line in text.lines().rev() {
                if let Some(pair) = parse_running_at_line(line) {
                    return Some(pair);
                }
            }
        }
    }
    None
}

#[allow(dead_code)]
fn parse_running_at_line(line: &str) -> Option<(String, u16)> {
    let lower = line.to_ascii_lowercase();
    let marker = "running at http://";
    let pos = lower.find(marker)?;
    let rest = &line[pos + marker.len()..];
    // Cut at first whitespace or common punctuation
    let url = rest
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ')' | '"' | '\'' | '>'))
        .next()
        .unwrap_or(rest)
        .trim()
        .trim_end_matches('/');
    if let Some((h, pstr)) = url.rsplit_once(':') {
        if let Ok(port) = pstr.trim().parse::<u16>() {
            let host = h.trim_start_matches(|c| c == '/' || c == ':').to_string();
            if !host.is_empty() && port > 0 {
                return Some((host, port));
            }
        }
    }
    None
}
