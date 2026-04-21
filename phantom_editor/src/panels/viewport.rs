use egui::{Id, TextureId, Ui};

use crate::render_resoruces::render_recource_keys::RenderReourceKey;

pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(id) = ui
            .ctx()
            .data(|d| d.get_temp::<TextureId>(Id::new(RenderReourceKey::ViewportTexture)))
        {
            let size = ui.available_size();
            ui.image(egui::load::SizedTexture::new(id, size));
        }
    }
}
