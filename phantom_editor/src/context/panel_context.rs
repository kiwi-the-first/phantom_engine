use crate::panels::{ViewportState, asset_browser::AssetBrowserState};

pub struct PanelContext {
    pub asset_browser: AssetBrowserState,
    pub viewport: ViewportState,
}
