use egui::Ui;

pub struct AssetBrowserPanel {}

impl AssetBrowserPanel {
    pub fn show(ui: &mut Ui) {
        ui.label("Asset Browser will appear here");
        ui.separator();
    }
}
