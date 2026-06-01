use std::sync::{Arc, Mutex};

use egui::{Id, TextureId, Ui, Vec2};
use phantom_core::{
    ecs::{
        Entity,
        components::{Transform, camera::Camera},
    },
    input::input_context::ViewportInfo,
};
use phantom_runtime::renderer::state::State;

use crate::{context::EditorContext, resources::ResourceKey};

pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let Some(id) = ui
            .ctx()
            .data(|d| d.get_temp::<TextureId>(Id::new(ResourceKey::ViewportTexture)))
        else {
            return;
        };
        let Some(ectx) = ui.data_mut(|w| {
            w.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(ResourceKey::EditorContext))
        }) else {
            return;
        };

        let mut ectx_lock = ectx.lock().unwrap();
        let (camera_pos, zoom, ref_resolution) = {
            let cameras = ectx_lock.active_world.query_with::<Camera>();
            cameras
                .first()
                .and_then(|e| {
                    let cam = ectx_lock.active_world.get_component::<Camera>(*e)?;
                    let transform = ectx_lock.active_world.get_component::<Transform>(*e)?;
                    Some((
                        glam::Vec2::new(transform.position.x, transform.position.y),
                        cam.zoom,
                        cam.reference_resolution,
                    ))
                })
                .unwrap_or((glam::Vec2::ZERO, 1.0, glam::UVec2::ZERO))
        };
        drop(ectx_lock);

        let size = ui.available_size();
        let viewport_info = ViewportInfo {
            size: glam::Vec2::new(size.x, size.y),
            offset: glam::Vec2::new(ui.min_rect().min.x, ui.min_rect().min.y),
            camera_pos,
            zoom: zoom,
            reference_resolution: ref_resolution.as_vec2(),
        };

        ui.image(egui::load::SizedTexture::new(id, size));
        ui.data_mut(|w| {
            w.insert_temp(Id::new(ResourceKey::ViewportInfo), viewport_info);
        });
    }
}
