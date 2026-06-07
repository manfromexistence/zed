use crate::IconName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DxUiIcon {
    Agent,
    Ai,
    Automations,
    Browser,
    Check,
    Commands,
    Extensions,
    Fonts,
    Forge,
    Icons,
    Loading,
    Media,
    Plugins,
    Project,
    Receipts,
    Search,
    Settings,
    Style,
    Ui,
    WebPreview,
}

pub fn dx_icon(icon: DxUiIcon) -> IconName {
    match icon {
        DxUiIcon::Agent => IconName::ZedAgent,
        DxUiIcon::Ai => IconName::ZedAssistant,
        DxUiIcon::Automations => IconName::ListTodo,
        DxUiIcon::Browser | DxUiIcon::WebPreview => IconName::ToolWeb,
        DxUiIcon::Check => IconName::Check,
        DxUiIcon::Commands => IconName::Terminal,
        DxUiIcon::Extensions => IconName::BoxOpen,
        DxUiIcon::Fonts => IconName::Font,
        DxUiIcon::Forge => IconName::Forgejo,
        DxUiIcon::Icons => IconName::SquareDot,
        DxUiIcon::Loading => IconName::DxLoader,
        DxUiIcon::Media => IconName::Image,
        DxUiIcon::Plugins | DxUiIcon::Ui => IconName::Blocks,
        DxUiIcon::Project => IconName::FileTree,
        DxUiIcon::Receipts => IconName::FileTextOutlined,
        DxUiIcon::Search => IconName::MagnifyingGlass,
        DxUiIcon::Settings => IconName::DxCog,
        DxUiIcon::Style => IconName::Sliders,
    }
}
