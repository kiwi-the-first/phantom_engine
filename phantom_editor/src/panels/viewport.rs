use egui::{Id, TextureId, Ui, Vec2};

use crate::resources::ResourceKey;

pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(id) = ui
            .ctx()
            .data(|d| d.get_temp::<TextureId>(Id::new(ResourceKey::ViewportTexture)))
        {
            let size = ui.available_size();
            ui.image(egui::load::SizedTexture::new(id, size));
            ui.data_mut(|w| {
                w.insert_temp::<Vec2>(Id::new(ResourceKey::ViewportSize), size.clone())
            });
        }
    }
}
