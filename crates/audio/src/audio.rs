use std::time::Duration;

use rodio::{ChannelCount, SampleRate, nz};

pub const REPLAY_DURATION: Duration = Duration::from_secs(30);
pub const SAMPLE_RATE: SampleRate = nz!(48000);
pub const CHANNEL_COUNT: ChannelCount = nz!(2);

mod audio_settings;
pub use audio_settings::AudioSettings;
pub use audio_settings::LIVE_SETTINGS;

mod dx_sounds;
pub use dx_sounds::{DxSoundEvent, DxSoundPolicy};

mod audio_pipeline;
pub use audio_pipeline::{Audio, AudioPlaybackHandle};
pub use audio_pipeline::{AudioDeviceInfo, AvailableAudioDevices};
pub use audio_pipeline::{ensure_devices_initialized, resolve_device};
// TODO(audio) replace with input test functionality in the audio crate
pub use audio_pipeline::RodioExt;
pub use audio_pipeline::init;
pub use audio_pipeline::{open_input_stream, open_test_output};

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum Sound {
    Joined,
    GuestJoined,
    Leave,
    Mute,
    Unmute,
    StartScreenshare,
    StopScreenshare,
    AgentDone,
    DxActionConfirm,
    DxAttentionBeep,
    DxChatDropMagic,
    DxDeleteSoft,
    DxDragWatchTick,
    DxHoverSoft,
    DxMagicFire,
    DxMagicHeal,
    DxMagicWand,
    DxMenuSnap,
    DxPanelClose,
    DxPanelOpen,
    DxPopConfirm,
    DxScreenLaunch,
    DxSuccessChime,
    DxTypingKey,
}

impl Sound {
    fn file(&self) -> &'static str {
        match self {
            Self::Joined => "joined_call",
            Self::GuestJoined => "guest_joined_call",
            Self::Leave => "leave_call",
            Self::Mute => "mute",
            Self::Unmute => "unmute",
            Self::StartScreenshare => "start_screenshare",
            Self::StopScreenshare => "stop_screenshare",
            Self::AgentDone => "agent_done",
            Self::DxActionConfirm => "dx_action_confirm",
            Self::DxAttentionBeep => "dx_attention_beep",
            Self::DxChatDropMagic => "dx_chat_drop_magic",
            Self::DxDeleteSoft => "dx_delete_soft",
            Self::DxDragWatchTick => "dx_drag_watch_tick",
            Self::DxHoverSoft => "dx_hover_soft",
            Self::DxMagicFire => "dx_magic_fire",
            Self::DxMagicHeal => "dx_magic_heal",
            Self::DxMagicWand => "dx_magic_wand",
            Self::DxMenuSnap => "dx_menu_snap",
            Self::DxPanelClose => "dx_panel_close",
            Self::DxPanelOpen => "dx_panel_open",
            Self::DxPopConfirm => "dx_pop_confirm",
            Self::DxScreenLaunch => "dx_screen_launch",
            Self::DxSuccessChime => "dx_success_chime",
            Self::DxTypingKey => "dx_typing_key",
        }
    }
}
