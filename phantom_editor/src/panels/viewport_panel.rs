use crate::context::{
    EditorContext,
    panel_context::{self, PanelContext},
};
use egui::Ui;
use phantom_core::{
    ecs::components::{Collider, Transform, camera::Camera},
    input::input_context::ViewportInfo,
};

/// Stateless display stage: draws the [`Viewport`]'s texture and reports the panel's
/// size + camera framing back to it. Mirrors the other panels' `show(ui, ...)` shape.
pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn show(ui: &mut Ui, ectx: &mut EditorContext, panel_context: &mut PanelContext) {
        egui::Panel::top("Viewport Tool Bar").show_inside(ui, |ui| {
            if ui.button("Show Gizmos").clicked() {
                ectx.show_colliders = !ectx.show_colliders;
            }
        });

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

        let image_rect = ui
            .image(egui::load::SizedTexture::new(viewport.texture_id(), size))
            .rect;

        if ectx.show_colliders {
            let viewport_w = size.x;
            let viewport_h = size.y;
            let ref_res = ref_resolution.as_vec2();
            let scale_x = viewport_w / ref_res.x * zoom;
            let scale_y = viewport_h / ref_res.y * zoom;
            let scale = scale_x.min(scale_y);
            let half_w = (viewport_w / scale) / 2.0;
            let half_h = (viewport_h / scale) / 2.0;
            let origin = image_rect.min;

            let world_to_screen = |wx: f32, wy: f32| -> egui::Pos2 {
                let ndc_x = (wx - camera_pos.x) / half_w;
                let ndc_y = (wy - camera_pos.y) / half_h;
                egui::pos2(
                    (ndc_x + 1.0) / 2.0 * viewport_w + origin.x,
                    (1.0 - ndc_y) / 2.0 * viewport_h + origin.y,
                )
            };

            let painter = ui.painter();
            let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(0, 255, 80));

            for entity in world.query_with2::<Collider, Transform>() {
                let Some(col) = world.get_component::<Collider>(entity) else {
                    continue;
                };
                let Some(transform) = world.get_component::<Transform>(entity) else {
                    continue;
                };
                let cx = transform.position.x + col.offset.x;
                let cy = transform.position.y + col.offset.y;
                let hw = col.width * transform.scale.x / 2.0;
                let hh = col.height * transform.scale.y / 2.0;

                let tl = world_to_screen(cx - hw, cy + hh);
                let tr = world_to_screen(cx + hw, cy + hh);
                let br = world_to_screen(cx + hw, cy - hh);
                let bl = world_to_screen(cx - hw, cy - hh);

                painter.line_segment([tl, tr], stroke);
                painter.line_segment([tr, br], stroke);
                painter.line_segment([br, bl], stroke);
                painter.line_segment([bl, tl], stroke);
            }
        }
    }
}
