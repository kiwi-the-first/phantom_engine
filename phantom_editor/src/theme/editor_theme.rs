use egui::Color32;

pub struct EditorTheme {
    pub bg: Color32,
    pub accent: Color32,
    pub stroke: Color32,
}

impl Default for EditorTheme {
    fn default() -> Self {
        Self {
            bg: Color32::BLACK,
            accent: Color32::from_hex("#8959d5").unwrap(),
            stroke: Color32::WHITE,
        }
    }
}

impl EditorTheme {
    pub fn apply(&self, ctx: &egui::Context) {
        let v = egui::Visuals::dark();
        ctx.set_visuals(v);
    }
}
