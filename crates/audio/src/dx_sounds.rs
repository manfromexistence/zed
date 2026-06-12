use std::time::Duration;

use crate::Sound;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum DxSoundEvent {
    ActionConfirm,
    AttentionBeep,
    ChatDropMagic,
    DeleteSoft,
    DragWatchTick,
    HoverSoft,
    MagicFire,
    MagicHeal,
    MagicWand,
    MenuSnap,
    PanelClose,
    PanelOpen,
    PopConfirm,
    ScreenLaunch,
    SuccessChime,
    TypingKey,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DxSoundPolicy {
    EnabledByDefault,
    ExplicitOptIn,
}

impl DxSoundEvent {
    pub(crate) fn sound(self) -> Sound {
        match self {
            Self::ActionConfirm => Sound::DxActionConfirm,
            Self::AttentionBeep => Sound::DxAttentionBeep,
            Self::ChatDropMagic => Sound::DxChatDropMagic,
            Self::DeleteSoft => Sound::DxDeleteSoft,
            Self::DragWatchTick => Sound::DxDragWatchTick,
            Self::HoverSoft => Sound::DxHoverSoft,
            Self::MagicFire => Sound::DxMagicFire,
            Self::MagicHeal => Sound::DxMagicHeal,
            Self::MagicWand => Sound::DxMagicWand,
            Self::MenuSnap => Sound::DxMenuSnap,
            Self::PanelClose => Sound::DxPanelClose,
            Self::PanelOpen => Sound::DxPanelOpen,
            Self::PopConfirm => Sound::DxPopConfirm,
            Self::ScreenLaunch => Sound::DxScreenLaunch,
            Self::SuccessChime => Sound::DxSuccessChime,
            Self::TypingKey => Sound::DxTypingKey,
        }
    }

    pub(crate) fn policy(self) -> DxSoundPolicy {
        match self {
            Self::TypingKey | Self::DeleteSoft | Self::HoverSoft => DxSoundPolicy::ExplicitOptIn,
            _ => DxSoundPolicy::EnabledByDefault,
        }
    }

    pub(crate) fn gain(self) -> f32 {
        0.05
    }

    pub(crate) fn cooldown(self) -> Duration {
        match self {
            Self::TypingKey => Duration::from_millis(36),
            Self::HoverSoft => Duration::from_millis(250),
            Self::DragWatchTick => Duration::from_millis(120),
            Self::MenuSnap | Self::PopConfirm => Duration::from_millis(90),
            Self::PanelOpen | Self::PanelClose | Self::ScreenLaunch => Duration::from_millis(140),
            Self::DeleteSoft | Self::ActionConfirm => Duration::from_millis(160),
            Self::ChatDropMagic | Self::SuccessChime => Duration::from_millis(300),
            Self::AttentionBeep | Self::MagicFire | Self::MagicHeal | Self::MagicWand => {
                Duration::from_millis(360)
            }
        }
    }
}
