use egui::Ui;

pub struct AssetBrowserPanel {}

impl AssetBrowserPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.label("Asset Browser will appear here");
        ui.separator();
    }
}
