use egui::Ui;

use crate::context::{EditorContext, panel_context::PanelContext};

pub struct AssetBrowserPanel {}

impl AssetBrowserPanel {
    pub fn show(ui: &mut Ui, ectx: &EditorContext, pctx: &mut PanelContext) {
        pctx.asset_browser.show(ui, ectx);
    }
}
