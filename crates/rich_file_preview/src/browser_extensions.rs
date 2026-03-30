use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use paths::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserEngine {
    Chromium,
    FirefoxLike,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreviewExtensionSupport {
    NativeImport,
    DetectionOnly,
}

impl PreviewExtensionSupport {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NativeImport => "Loadable in preview",
            Self::DetectionOnly => "Detected only",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub path: PathBuf,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserProfile {
    pub browser_id: String,
    pub browser_name: String,
    pub profile_name: String,
    pub engine: BrowserEngine,
    pub profile_path: PathBuf,
    pub extensions_path: Option<PathBuf>,
    pub preview_support: PreviewExtensionSupport,
    pub extensions: Vec<DetectedExtension>,
    pub notes: Vec<String>,
}

impl BrowserProfile {
    pub fn display_name(&self) -> String {
        format!("{} / {}", self.browser_name, self.profile_name)
    }

    pub fn is_preview_compatible(&self) -> bool {
        self.preview_support == PreviewExtensionSupport::NativeImport
            && self.extensions_path.is_some()
    }

    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }

    pub fn profile_key(&self) -> String {
        sanitize_component(&format!("{}-{}", self.browser_id, self.profile_name))
    }
}

pub fn detect_browser_profiles() -> anyhow::Result<Vec<BrowserProfile>> {
    let mut profiles = Vec::new();

    for candidate in chromium_candidates() {
        profiles.extend(scan_chromium_browser(candidate)?);
    }

    for candidate in firefox_candidates() {
        profiles.extend(scan_firefox_browser(candidate)?);
    }

    profiles.sort_by(|left, right| {
        right
            .is_preview_compatible()
            .cmp(&left.is_preview_compatible())
            .then_with(|| right.extension_count().cmp(&left.extension_count()))
            .then_with(|| left.browser_name.cmp(&right.browser_name))
            .then_with(|| left.profile_name.cmp(&right.profile_name))
    });

    Ok(profiles)
}

pub fn preferred_profile_index(profiles: &[BrowserProfile]) -> Option<usize> {
    profiles
        .iter()
        .position(|profile| profile.is_preview_compatible() && !profile.extensions.is_empty())
        .or_else(|| profiles.iter().position(BrowserProfile::is_preview_compatible))
        .or_else(|| (!profiles.is_empty()).then_some(0))
}

struct ChromiumCandidate {
    browser_id: &'static str,
    browser_name: &'static str,
    root: PathBuf,
}

struct FirefoxCandidate {
    browser_id: &'static str,
    browser_name: &'static str,
    profiles_root: PathBuf,
}

fn chromium_candidates() -> Vec<ChromiumCandidate> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        if let Some(local) = local {
            candidates.push(ChromiumCandidate {
                browser_id: "chrome",
                browser_name: "Google Chrome",
                root: local.join("Google").join("Chrome").join("User Data"),
            });
            candidates.push(ChromiumCandidate {
                browser_id: "edge",
                browser_name: "Microsoft Edge",
                root: local.join("Microsoft").join("Edge").join("User Data"),
            });
            candidates.push(ChromiumCandidate {
                browser_id: "brave",
                browser_name: "Brave",
                root: local
                    .join("BraveSoftware")
                    .join("Brave-Browser")
                    .join("User Data"),
            });
            candidates.push(ChromiumCandidate {
                browser_id: "chromium",
                browser_name: "Chromium",
                root: local.join("Chromium").join("User Data"),
            });
            candidates.push(ChromiumCandidate {
                browser_id: "vivaldi",
                browser_name: "Vivaldi",
                root: local.join("Vivaldi").join("User Data"),
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_support = home_dir().join("Library").join("Application Support");
        candidates.push(ChromiumCandidate {
            browser_id: "chrome",
            browser_name: "Google Chrome",
            root: app_support.join("Google").join("Chrome"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "edge",
            browser_name: "Microsoft Edge",
            root: app_support.join("Microsoft Edge"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "brave",
            browser_name: "Brave",
            root: app_support.join("BraveSoftware").join("Brave-Browser"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "chromium",
            browser_name: "Chromium",
            root: app_support.join("Chromium"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "vivaldi",
            browser_name: "Vivaldi",
            root: app_support.join("Vivaldi"),
        });
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let config_root = home_dir().join(".config");
        candidates.push(ChromiumCandidate {
            browser_id: "chrome",
            browser_name: "Google Chrome",
            root: config_root.join("google-chrome"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "edge",
            browser_name: "Microsoft Edge",
            root: config_root.join("microsoft-edge"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "brave",
            browser_name: "Brave",
            root: config_root.join("BraveSoftware").join("Brave-Browser"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "chromium",
            browser_name: "Chromium",
            root: config_root.join("chromium"),
        });
        candidates.push(ChromiumCandidate {
            browser_id: "vivaldi",
            browser_name: "Vivaldi",
            root: config_root.join("vivaldi"),
        });
    }

    candidates
}

fn firefox_candidates() -> Vec<FirefoxCandidate> {
    let mut candidates = Vec::new();

    #[cfg(target_os = "windows")]
    {
        let roaming = std::env::var_os("APPDATA").map(PathBuf::from);
        if let Some(roaming) = roaming {
            candidates.push(FirefoxCandidate {
                browser_id: "firefox",
                browser_name: "Firefox",
                profiles_root: roaming.join("Mozilla").join("Firefox").join("Profiles"),
            });
            candidates.push(FirefoxCandidate {
                browser_id: "librewolf",
                browser_name: "LibreWolf",
                profiles_root: roaming.join("LibreWolf").join("Profiles"),
            });
            candidates.push(FirefoxCandidate {
                browser_id: "floorp",
                browser_name: "Floorp",
                profiles_root: roaming.join("Floorp").join("Profiles"),
            });
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_support = home_dir().join("Library").join("Application Support");
        candidates.push(FirefoxCandidate {
            browser_id: "firefox",
            browser_name: "Firefox",
            profiles_root: app_support.join("Firefox").join("Profiles"),
        });
        candidates.push(FirefoxCandidate {
            browser_id: "librewolf",
            browser_name: "LibreWolf",
            profiles_root: app_support.join("LibreWolf").join("Profiles"),
        });
        candidates.push(FirefoxCandidate {
            browser_id: "floorp",
            browser_name: "Floorp",
            profiles_root: app_support.join("Floorp").join("Profiles"),
        });
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        candidates.push(FirefoxCandidate {
            browser_id: "firefox",
            browser_name: "Firefox",
            profiles_root: home_dir().join(".mozilla").join("firefox"),
        });
        candidates.push(FirefoxCandidate {
            browser_id: "librewolf",
            browser_name: "LibreWolf",
            profiles_root: home_dir().join(".librewolf"),
        });
        candidates.push(FirefoxCandidate {
            browser_id: "floorp",
            browser_name: "Floorp",
            profiles_root: home_dir().join(".floorp"),
        });
    }

    candidates
}

fn scan_chromium_browser(candidate: ChromiumCandidate) -> anyhow::Result<Vec<BrowserProfile>> {
    if !candidate.root.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(&candidate.root)
        .with_context(|| format!("reading browser data directory {}", candidate.root.display()))?
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !is_chromium_profile_dir(file_name) {
            continue;
        }

        let extensions_path = path.join("Extensions");
        let preferences_path = path.join("Preferences");
        if !extensions_path.exists() && !preferences_path.exists() {
            continue;
        }

        let profile_name = chromium_profile_name(file_name, &preferences_path).unwrap_or_else(|| file_name.to_string());
        let extensions = if extensions_path.exists() {
            scan_chromium_extensions(&extensions_path)
        } else {
            Vec::new()
        };

        let mut notes = Vec::new();
        if extensions.is_empty() {
            notes.push("No unpacked Chromium extensions were found in this browser profile.".to_string());
        } else {
            notes.push("Chromium-family extensions can be mounted directly into the embedded preview.".to_string());
        }

        profiles.push(BrowserProfile {
            browser_id: candidate.browser_id.to_string(),
            browser_name: candidate.browser_name.to_string(),
            profile_name,
            engine: BrowserEngine::Chromium,
            profile_path: path,
            extensions_path: extensions_path.exists().then_some(extensions_path),
            preview_support: PreviewExtensionSupport::NativeImport,
            extensions,
            notes,
        });
    }

    Ok(profiles)
}

fn scan_firefox_browser(candidate: FirefoxCandidate) -> anyhow::Result<Vec<BrowserProfile>> {
    if !candidate.profiles_root.exists() {
        return Ok(Vec::new());
    }

    let mut profiles = Vec::new();
    for entry in fs::read_dir(&candidate.profiles_root).with_context(|| {
        format!(
            "reading Firefox-family profile directory {}",
            candidate.profiles_root.display()
        )
    })? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let extensions_json = path.join("extensions.json");
        if !extensions_json.exists() {
            continue;
        }

        let profile_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(firefox_profile_name)
            .unwrap_or_else(|| "Profile".to_string());
        let extensions = scan_firefox_extensions(&extensions_json, &path);

        profiles.push(BrowserProfile {
            browser_id: candidate.browser_id.to_string(),
            browser_name: candidate.browser_name.to_string(),
            profile_name,
            engine: BrowserEngine::FirefoxLike,
            profile_path: path,
            extensions_path: None,
            preview_support: PreviewExtensionSupport::DetectionOnly,
            extensions,
            notes: vec![
                "Firefox-family add-ons are detected for visibility, but the embedded preview currently mounts Chromium/WebView2-compatible extension trees only.".to_string(),
            ],
        });
    }

    Ok(profiles)
}

fn is_chromium_profile_dir(name: &str) -> bool {
    name == "Default"
        || name.starts_with("Profile ")
        || name.starts_with("Person ")
        || name == "Guest Profile"
}

fn chromium_profile_name(dir_name: &str, preferences_path: &Path) -> Option<String> {
    let raw = fs::read_to_string(preferences_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    json.get("profile")
        .and_then(|profile| profile.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string())
        .or_else(|| Some(dir_name.to_string()))
}

fn firefox_profile_name(raw_name: &str) -> String {
    raw_name
        .split_once('.')
        .map(|(_, suffix)| suffix.replace('-', " "))
        .unwrap_or_else(|| raw_name.replace('-', " "))
}

fn scan_chromium_extensions(extensions_root: &Path) -> Vec<DetectedExtension> {
    let mut extensions = Vec::new();

    let Ok(entries) = fs::read_dir(extensions_root) else {
        return extensions;
    };

    for entry in entries.flatten() {
        let extension_dir = entry.path();
        if !extension_dir.is_dir() {
            continue;
        }

        let extension_id = entry.file_name().to_string_lossy().to_string();
        let Some(version_dir) = newest_extension_version_dir(&extension_dir) else {
            continue;
        };
        let Some(extension) = read_chromium_extension_manifest(&extension_id, &version_dir) else {
            continue;
        };
        extensions.push(extension);
    }

    extensions.sort_by(|left, right| left.name.cmp(&right.name).then_with(|| left.id.cmp(&right.id)));
    extensions
}

fn newest_extension_version_dir(extension_dir: &Path) -> Option<PathBuf> {
    let mut versions = fs::read_dir(extension_dir)
        .ok()?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            path.is_dir().then_some(path)
        })
        .collect::<Vec<_>>();
    versions.sort();
    versions.pop()
}

#[derive(Default, Deserialize)]
struct ChromiumManifest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    description: String,
    default_locale: Option<String>,
}

fn read_chromium_extension_manifest(extension_id: &str, version_dir: &Path) -> Option<DetectedExtension> {
    let manifest_path = version_dir.join("manifest.json");
    let raw_manifest = fs::read_to_string(&manifest_path).ok()?;
    let manifest: ChromiumManifest = serde_json::from_str(&raw_manifest).ok()?;

    let name = resolve_localized_manifest_value(&manifest.name, manifest.default_locale.as_deref(), version_dir)
        .unwrap_or_else(|| extension_id.to_string());
    let description = resolve_localized_manifest_value(
        &manifest.description,
        manifest.default_locale.as_deref(),
        version_dir,
    );

    Some(DetectedExtension {
        id: extension_id.to_string(),
        name,
        version: if manifest.version.is_empty() {
            version_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string()
        } else {
            manifest.version
        },
        description,
        path: version_dir.to_path_buf(),
        enabled: true,
    })
}

fn resolve_localized_manifest_value(raw: &str, default_locale: Option<&str>, version_dir: &Path) -> Option<String> {
    if raw.is_empty() {
        return None;
    }
    let Some(key) = raw
        .strip_prefix("__MSG_")
        .and_then(|value| value.strip_suffix("__"))
    else {
        return Some(raw.to_string());
    };

    let mut locales = Vec::new();
    if let Some(default_locale) = default_locale {
        locales.push(default_locale.to_string());
    }
    locales.extend(["en", "en_US", "en_GB"].into_iter().map(ToString::to_string));

    for locale in locales {
        let messages_path = version_dir.join("_locales").join(locale).join("messages.json");
        let Ok(raw_messages) = fs::read_to_string(messages_path) else {
            continue;
        };
        let Ok(messages) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw_messages) else {
            continue;
        };
        if let Some(message) = messages
            .get(key)
            .and_then(|entry| entry.get("message"))
            .and_then(|message| message.as_str())
        {
            return Some(message.to_string());
        }
    }

    Some(key.replace('_', " "))
}

#[derive(Default, Deserialize)]
struct FirefoxExtensionsFile {
    #[serde(default)]
    addons: Vec<FirefoxAddon>,
}

#[derive(Default, Deserialize)]
struct FirefoxAddon {
    #[serde(default)]
    id: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    version: String,
    #[serde(default, rename = "defaultLocale")]
    default_locale: FirefoxLocale,
    #[serde(default)]
    active: bool,
    #[serde(default, rename = "userDisabled")]
    user_disabled: bool,
}

#[derive(Default, Deserialize)]
struct FirefoxLocale {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
}

fn scan_firefox_extensions(extensions_json: &Path, profile_path: &Path) -> Vec<DetectedExtension> {
    let Ok(raw) = fs::read_to_string(extensions_json) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<FirefoxExtensionsFile>(&raw) else {
        return Vec::new();
    };

    let mut extensions = parsed
        .addons
        .into_iter()
        .filter(|addon| addon.active && !addon.user_disabled)
        .map(|addon| DetectedExtension {
            id: addon.id.clone(),
            name: if addon.default_locale.name.is_empty() {
                addon.id.clone()
            } else {
                addon.default_locale.name
            },
            version: if addon.version.is_empty() {
                "unknown".to_string()
            } else {
                addon.version
            },
            description: (!addon.default_locale.description.is_empty()).then_some(addon.default_locale.description),
            path: firefox_addon_path(profile_path, &addon.path),
            enabled: addon.active,
        })
        .collect::<Vec<_>>();

    extensions.sort_by(|left, right| left.name.cmp(&right.name).then_with(|| left.id.cmp(&right.id)));
    extensions
}

fn firefox_addon_path(profile_path: &Path, addon_path: &str) -> PathBuf {
    let path = PathBuf::from(addon_path);
    if path.is_absolute() {
        path
    } else {
        profile_path.join(path)
    }
}

pub fn sanitize_component(value: &str) -> String {
    let mut sanitized = value
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect::<String>();
    while sanitized.contains("--") {
        sanitized = sanitized.replace("--", "-");
    }
    sanitized.trim_matches('-').to_ascii_lowercase()
}
