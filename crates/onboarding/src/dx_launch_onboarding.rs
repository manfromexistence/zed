#[derive(Clone, Debug)]
pub struct DxLaunchPreviewTarget {
    pub url: String,
}

#[derive(Clone, Debug)]
pub struct DxLaunchPreviewTargets {
    pub primary: DxLaunchPreviewTarget,
}

impl DxLaunchPreviewTargets {
    pub fn local_web_preview_onboarding(url: String) -> Self {
        Self {
            primary: DxLaunchPreviewTarget { url },
        }
    }
}
