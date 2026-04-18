use egui::Ui;

pub struct HierarchyPanel {}

impl HierarchyPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.label("Hierarchy will appear here");
        ui.separator();
    }
}
