use crate::context::{
    EditorContext,
    panel_context::{self, PanelContext},
};
use egui::Ui;
use phantom_core::{
    ecs::components::{Transform, camera::Camera},
    input::input_context::ViewportInfo,
};

/// Stateless display stage: draws the [`Viewport`]'s texture and reports the panel's
/// size + camera framing back to it. Mirrors the other panels' `show(ui, ...)` shape.
pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn show(ui: &mut Ui, ectx: &EditorContext, panel_context: &mut PanelContext) {
        let world = &ectx.active_world;
        let (camera_pos, zoom, ref_resolution) = {
            let cameras = world.query_with::<Camera>();
            cameras
                .first()
                .and_then(|e| {
                    let cam = world.get_component::<Camera>(*e)?;
                    let transform = world.get_component::<Transform>(*e)?;
                    Some((
                        glam::Vec2::new(transform.position.x, transform.position.y),
                        cam.zoom,
                        cam.reference_resolution,
                    ))
                })
                .unwrap_or((glam::Vec2::ZERO, 1.0, glam::UVec2::ZERO))
        };

        let size = ui.available_size();
        let info = ViewportInfo {
            size: glam::Vec2::new(size.x, size.y),
            offset: glam::Vec2::new(ui.min_rect().min.x, ui.min_rect().min.y),
            camera_pos,
            zoom,
            reference_resolution: ref_resolution.as_vec2(),
        };
        let viewport = &mut panel_context.viewport;
        viewport.set_frame(size, info);

        ui.image(egui::load::SizedTexture::new(viewport.texture_id(), size));
    }
}
