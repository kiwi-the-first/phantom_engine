use std::sync::Arc;

use anyhow::Result;
use glam::Vec2;
use phantom_common::dirs::dirs::PlayerDirs;
use phantom_core::ecs::World;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use log::*;

use crate::asset_manager::asset_manager::AssetManager;
use crate::game_loader::game_loader::GameLoader;
use crate::renderer::scene_renderer::SceneRenderer;
use crate::renderer::state::State;

#[derive(Default)]
pub struct App {
    state: Option<State>,
    scene_renderer: Option<SceneRenderer>,
    world: Option<World>,
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(State::new(window.clone())).unwrap();
        self.state = Some(state);

        self.scene_renderer = Some(SceneRenderer::new(
            &self.state.as_ref().unwrap().device,
            &self.state.as_ref().unwrap().queue,
            self.state.as_ref().unwrap().surface_format(),
        ));

        match GameLoader::load_world() {
            Ok(world) => self.world = Some(world),
            Err(e) => log::error!("FAILED TO LOAD WORLD FILE! {e}"),
        }

        let mut asset_manager = AssetManager::new();
        if let Some(world) = self.world.as_ref() {
            asset_manager.load_sprite_assets(&world, &PlayerDirs::data());
        }

        let state = self.state.as_mut().unwrap();
        let scene_renderer = self.scene_renderer.as_mut().unwrap();
        scene_renderer.upload_textures(&state.device, &state.queue, &asset_manager.textures);
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let state = self.state.as_mut().unwrap();
        let scene_renderer = self.scene_renderer.as_mut().unwrap();

        let output = match state.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture, // Use it but should reconfigure
            _ => {
                log::warn!("Surface error, skipping frame");
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        if let Some(world) = &self.world {
            scene_renderer.render(
                &state.device,
                &state.queue,
                &mut encoder,
                &view,
                world,
                Vec2::new(state.config.width as f32, state.config.height as f32),
            );
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            scene_renderer: None,
            world: None,
        }
    }

    pub fn run() -> anyhow::Result<()> {
        env_logger::init();

        let event_loop = EventLoop::with_user_event().build()?;

        let mut app = App::new();
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}
