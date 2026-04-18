use egui::Ui;

pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.label("Viewport will appear here");
        ui.separator();
    }
}
