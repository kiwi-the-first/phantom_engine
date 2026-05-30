use std::sync::Arc;

use anyhow::Result;
use env_logger::init;
use glam::Vec2;
use libloading::Library;
use phantom_common::dirs::dirs::PlayerDirs;
use phantom_core::ecs::World;
use phantom_core::input::Input;
use phantom_core::input::input::ViewportInfo;
use phantom_core::scripting::ScriptContext;
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
    script_context: Option<ScriptContext>,
    game_dylib: Option<Library>,
    asset_manager: Option<AssetManager>,
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

        self.scene_renderer.as_mut().unwrap().resize(
            &self.state.as_ref().unwrap().device,
            window.inner_size().width as u32,
            window.inner_size().height as u32,
        );

        match GameLoader::load_dylib() {
            Ok(dylib) => {
                self.game_dylib = Some(dylib);
                log::info!("Game dylib loaded sucessfully!");
            }
            Err(e) => log::error!("FAILED TO LOAD GAME DYLIB! {e}"),
        }

        if let Some(dylib) = self.game_dylib.as_ref() {
            match GameLoader::init_dylib(&dylib) {
                Ok(_) => log::info!("All components from game are registered!"),
                Err(e) => log::error!("FAILED TO REGISTER COMPONENTS/SCRIPTS FROM GAME! {e}"),
            }
        }

        match GameLoader::load_world() {
            Ok(world) => self.world = Some(world),
            Err(e) => log::error!("FAILED TO LOAD WORLD FILE! {e}"),
        }

        let mut asset_manager = AssetManager::new();
        if let Some(world) = self.world.as_ref() {
            if let Err(e) = asset_manager.load_sprite_assets(&world, &PlayerDirs::data()) {
                log::error!("Failed to load sprite assets: {e}");
            }
        }

        self.script_context = Some(ScriptContext::default());

        let state = self.state.as_mut().unwrap();
        let scene_renderer = self.scene_renderer.as_mut().unwrap();
        scene_renderer.upload_textures(&state.device, &state.queue, &asset_manager.textures);
        self.asset_manager = Some(asset_manager);

        phantom_core::scripting::script_scheduler::ScriptScheduler::run_all_start_scripts(
            &mut self.world.as_mut().unwrap(),
            &self.script_context.as_mut().unwrap(),
        );
    }
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(script_ctx) = &mut self.script_context {
            let input_system = &mut script_ctx.input;
            input_system.handle_event(&event);
        }

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
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    let width = physical_size.width;
                    let height = physical_size.height;
                    state.resize(width, height);
                    self.scene_renderer
                        .as_mut()
                        .unwrap()
                        .resize(&state.device, width, height);
                }
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

        // Pick up textures for any sprites spawned by scripts since the last frame.
        if let (Some(world), Some(asset_manager)) =
            (self.world.as_ref(), self.asset_manager.as_mut())
        {
            if let Err(e) = asset_manager.load_sprite_assets(world, &PlayerDirs::data()) {
                log::error!("Failed to load sprite assets: {e}");
            }
            scene_renderer.upload_textures(&state.device, &state.queue, &asset_manager.textures);
        }

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        if let Some(world) = &self.world {
            if let Err(e) = scene_renderer.render(
                &state.device,
                &state.queue,
                &mut encoder,
                &view,
                world,
                Vec2::new(state.config.width as f32, state.config.height as f32),
            ) {
                log::error!("Render failed: {e}");
            };
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        phantom_core::scripting::script_scheduler::ScriptScheduler::run_all_update_scripts(
            &mut self.world.as_mut().unwrap(),
            &self.script_context.as_mut().unwrap(),
        );

        if let Some(script_ctx) = &mut self.script_context {
            let window_size = Vec2::new(state.config.width as f32, state.config.height as f32);

            let (camera_pos, zoom, ref_res) = self
                .world
                .as_ref()
                .and_then(|world| {
                    use phantom_core::ecs::components::{Transform, camera::Camera};
                    let cams = world.query_with::<Camera>();
                    let cam_entity = cams.first()?;
                    let cam = world.get_component::<Camera>(*cam_entity)?;
                    let transform = world.get_component::<Transform>(*cam_entity)?;
                    Some((
                        Vec2::new(transform.position.x, transform.position.y),
                        cam.zoom,
                        cam.reference_resolution.as_vec2(),
                    ))
                })
                .unwrap_or((Vec2::ZERO, 100.0, Vec2::new(1280.0, 720.0)));

            let input_system = &mut script_ctx.input;
            input_system.set_viewport(ViewportInfo {
                size: window_size,
                offset: Vec2::ZERO,
                camera_pos,
                zoom,
                reference_resolution: ref_res,
            });
            input_system.end_frame();

            let time_system = &mut script_ctx.time;
            time_system.tick();
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            scene_renderer: None,
            world: None,
            script_context: None,
            game_dylib: None,
            asset_manager: None,
        }
    }

    pub fn run() -> anyhow::Result<()> {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

        let event_loop = EventLoop::with_user_event().build()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let mut app = App::new();
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}
