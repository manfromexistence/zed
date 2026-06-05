use ui::IconName;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ProviderGroup {
    Code,
    Storage,
    Media,
}

impl ProviderGroup {
    pub(super) const ALL: [ProviderGroup; 3] = [
        ProviderGroup::Code,
        ProviderGroup::Storage,
        ProviderGroup::Media,
    ];
}

pub(super) struct ForgeProvider {
    pub(super) id: &'static str,
    pub(super) label: &'static str,
    pub(super) icon: IconName,
    pub(super) group: ProviderGroup,
    pub(super) source_pack: &'static str,
    pub(super) source_slug: &'static str,
}

pub(super) const PROVIDERS: &[ForgeProvider] = &[
    ForgeProvider {
        id: "github",
        label: "GitHub",
        icon: IconName::DxForgeProviderGithub,
        group: ProviderGroup::Code,
        source_pack: "svgl",
        source_slug: "github_dark",
    },
    ForgeProvider {
        id: "gitlab",
        label: "GitLab",
        icon: IconName::DxForgeProviderGitlab,
        group: ProviderGroup::Code,
        source_pack: "svgl",
        source_slug: "gitlab",
    },
    ForgeProvider {
        id: "bitbucket",
        label: "Bitbucket",
        icon: IconName::DxForgeProviderBitbucket,
        group: ProviderGroup::Code,
        source_pack: "simple-icons",
        source_slug: "bitbucket",
    },
    ForgeProvider {
        id: "drive",
        label: "Google Drive",
        icon: IconName::DxForgeProviderDrive,
        group: ProviderGroup::Storage,
        source_pack: "svgl",
        source_slug: "drive",
    },
    ForgeProvider {
        id: "dropbox",
        label: "Dropbox",
        icon: IconName::DxForgeProviderDropbox,
        group: ProviderGroup::Storage,
        source_pack: "svgl",
        source_slug: "dropbox",
    },
    ForgeProvider {
        id: "youtube",
        label: "YouTube",
        icon: IconName::DxForgeProviderYoutube,
        group: ProviderGroup::Media,
        source_pack: "svgl",
        source_slug: "youtube",
    },
    ForgeProvider {
        id: "soundbox",
        label: "SoundBox",
        icon: IconName::DxForgeProviderSoundbox,
        group: ProviderGroup::Media,
        source_pack: "svgl",
        source_slug: "soundcloud-logo",
    },
    ForgeProvider {
        id: "soundcloud",
        label: "SoundCloud",
        icon: IconName::DxForgeProviderSoundcloud,
        group: ProviderGroup::Media,
        source_pack: "svgl",
        source_slug: "soundcloud-logo",
    },
];

impl ProviderGroup {
    pub(super) fn key(self) -> &'static str {
        match self {
            ProviderGroup::Code => "code",
            ProviderGroup::Storage => "storage",
            ProviderGroup::Media => "media",
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            ProviderGroup::Code => "Code targets",
            ProviderGroup::Storage => "Storage targets",
            ProviderGroup::Media => "Media targets",
        }
    }

    pub(super) fn provider_labels(self) -> String {
        providers_for(self)
            .map(|provider| provider.label)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub(super) fn providers_for(group: ProviderGroup) -> impl Iterator<Item = &'static ForgeProvider> {
    PROVIDERS
        .iter()
        .filter(move |provider| provider.group == group)
}
