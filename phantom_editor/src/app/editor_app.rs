use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use egui_wgpu::wgpu;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use log::*;

use phantom_runtime::renderer::state::State;

use crate::egui::egui_renderer::EguiRenderer;

pub struct EditorApp {
    state: Option<State>,
    egui_renderer: Option<EguiRenderer>,
    scale_factor: f32,
}

impl ApplicationHandler<State> for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let state = pollster::block_on(State::new(window.clone())).unwrap();

        let egui_renderer =
            EguiRenderer::new(&window, &state.device, state.surface_format(), None, 1);

        self.state = Some(state);
        self.egui_renderer = Some(egui_renderer);
    }
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                //self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            return;
        }
        self.handle_redraw();
    }
}

impl EditorApp {
    pub fn new() -> Self {
        Self {
            state: None,
            egui_renderer: None,
            scale_factor: 1.0,
        }
    }

    pub fn run() -> anyhow::Result<()> {
        env_logger::init();

        let event_loop = EventLoop::with_user_event().build()?;

        let mut app = EditorApp::new();
        event_loop.run_app(&mut app)?;

        Ok(())
    }

    fn handle_redraw(&mut self) {
        let state = self.state.as_mut().unwrap();

        // Attempt to handle minimizing window

        if let Some(min) = state.window.is_minimized() {
            if min {
                println!("Window is minimized");
                return;
            }
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.config.width, state.config.height],
            pixels_per_point: state.window.as_ref().scale_factor() as f32 * self.scale_factor,
        };

        let output = match state.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture, // Use it but should reconfigure
            _ => {
                log::warn!("Surface error, skipping frame");
                return;
            }
        };

        let surface_view = output.texture.create_view(&Default::default());

        let mut encoder = state.device.create_command_encoder(&Default::default());

        let window = state.window.as_ref();

        {
            let egui_renderer = self.egui_renderer.as_mut().unwrap();

            egui_renderer.begin_frame(window);

            egui::Window::new("winit + egui + wgpu says hello!")
                .resizable(true)
                .vscroll(true)
                .default_open(false)
                .show(egui_renderer.context(), |ui| {
                    ui.label("Label!");

                    if ui.button("Button!").clicked() {
                        println!("boom!")
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "Pixels per point: {}",
                            egui_renderer.context().pixels_per_point()
                        ));
                        if ui.button("-").clicked() {
                            self.scale_factor = (self.scale_factor - 0.1).max(0.3);
                        }
                        if ui.button("+").clicked() {
                            self.scale_factor = (self.scale_factor + 0.1).min(3.0);
                        }
                    });
                });

            self.egui_renderer.as_mut().unwrap().end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );
        }

        state.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
