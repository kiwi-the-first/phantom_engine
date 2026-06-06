use glam::Vec2;

use crate::ecs::{
    World,
    components::{Anchor, Camera, Transform},
};

/// Reposition and rescale every `Anchor` entity so it sticks to the camera's
/// visible rectangle at a constant on-screen size — faking screen-space UI through
/// world-space sprites. Run each frame, after scripts, just before rendering, with
/// the same `viewport_size` the renderer uses.
///
/// Overwrites `Transform.position.xy` and `Transform.scale.xy` for anchored
/// entities (z is left alone for layering).
pub fn update_anchors(world: &mut World, viewport_size: Vec2) {
    // The active camera defines the visible rect we anchor against.
    let Some(camera_entity) = world.query_with2::<Camera, Transform>().first().copied() else {
        return;
    };
    let (camera_pos, zoom, ref_res) = {
        let camera = world.get_component::<Camera>(camera_entity).unwrap();
        let transform = world.get_component::<Transform>(camera_entity).unwrap();
        (
            transform.position.truncate(),
            camera.zoom,
            camera.reference_resolution.as_vec2(),
        )
    };

    // Same fit-to-reference scale the renderer (and mouse_pos) computes.
    let scale = (viewport_size.x / ref_res.x * zoom).min(viewport_size.y / ref_res.y * zoom);
    if !scale.is_finite() || scale <= 0.0 {
        return;
    }
    let half_width = (viewport_size.x / scale) / 2.0;
    let half_height = (viewport_size.y / scale) / 2.0;

    for entity in world.query_with2::<Anchor, Transform>() {
        let (anchor, offset, base_scale) = {
            let a = world.get_component::<Anchor>(entity).unwrap();
            (a.anchor, a.offset, a.base_scale)
        };
        let transform = world.get_component_mut::<Transform>(entity).unwrap();
        // Anchor to a corner/edge, then nudge by a pixel offset (÷scale → world).
        transform.position.x = camera_pos.x + anchor.x * half_width + offset.x / scale;
        transform.position.y = camera_pos.y + anchor.y * half_height + offset.y / scale;
        // Counter the camera scale so on-screen pixel size stays constant.
        transform.scale.x = base_scale.x / scale;
        transform.scale.y = base_scale.y / scale;
    }
}
