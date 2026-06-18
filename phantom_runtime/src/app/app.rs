use std::sync::Arc;

use crate::audio::AudioSystem;
use glam::Vec2;
use libloading::Library;
use phantom_assets::asset_manager::{AssetManager, asset_manager};
use phantom_assets::texture_loader::TextureLoader;
use phantom_common::dirs;
use phantom_common::dirs::dirs::PlayerDirs;
use phantom_core::ecs::World;
use phantom_core::input::InputSystem;
use phantom_core::input::input_context::ViewportInfo;
use phantom_core::scripting::ScriptContext;
use phantom_core::scripting::script_scheduler::ScriptScheduler;
use phantom_core::time::time_system::TimeSystem;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use log::*;

use crate::game_loader::game_loader::GameLoader;
use crate::renderer::scene_renderer::SceneRenderer;
use crate::renderer::state::State;

#[derive(Default)]
pub struct App {
    state: Option<State>,
    scene_renderer: Option<SceneRenderer>,
    world: Option<World>,
    asset_manager: Option<AssetManager>,
    texture_loader: Option<TextureLoader>,
    input_system: Option<InputSystem>,
    time_system: Option<TimeSystem>,
    audio_system: AudioSystem,

    /// Held to prevent drop from memory
    game_dylib: Option<Library>,
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let state = pollster::block_on(State::new(window.clone())).unwrap();

        let mut scene_renderer =
            SceneRenderer::new(&state.device, &state.queue, state.surface_format());

        scene_renderer.resize(
            &state.device,
            window.inner_size().width as u32,
            window.inner_size().height as u32,
        );

        let game_dylib = match GameLoader::load_dylib() {
            Ok(dylib) => {
                GameLoader::init_dylib(&dylib).expect("FAILED TO INITIALIZE GAME DYLIB!");
                log::info!("Game dylib loaded and initialized successfully!");
                dylib
            }
            Err(e) => panic!("FAILED TO LOAD GAME DYLIB! {e}"),
        };

        // World MUST be loaded after the game_dylib!
        let mut world = match GameLoader::load_world() {
            Ok(world) => {
                log::info!("World loaded successfully!");
                world
            }
            Err(e) => panic!("FAILED TO LOAD WORLD FILE! {e}"),
        };

        let mut asset_manager = AssetManager::default();

        asset_manager.init(&dirs::PlayerDirs::data());

        let mut texture_loader = TextureLoader::default();

        match texture_loader.load_sprite_assets(&mut asset_manager) {
            Ok(_) => {
                scene_renderer.upload_textures(
                    &state.device,
                    &state.queue,
                    &texture_loader.textures,
                );
                log::info!("Sprites loaded successfully!")
            }
            Err(e) => {
                panic!("FAILED TO LOAD SPRITE ASSETS!: {e}");
            }
        }

        let time_system = TimeSystem::default();
        let input_system = InputSystem::default();
        {
            let script_ctx = ScriptContext {
                input: &input_system.input_ctx,
                time: &time_system.time_ctx,
                audio: &self.audio_system.audio_ctx,
            };
            ScriptScheduler::run_all_start_scripts(&mut world, &script_ctx);
        }
        // Play any sounds the start scripts queued.
        self.audio_system.update();

        self.state = Some(state);
        self.scene_renderer = Some(scene_renderer);
        self.world = Some(world);
        self.asset_manager = Some(asset_manager);
        self.texture_loader = Some(texture_loader);
        self.input_system = Some(input_system);
        self.time_system = Some(time_system);
        self.game_dylib = Some(game_dylib);
    }
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut _event: State) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(input_system) = &mut self.input_system {
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
                if let (Some(state), Some(scene_renderer)) =
                    (&mut self.state, &mut self.scene_renderer)
                {
                    let width = physical_size.width;
                    let height = physical_size.height;
                    state.resize(width, height);
                    scene_renderer.resize(&state.device, width, height);
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
        // if let (Some(world), Some(asset_manager)) = (&self.world, &mut self.texture_loader) {
        //     if let Err(e) = asset_manager.load_sprite_assets(world, &PlayerDirs::data()) {
        //         log::error!("Failed to load sprite assets: {e}");
        //     }
        //     scene_renderer.upload_textures(&state.device, &state.queue, &asset_manager.textures);
        // }

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let viewport_size = Vec2::new(state.config.width as f32, state.config.height as f32);
        if let (Some(world), Some(asset_manager)) = (&mut self.world, &self.asset_manager) {
            // Pin anchored UI to the camera before drawing.
            phantom_core::ui::update_anchors(world, viewport_size);
            if let Err(e) = scene_renderer.render(
                &state.device,
                &state.queue,
                &mut encoder,
                &view,
                world,
                asset_manager,
                viewport_size,
                true,
            ) {
                log::error!("Render failed: {e}");
            };
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        let viewport_info = self
            .gather_viewport_info()
            .expect("CAMERA MUST EXIST TO RENDER!");

        if let (Some(input_system), Some(time_system), Some(world)) = (
            &mut self.input_system,
            &mut self.time_system,
            &mut self.world,
        ) {
            {
                let script_ctx = ScriptContext {
                    input: &input_system.input_ctx,
                    time: &time_system.time_ctx,
                    audio: &self.audio_system.audio_ctx,
                };
                ScriptScheduler::run_all_update_scripts(world, &script_ctx);
            }
            phantom_core::animation::AnimationSystem::update(world, time_system.time_ctx.delta);
            phantom_core::collision::CollisionSystem::update(world);
            // Drain sounds the update scripts queued, then reap finished ones.
            self.audio_system.update();

            input_system.set_viewport(viewport_info);

            input_system.end_frame();
            time_system.tick();
        };
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            state: None,
            scene_renderer: None,
            world: None,
            asset_manager: None,
            game_dylib: None,
            texture_loader: None,
            input_system: None,
            time_system: None,
            audio_system: AudioSystem::default(),
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

    fn gather_viewport_info(&self) -> anyhow::Result<ViewportInfo> {
        let state = self.state.as_ref().expect("FAILED TO GATHER STATE!");

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
            .ok_or_else(|| anyhow::anyhow!("No camera found in world"))?;

        let info = ViewportInfo {
            size: window_size,
            offset: Vec2::ZERO,
            camera_pos,
            zoom,
            reference_resolution: ref_res,
        };

        Ok(info)
    }
}
