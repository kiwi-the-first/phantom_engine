use egui::Ui;

pub struct InspectorPanel {}

impl InspectorPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.label("Inspector will appear here");
        ui.separator();
    }
}
