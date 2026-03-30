use anyhow::Result;
use gpui::{App, Entity, Task};
use project::{Project, ProjectEntryId, ProjectPath};
use workspace::ProjectItem as WorkspaceProjectItem;

use crate::registry::{PreviewKind, preview_handler_for_path};

pub struct PreviewAssetItem {
    pub project_path: ProjectPath,
    pub entry_id: Option<ProjectEntryId>,
    pub abs_path: Option<std::path::PathBuf>,
    pub file_name: String,
    pub kind: PreviewKind,
}

impl PreviewAssetItem {
    pub fn title(&self) -> String {
        format!("{} Preview", self.file_name)
    }
}

impl project::ProjectItem for PreviewAssetItem {
    fn try_open(
        project: &Entity<Project>,
        path: &ProjectPath,
        cx: &mut App,
    ) -> Option<Task<Result<Entity<Self>>>> {
        let handler = preview_handler_for_path(path.path.as_std_path())?;
        let entry_id = project.read(cx).entry_for_path(path, cx).map(|entry| entry.id);
        let abs_path = project.read(cx).absolute_path(path, cx);
        let file_name = path
            .path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.path.as_std_path().display().to_string());
        let project_path = path.clone();

        Some(Task::ready(Ok(cx.new(|_| Self {
            project_path,
            entry_id,
            abs_path,
            file_name,
            kind: handler.kind,
        }))))
    }

    fn entry_id(&self, _cx: &App) -> Option<ProjectEntryId> {
        self.entry_id
    }

    fn project_path(&self, _cx: &App) -> Option<ProjectPath> {
        Some(self.project_path.clone())
    }

    fn is_dirty(&self) -> bool {
        false
    }
}

impl WorkspaceProjectItem for crate::preview_view::UniversalPreviewView {
    type Item = PreviewAssetItem;

    fn project_item_kind() -> Option<workspace::item::ProjectItemKind> {
        Some(workspace::item::ProjectItemKind("RichFilePreview"))
    }

    fn for_project_item(
        project: Entity<Project>,
        _pane: Option<&workspace::Pane>,
        item: Entity<Self::Item>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> Self
    where
        Self: Sized,
    {
        Self::new(project, item, window, cx)
    }
}

