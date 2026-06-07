use gpui::{AnyElement, IntoElement};

use crate::traits::animation_ext::CommonAnimationExt;
use crate::{Color, Icon, IconName, IconSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DxUiIcon {
    Agent,
    Ai,
    Automations,
    Browser,
    Channels,
    Check,
    Commands,
    Computer,
    Connections,
    Credentials,
    Evidence,
    Extensions,
    Fonts,
    Forge,
    Gateway,
    Icons,
    Loading,
    Media,
    Mcp,
    Permissions,
    Plugins,
    Project,
    Receipts,
    Search,
    Settings,
    Source,
    Style,
    Storage,
    Ui,
    WebPreview,
}

pub fn dx_icon(icon: DxUiIcon) -> IconName {
    match icon {
        DxUiIcon::Agent => IconName::ZedAgent,
        DxUiIcon::Ai => IconName::ZedAssistant,
        DxUiIcon::Automations => IconName::ListTodo,
        DxUiIcon::Browser | DxUiIcon::WebPreview => IconName::ToolWeb,
        DxUiIcon::Channels => IconName::QueueMessage,
        DxUiIcon::Check => IconName::Check,
        DxUiIcon::Commands => IconName::Terminal,
        DxUiIcon::Computer => IconName::Screen,
        DxUiIcon::Connections => IconName::UserGroup,
        DxUiIcon::Credentials => IconName::LockOutlined,
        DxUiIcon::Evidence => IconName::Public,
        DxUiIcon::Extensions => IconName::BoxOpen,
        DxUiIcon::Fonts => IconName::Font,
        DxUiIcon::Forge => IconName::Forgejo,
        DxUiIcon::Gateway => IconName::Server,
        DxUiIcon::Icons => IconName::SquareDot,
        DxUiIcon::Loading => IconName::DxLoader,
        DxUiIcon::Media => IconName::Image,
        DxUiIcon::Mcp => IconName::Server,
        DxUiIcon::Permissions => IconName::UserCheck,
        DxUiIcon::Plugins | DxUiIcon::Ui => IconName::Blocks,
        DxUiIcon::Project => IconName::FileTree,
        DxUiIcon::Receipts => IconName::FileTextOutlined,
        DxUiIcon::Search => IconName::MagnifyingGlass,
        DxUiIcon::Settings => IconName::DxCog,
        DxUiIcon::Source => IconName::FolderSearch,
        DxUiIcon::Style => IconName::Sliders,
        DxUiIcon::Storage => IconName::DatabaseZap,
    }
}

pub fn dx_loading_icon(size: IconSize, color: Color, duration_secs: u64) -> AnyElement {
    Icon::new(dx_icon(DxUiIcon::Loading))
        .size(size)
        .color(color)
        .with_rotate_animation(duration_secs)
        .into_any_element()
}
