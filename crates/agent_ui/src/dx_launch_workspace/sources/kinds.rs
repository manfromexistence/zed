use ui::{DxUiIcon, IconName, dx_icon};

use crate::dx_source_sets::DxSourceKind;

pub(super) fn source_kind_icon(kind: DxSourceKind) -> IconName {
    match kind {
        DxSourceKind::WorkspaceRoot => IconName::Folder,
        DxSourceKind::MetasearchSourcePack => IconName::FileTextOutlined,
        DxSourceKind::ReducedContextReceipt => IconName::FileTextOutlined,
        DxSourceKind::MediaOutput => dx_icon(DxUiIcon::Media),
        DxSourceKind::ForgeRestorePreview => IconName::Archive,
        DxSourceKind::DxToolchainConfig => dx_icon(DxUiIcon::Settings),
    }
}
