use std::sync::Arc;

use anyhow::{Result, bail};
use collections::IndexMap;
use convert_case::{Case, Casing as _};
use fs::Fs;
use gpui::{App, SharedString};
use settings::{
    AgentProfileContent, ContextServerPresetContent, LanguageModelSelection, Settings as _,
    SettingsContent, update_settings_file,
};
use util::ResultExt as _;

use crate::{AgentProfileId, AgentSettings};

pub mod builtin_profiles {
    use super::AgentProfileId;

    pub const WRITE: &str = "write";
    pub const ASK: &str = "ask";
    pub const MEDIA: &str = "media";
    pub const SEARCH: &str = "search";
    pub const STUDY: &str = "study";
    pub const LEGACY_MINIMAL: &str = "minimal";
    pub const DX_PROFILE_ORDER: [&str; 5] = [ASK, WRITE, SEARCH, STUDY, MEDIA];

    pub fn is_builtin(profile_id: &AgentProfileId) -> bool {
        matches!(profile_id.as_str(), WRITE | ASK | MEDIA | SEARCH | STUDY)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxAiProfileKind {
    Ask,
    Agents,
    Search,
    Study,
    Media,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxAiProfileBackendState {
    Wired,
    EvidenceBacked,
    ReceiptBacked,
    ProviderPending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DxAiProfileMetadata {
    pub id: &'static str,
    pub kind: DxAiProfileKind,
    pub display_name: &'static str,
    pub summary: &'static str,
    pub backend_state: DxAiProfileBackendState,
    pub runtime_proof_backend_lane_id: Option<&'static str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentProfile {
    id: AgentProfileId,
}

pub type AvailableProfiles = IndexMap<AgentProfileId, SharedString>;

impl AgentProfile {
    pub fn new(id: AgentProfileId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &AgentProfileId {
        &self.id
    }

    pub fn normalize_id(profile_id: AgentProfileId, cx: &App) -> AgentProfileId {
        Self::normalize_id_from_profiles(profile_id, &AgentSettings::get_global(cx).profiles)
    }

    pub fn normalize_id_from_profiles(
        profile_id: AgentProfileId,
        profiles: &IndexMap<AgentProfileId, AgentProfileSettings>,
    ) -> AgentProfileId {
        let ask_profile = AgentProfileId(builtin_profiles::ASK.into());
        if profile_id.as_str() == builtin_profiles::LEGACY_MINIMAL
            && profiles.contains_key(&ask_profile)
        {
            return ask_profile;
        }

        if profiles.contains_key(&profile_id) {
            return profile_id;
        }

        let write_profile = AgentProfileId(builtin_profiles::WRITE.into());
        if profiles.contains_key(&write_profile) {
            return write_profile;
        }

        if profiles.contains_key(&ask_profile) {
            return ask_profile;
        }

        profile_id
    }

    pub fn display_name(profile_id: &AgentProfileId, name: &SharedString) -> SharedString {
        if let Some(metadata) = Self::dx_builtin_metadata_for_id(profile_id.as_str()) {
            metadata.display_name.into()
        } else {
            name.clone()
        }
    }

    pub fn dx_builtin_metadata(profile_id: &AgentProfileId) -> Option<DxAiProfileMetadata> {
        Self::dx_builtin_metadata_for_id(profile_id.as_str())
    }

    pub fn dx_builtin_metadata_for_id(profile_id: &str) -> Option<DxAiProfileMetadata> {
        match profile_id {
            builtin_profiles::ASK | builtin_profiles::LEGACY_MINIMAL => Some(DxAiProfileMetadata {
                id: builtin_profiles::ASK,
                kind: DxAiProfileKind::Ask,
                display_name: "Ask",
                summary: "Answers questions with lightweight local and web sources.",
                backend_state: DxAiProfileBackendState::Wired,
                runtime_proof_backend_lane_id: None,
            }),
            builtin_profiles::WRITE => Some(DxAiProfileMetadata {
                id: builtin_profiles::WRITE,
                kind: DxAiProfileKind::Agents,
                display_name: "Agents",
                summary: "Runs builder and worker flows for code, tools, goals, plans, and multitask work.",
                backend_state: DxAiProfileBackendState::Wired,
                runtime_proof_backend_lane_id: None,
            }),
            builtin_profiles::SEARCH => Some(DxAiProfileMetadata {
                id: builtin_profiles::SEARCH,
                kind: DxAiProfileKind::Search,
                display_name: "Search",
                summary: "Uses DX MetaSearch, source-pack evidence, and Web Preview inspection where available.",
                backend_state: DxAiProfileBackendState::EvidenceBacked,
                runtime_proof_backend_lane_id: Some("dx-metasearch-live-proof"),
            }),
            builtin_profiles::STUDY => Some(DxAiProfileMetadata {
                id: builtin_profiles::STUDY,
                kind: DxAiProfileKind::Study,
                display_name: "Study",
                summary: "Organizes attached sources, receipts, and study rails without inventing results.",
                backend_state: DxAiProfileBackendState::ReceiptBacked,
                runtime_proof_backend_lane_id: Some("study-source-workspace-execution"),
            }),
            builtin_profiles::MEDIA => Some(DxAiProfileMetadata {
                id: builtin_profiles::MEDIA,
                kind: DxAiProfileKind::Media,
                display_name: "Media",
                summary: "Plans and gates image, video, audio, music, 3D, and document providers while execution setup is pending.",
                backend_state: DxAiProfileBackendState::ProviderPending,
                runtime_proof_backend_lane_id: Some("media-provider-readiness-proof"),
            }),
            _ => None,
        }
    }

    pub fn builtin_sort_index(profile_id: &AgentProfileId) -> Option<usize> {
        builtin_profiles::DX_PROFILE_ORDER
            .iter()
            .position(|id| *id == profile_id.as_str())
    }

    /// Saves a new profile to the settings.
    pub fn create(
        name: String,
        base_profile_id: Option<AgentProfileId>,
        fs: Arc<dyn Fs>,
        cx: &App,
    ) -> Result<AgentProfileId> {
        let name = name.trim().to_string();
        if name.is_empty() {
            bail!("Profile name is required.");
        }

        let id = AgentProfileId(name.to_case(Case::Kebab).into());
        if id.as_str() == builtin_profiles::LEGACY_MINIMAL {
            bail!(
                "`{}` is reserved for legacy profile compatibility. Choose another profile name.",
                name
            );
        }

        let settings = AgentSettings::get_global(cx);
        if settings.profiles.contains_key(&id) {
            bail!("A profile named `{}` already exists.", name);
        }
        if settings
            .profiles
            .iter()
            .filter(|(profile_id, _)| profile_id.as_str() != builtin_profiles::LEGACY_MINIMAL)
            .any(|(profile_id, profile)| {
                Self::display_name(profile_id, &profile.name)
                    .as_ref()
                    .eq_ignore_ascii_case(&name)
            })
        {
            bail!("A profile named `{}` already exists.", name);
        }

        let base_profile = base_profile_id
            .map(|id| Self::normalize_id(id, cx))
            .and_then(|id| AgentSettings::get_global(cx).profiles.get(&id).cloned());

        // Copy toggles from the base profile so the new profile starts with familiar defaults.
        let tools = base_profile
            .as_ref()
            .map(|profile| profile.tools.clone())
            .unwrap_or_default();
        let enable_all_context_servers = base_profile
            .as_ref()
            .map(|profile| profile.enable_all_context_servers)
            .unwrap_or_default();
        let context_servers = base_profile
            .as_ref()
            .map(|profile| profile.context_servers.clone())
            .unwrap_or_default();
        // Preserve the base profile's model preference when cloning into a new profile.
        let default_model = base_profile
            .as_ref()
            .and_then(|profile| profile.default_model.clone());

        let profile_settings = AgentProfileSettings {
            name: name.into(),
            tools,
            enable_all_context_servers,
            context_servers,
            default_model,
        };

        update_settings_file(fs, cx, {
            let id = id.clone();
            move |settings, _cx| {
                profile_settings.save_to_settings(id, settings).log_err();
            }
        });

        Ok(id)
    }

    /// Returns a map of AgentProfileIds to their names
    pub fn available_profiles(cx: &App) -> AvailableProfiles {
        let mut profiles = AvailableProfiles::default();
        for (id, profile) in AgentSettings::get_global(cx).profiles.iter() {
            if id.as_str() == builtin_profiles::LEGACY_MINIMAL {
                continue;
            }
            profiles.insert(id.clone(), Self::display_name(id, &profile.name));
        }
        profiles
    }
}

/// A profile for the Zed Agent that controls its behavior.
#[derive(Debug, Clone)]
pub struct AgentProfileSettings {
    /// The name of the profile.
    pub name: SharedString,
    pub tools: IndexMap<Arc<str>, bool>,
    pub enable_all_context_servers: bool,
    pub context_servers: IndexMap<Arc<str>, ContextServerPreset>,
    /// Default language model to apply when this profile becomes active.
    pub default_model: Option<LanguageModelSelection>,
}

impl AgentProfileSettings {
    pub fn is_tool_enabled(&self, tool_name: &str) -> bool {
        self.tools.get(tool_name) == Some(&true)
    }

    pub fn is_context_server_tool_enabled(&self, server_id: &str, tool_name: &str) -> bool {
        self.context_servers
            .get(server_id)
            .and_then(|preset| preset.tools.get(tool_name).copied())
            .unwrap_or(self.enable_all_context_servers)
    }

    pub fn save_to_settings(
        &self,
        profile_id: AgentProfileId,
        content: &mut SettingsContent,
    ) -> Result<()> {
        let profiles = content
            .agent
            .get_or_insert_default()
            .profiles
            .get_or_insert_default();
        if profile_id.as_str() == builtin_profiles::LEGACY_MINIMAL {
            bail!(
                "'{profile_id}' is reserved for legacy profile compatibility and cannot be saved as a custom profile"
            );
        }

        if profiles.contains_key(&profile_id.0) {
            bail!("profile with ID '{profile_id}' already exists");
        }

        profiles.insert(
            profile_id.0,
            AgentProfileContent {
                name: self.name.clone().into(),
                tools: self.tools.clone(),
                enable_all_context_servers: Some(self.enable_all_context_servers),
                context_servers: self
                    .context_servers
                    .clone()
                    .into_iter()
                    .map(|(server_id, preset)| {
                        (
                            server_id,
                            ContextServerPresetContent {
                                tools: preset.tools,
                            },
                        )
                    })
                    .collect(),
                default_model: self.default_model.clone(),
            },
        );

        Ok(())
    }
}

impl From<AgentProfileContent> for AgentProfileSettings {
    fn from(content: AgentProfileContent) -> Self {
        let AgentProfileContent {
            name,
            tools,
            enable_all_context_servers,
            context_servers,
            default_model,
        } = content;

        Self {
            name: name.into(),
            tools,
            enable_all_context_servers: enable_all_context_servers.unwrap_or_default(),
            context_servers: context_servers
                .into_iter()
                .map(|(server_id, preset)| (server_id, preset.into()))
                .collect(),
            default_model,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContextServerPreset {
    pub tools: IndexMap<Arc<str>, bool>,
}

impl From<settings::ContextServerPresetContent> for ContextServerPreset {
    fn from(content: settings::ContextServerPresetContent) -> Self {
        Self {
            tools: content.tools,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(
        enable_all_context_servers: bool,
        context_servers: IndexMap<Arc<str>, ContextServerPreset>,
    ) -> AgentProfileSettings {
        AgentProfileSettings {
            name: "test".into(),
            tools: IndexMap::default(),
            enable_all_context_servers,
            context_servers,
            default_model: None,
        }
    }

    fn preset(tools: &[(&str, bool)]) -> ContextServerPreset {
        ContextServerPreset {
            tools: tools
                .iter()
                .map(|(name, enabled)| (Arc::from(*name), *enabled))
                .collect(),
        }
    }

    #[test]
    fn explicit_false_disables_tool_when_enable_all_is_true() {
        let mut servers = IndexMap::default();
        servers.insert(Arc::from("server"), preset(&[("disabled_tool", false)]));
        let profile = profile(true, servers);

        assert!(!profile.is_context_server_tool_enabled("server", "disabled_tool"));
        assert!(profile.is_context_server_tool_enabled("server", "other_tool"));
        assert!(profile.is_context_server_tool_enabled("other_server", "any_tool"));
    }

    #[test]
    fn explicit_true_enables_tool_when_enable_all_is_false() {
        let mut servers = IndexMap::default();
        servers.insert(Arc::from("server"), preset(&[("enabled_tool", true)]));
        let profile = profile(false, servers);

        assert!(profile.is_context_server_tool_enabled("server", "enabled_tool"));
        assert!(!profile.is_context_server_tool_enabled("server", "other_tool"));
        assert!(!profile.is_context_server_tool_enabled("other_server", "any_tool"));
    }
}
