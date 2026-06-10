use ui::{DxUiIcon, IconName, dx_icon};

pub(crate) fn workflow_node_icon_for(
    icon_hint: Option<&str>,
    category_hint: Option<&str>,
    display_name: &str,
) -> IconName {
    let hints = [
        icon_hint.unwrap_or_default(),
        category_hint.unwrap_or_default(),
        display_name,
    ]
    .join(" ")
    .to_ascii_lowercase();

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
