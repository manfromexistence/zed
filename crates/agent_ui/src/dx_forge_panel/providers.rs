use gpui::{AnyElement, App, SharedString, px};
use ui::{IconName, Tooltip, prelude::*};

use super::snapshot::DxForgePanelSnapshot;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProviderGroup {
    Code,
    Storage,
    Media,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProviderStatus {
    Ready,
    Waiting,
    Unavailable,
}

struct ForgeProvider {
    id: &'static str,
    label: &'static str,
    icon: IconName,
    group: ProviderGroup,
    source_pack: &'static str,
}

const PROVIDERS: &[ForgeProvider] = &[
    ForgeProvider {
        id: "github",
        label: "GitHub",
        icon: IconName::DxForgeProviderGithub,
        group: ProviderGroup::Code,
        source_pack: "svgl",
    },
    ForgeProvider {
        id: "gitlab",
        label: "GitLab",
        icon: IconName::DxForgeProviderGitlab,
        group: ProviderGroup::Code,
        source_pack: "svgl",
    },
    ForgeProvider {
        id: "bitbucket",
        label: "Bitbucket",
        icon: IconName::DxForgeProviderBitbucket,
        group: ProviderGroup::Code,
        source_pack: "material-icon-theme",
    },
    ForgeProvider {
        id: "drive",
        label: "Google Drive",
        icon: IconName::DxForgeProviderDrive,
        group: ProviderGroup::Storage,
        source_pack: "svgl",
    },
    ForgeProvider {
        id: "dropbox",
        label: "Dropbox",
        icon: IconName::DxForgeProviderDropbox,
        group: ProviderGroup::Storage,
        source_pack: "svgl",
    },
    ForgeProvider {
        id: "youtube",
        label: "YouTube",
        icon: IconName::DxForgeProviderYoutube,
        group: ProviderGroup::Media,
        source_pack: "svgl",
    },
    ForgeProvider {
        id: "soundcloud",
        label: "SoundCloud",
        icon: IconName::DxForgeProviderSoundcloud,
        group: ProviderGroup::Media,
        source_pack: "svgl",
    },
];

pub(super) fn remote_target_strip(snapshot: &DxForgePanelSnapshot, cx: &App) -> AnyElement {
    let ready_count = PROVIDERS
        .iter()
        .filter(|provider| provider_status(provider, snapshot) == ProviderStatus::Ready)
        .count();

    v_flex()
        .id("dx-forge-remote-targets")
        .w_full()
        .min_w_0()
        .gap_1()
        .px_2()
        .py_1()
        .border_b_1()
        .border_color(cx.theme().colors().border)
        .child(
            h_flex()
                .h(px(22.0))
                .w_full()
                .min_w_0()
                .gap_1()
                .child(Icon::new(IconName::CloudDownload).size(IconSize::XSmall))
                .child(
                    Label::new("Remote targets")
                        .size(LabelSize::Small)
                        .truncate(),
                )
                .child(div().flex_1())
                .child(
                    Label::new(format!("{ready_count}/{} ready", PROVIDERS.len()))
                        .size(LabelSize::XSmall)
                        .color(Color::Muted),
                ),
        )
        .child(provider_group_row(ProviderGroup::Code, snapshot, cx))
        .child(provider_group_row(ProviderGroup::Storage, snapshot, cx))
        .child(provider_group_row(ProviderGroup::Media, snapshot, cx))
        .into_any_element()
}

fn provider_group_row(
    group: ProviderGroup,
    snapshot: &DxForgePanelSnapshot,
    cx: &App,
) -> AnyElement {
    let mut row = h_flex()
        .id(SharedString::from(format!(
            "dx-forge-provider-group-{}",
            group.key()
        )))
        .w_full()
        .min_w_0()
        .gap_1()
        .flex_wrap()
        .child(
            h_flex()
                .h(px(26.0))
                .w(px(56.0))
                .flex_none()
                .gap_1()
                .child(
                    Icon::new(group.icon())
                        .size(IconSize::XSmall)
                        .color(Color::Muted),
                )
                .child(
                    Label::new(group.label())
                        .size(LabelSize::XSmall)
                        .color(Color::Muted)
                        .truncate(),
                ),
        );

    for provider in PROVIDERS.iter().filter(|provider| provider.group == group) {
        row = row.child(provider_target(provider, snapshot, cx));
    }

    row.into_any_element()
}

fn provider_target(
    provider: &'static ForgeProvider,
    snapshot: &DxForgePanelSnapshot,
    cx: &App,
) -> AnyElement {
    let status = provider_status(provider, snapshot);
    let tooltip_title =
        SharedString::from(format!("{} {}", provider.label, provider.group.label()));
    let tooltip_meta = provider_tooltip(provider, status);

    h_flex()
        .id(SharedString::from(format!(
            "dx-forge-provider-{}",
            provider.id
        )))
        .h(px(26.0))
        .min_w_0()
        .gap_1()
        .px_1p5()
        .border_1()
        .border_r_2()
        .border_color(cx.theme().colors().border.opacity(0.72))
        .bg(cx.theme().colors().ghost_element_background)
        .hover(|style| style.bg(cx.theme().colors().ghost_element_hover))
        .child(Icon::new(provider.icon).size(IconSize::Small))
        .child(
            Label::new(provider.label)
                .size(LabelSize::XSmall)
                .truncate(),
        )
        .child(div().w(px(3.0)).flex_1())
        .child(
            Label::new(provider_status_label(status))
                .size(LabelSize::XSmall)
                .color(provider_status_color(status)),
        )
        .child(
            div()
                .size(px(5.0))
                .rounded_full()
                .bg(provider_status_color(status).color(cx)),
        )
        .tooltip(move |_, cx| {
            Tooltip::with_meta(tooltip_title.clone(), None, tooltip_meta.clone(), cx)
        })
        .into_any_element()
}

fn provider_status(provider: &ForgeProvider, snapshot: &DxForgePanelSnapshot) -> ProviderStatus {
    if snapshot.workspace_roots.is_empty() {
        return ProviderStatus::Unavailable;
    }

    match provider.group {
        ProviderGroup::Code => {
            if snapshot.history_root_exists {
                ProviderStatus::Ready
            } else {
                ProviderStatus::Waiting
            }
        }
        ProviderGroup::Storage => {
            if !snapshot.restore_previews.is_empty() {
                ProviderStatus::Ready
            } else if snapshot.history_root_exists {
                ProviderStatus::Waiting
            } else {
                ProviderStatus::Unavailable
            }
        }
        ProviderGroup::Media => {
            if !snapshot.media_outputs.is_empty() {
                ProviderStatus::Ready
            } else if snapshot.history_root_exists {
                ProviderStatus::Waiting
            } else {
                ProviderStatus::Unavailable
            }
        }
    }
}

fn provider_status_label(status: ProviderStatus) -> &'static str {
    match status {
        ProviderStatus::Ready => "ready",
        ProviderStatus::Waiting => "waiting",
        ProviderStatus::Unavailable => "offline",
    }
}

fn provider_status_color(status: ProviderStatus) -> Color {
    match status {
        ProviderStatus::Ready => Color::Success,
        ProviderStatus::Waiting => Color::Warning,
        ProviderStatus::Unavailable => Color::Muted,
    }
}

fn provider_tooltip(provider: &ForgeProvider, status: ProviderStatus) -> String {
    format!(
        "{} remote target\nStatus: {}\nDX icon pack: {}",
        provider.group.label(),
        provider_status_label(status),
        provider.source_pack,
    )
}

impl ProviderGroup {
    fn key(self) -> &'static str {
        match self {
            ProviderGroup::Code => "code",
            ProviderGroup::Storage => "storage",
            ProviderGroup::Media => "media",
        }
    }

    fn label(self) -> &'static str {
        match self {
            ProviderGroup::Code => "Code",
            ProviderGroup::Storage => "Storage",
            ProviderGroup::Media => "Media",
        }
    }

    fn icon(self) -> IconName {
        match self {
            ProviderGroup::Code => IconName::GitBranch,
            ProviderGroup::Storage => IconName::CloudDownload,
            ProviderGroup::Media => IconName::PlayOutlined,
        }
    }
}
