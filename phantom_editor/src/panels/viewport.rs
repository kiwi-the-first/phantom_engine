use std::collections::HashMap;

use egui::{TextureId, Ui, Vec2};
use egui_wgpu::wgpu;
use phantom_core::{
    ecs::{
        World,
        components::{Transform, camera::Camera},
    },
    input::input_context::ViewportInfo,
};
use phantom_runtime::{
    asset_manager::asset_types::texture::Texture, renderer::scene_renderer::SceneRenderer,
};

use crate::context::EditorContext;
use crate::egui::egui_renderer::EguiRenderer;

/// The scene render target. Owns the offscreen texture, the scene renderer, and
/// the egui texture handle. App-owned and driven across the frame: `render_scene`
/// (before the egui frame), `apply_resize` (after it). The on-screen display is the
/// separate, stateless [`ViewportPanel`] stage.
pub struct Viewport {
    texture: wgpu::Texture,
    texture_id: TextureId,
    scene_renderer: SceneRenderer,
    /// Size of the current offscreen texture.
    size: Vec2,
    /// Size the panel last requested (its available area); applied in `apply_resize`.
    requested_size: Vec2,
    /// Latest viewport framing, consumed by the input system to map cursor → world.
    info: Option<ViewportInfo>,
}

impl Viewport {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        egui_renderer: &mut EguiRenderer,
        size: Vec2,
    ) -> Self {
        let texture = Self::create_texture(device, size);
        let mut scene_renderer =
            SceneRenderer::new(device, queue, wgpu::TextureFormat::Rgba8UnormSrgb);
        scene_renderer.resize(device, size.x as u32, size.y as u32);

        let view = texture.create_view(&Default::default());
        let texture_id = egui_renderer.wgpu_renderer.register_native_texture(
            device,
            &view,
            wgpu::FilterMode::Linear,
        );

        Self {
            texture,
            texture_id,
            scene_renderer,
            size,
            requested_size: size,
            info: None,
        }
    }

    fn create_texture(device: &wgpu::Device, size: Vec2) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("viewport"),
            size: wgpu::Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        })
    }

    /// Render the scene into the offscreen texture. Call before the egui frame.
    pub fn render_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
    ) {
        let view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });
        let size = glam::Vec2::new(self.size.x, self.size.y);
        if let Err(e) = self
            .scene_renderer
            .render(device, queue, encoder, &view, world, size)
        {
            log::error!("Scene render failed: {e}");
        }
    }

    /// Upload any new sprite textures to the GPU.
    pub fn upload_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        textures: &HashMap<String, Texture>,
    ) {
        self.scene_renderer.upload_textures(device, queue, textures);
    }

    /// Recreate the offscreen texture if the panel size changed. Call after the egui frame.
    pub fn apply_resize(&mut self, device: &wgpu::Device, egui_renderer: &mut EguiRenderer) {
        if self.size == self.requested_size
            || self.requested_size.x < 1.0
            || self.requested_size.y < 1.0
        {
            return;
        }
        log::trace!(
            "viewport size changed from {:?} to {:?}",
            self.size,
            self.requested_size
        );
        let new_size = self.requested_size;
        self.texture = Self::create_texture(device, new_size);
        let view = self.texture.create_view(&Default::default());
        egui_renderer
            .wgpu_renderer
            .update_egui_texture_from_wgpu_texture(
                device,
                &view,
                wgpu::FilterMode::Linear,
                self.texture_id,
            );
        self.scene_renderer
            .resize(device, new_size.x as u32, new_size.y as u32);
        self.size = new_size;
    }

    /// The egui handle for the offscreen texture, for the panel to display.
    pub fn texture_id(&self) -> TextureId {
        self.texture_id
    }

    /// Record the panel's requested size and current framing (set by the panel each frame).
    pub fn set_frame(&mut self, requested_size: Vec2, info: ViewportInfo) {
        self.requested_size = requested_size;
        self.info = Some(info);
    }

    /// Latest viewport framing, for the input system's cursor mapping.
    pub fn info(&self) -> Option<ViewportInfo> {
        self.info
    }
}

/// Stateless display stage: draws the [`Viewport`]'s texture and reports the panel's
/// size + camera framing back to it. Mirrors the other panels' `show(ui, ...)` shape.
pub struct ViewportPanel {}

impl ViewportPanel {
    pub fn show(ui: &mut Ui, ectx: &EditorContext, viewport: &mut Viewport) {
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
        viewport.set_frame(size, info);

        ui.image(egui::load::SizedTexture::new(viewport.texture_id(), size));
    }
}
