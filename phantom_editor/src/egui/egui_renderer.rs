use egui::Context;
use egui_wgpu::{
    Renderer, RendererOptions, ScreenDescriptor,
    wgpu::{
        self, CommandEncoder, Device, Queue, RenderPassDescriptor, StoreOp, TextureFormat,
        TextureView,
    },
};
use egui_winit::State;
use winit::{event::WindowEvent, window::Window};

pub struct EguiRenderer {
    egui_state: State,
    pub wgpu_renderer: Renderer,
    /// Output of the pass produced by `run`, consumed by `end_frame_and_draw`.
    full_output: Option<egui::FullOutput>,
    /// Events to inject at the start of the next egui frame.
    /// Used to synthesize things like PointerGone when the OS swallows
    /// a mouse-release (e.g. during drag-and-drop from an external app).
    pending_events: Vec<egui::Event>,
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

        egui_extras::install_image_loaders(&egui_context);

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
            wgpu_renderer: egui_renderer,
            full_output: None,
            pending_events: Vec::new(),
        }
    }

    pub fn context(&self) -> &Context {
        self.egui_state.egui_ctx()
    }

    pub fn handle_input(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) -> egui_winit::EventResponse {
        self.egui_state.on_window_event(window, event)
    }

    pub fn ppp(&mut self, v: f32) {
        self.context().set_pixels_per_point(v);
    }

    /// Queue an egui event to be injected at the start of the next frame.
    /// Useful for synthesising events the OS sometimes swallows
    /// (e.g. pointer-gone during a drag-and-drop from an external app).
    pub fn inject_event(&mut self, event: egui::Event) {
        self.pending_events.push(event);
    }

    /// Run one egui pass, building the UI inside `build_ui`.
    ///
    /// Must go through `Context::run_ui` rather than the manual
    /// `begin_pass`/`end_pass` API: only `run_ui` invokes egui's Plugin
    /// pass callbacks, where `LabelSelectionState` ends text-selection
    /// drags on mouse release and `DragAndDrop` clears stale payloads.
    ///
    /// `build_ui` may be called more than once if egui requests a
    /// discard-and-repeat pass.
    pub fn run(&mut self, window: &Window, mut build_ui: impl FnMut(&Context)) {
        let mut raw_input = self.egui_state.take_egui_input(window);
        if !self.pending_events.is_empty() {
            // Prepend synthesised events so egui sees them before anything
            // accumulated since the last frame.
            let mut injected = std::mem::take(&mut self.pending_events);
            injected.extend(raw_input.events);
            raw_input.events = injected;
        }
        let full_output = self
            .egui_state
            .egui_ctx()
            .run_ui(raw_input, |ui| build_ui(ui.ctx()));
        self.full_output = Some(full_output);
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        let full_output = self
            .full_output
            .take()
            .expect("run must be called before end_frame_and_draw can be called!");

        self.ppp(screen_descriptor.pixels_per_point);

        self.egui_state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self.egui_state.egui_ctx().tessellate(
            full_output.shapes,
            self.egui_state.egui_ctx().pixels_per_point(),
        );
        for (id, image_delta) in &full_output.textures_delta.set {
            self.wgpu_renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.wgpu_renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
            multiview_mask: Default::default(),
        });

        self.wgpu_renderer
            .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.wgpu_renderer.free_texture(x)
        }
    }
}
