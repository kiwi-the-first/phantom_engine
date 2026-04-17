use egui::Context;
use egui_wgpu::{
    Renderer, RendererOptions,
    wgpu::{Device, TextureFormat},
};
use egui_winit::State;
use winit::window::Window;

pub struct EguiRenderer {
    egui_state: State,
    egui_renderer: Renderer,
    frame_started: bool,
}

impl EguiRenderer {
    pub fn new(
        window: &Window,
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
    ) -> Self {
        let egui_context = Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024), // default dimension is 2048)
        );

        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            RendererOptions {
                msaa_samples: msaa_samples,
                depth_stencil_format: output_depth_format,
                dithering: true,
                predictable_texture_filtering: true,
            },
        );
        Self {
            egui_state: egui_state,
            egui_renderer: egui_renderer,
            frame_started: false,
        }
    }

    pub fn context(&self) -> &Context {
        self.egui_state.egui_ctx()
    }
}
