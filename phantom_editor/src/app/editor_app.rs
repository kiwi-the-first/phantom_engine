use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::wgpu;
use phantom_core::scripting::ScriptContext;
use phantom_project::project_manager::project_manager::ProjectManager;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Icon;
use winit::window::{Window, WindowId};

use log::*;

use phantom_runtime::renderer::state::State;

use crate::actions::Actions;
use crate::context::EditorContext;
use crate::context::panel_context::PanelContext;
use crate::dock::DockManager;
use crate::egui::egui_renderer::EguiRenderer;
use crate::logger::PhantomLogger;
use crate::panels::{ViewportState, asset_browser::AssetBrowserState};
use crate::shortcuts::EditorShortcuts;
use crate::theme::EditorTheme;
use crate::top_bar::TopBar;

pub struct EditorApp {
    state: Option<State>,
    egui_renderer: Option<EguiRenderer>,
    scale_factor: f32,
    is_closing: bool,
    dock: DockManager,
    editor_context: Option<EditorContext>,
    panel_context: Option<PanelContext>,
    actions: Actions,
    editor_theme: Option<EditorTheme>,
}

impl ApplicationHandler<State> for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let icon = self
            .create_window_icon()
            .inspect_err(|e| log::error!("FAILED TO CREATE WINDOW ICON! {e}"))
            .ok();

        let window_attributes = Window::default_attributes()
            .with_title("Phantom Engine")
            .with_window_icon(icon);

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let state = pollster::block_on(State::new(window.clone())).unwrap();

        let mut egui_renderer =
            EguiRenderer::new(&window, &state.device, state.surface_format(), None, 1);

        let ectx = self.editor_context.as_mut().unwrap();

        // LOAD TEXTURES ON STARTUP
        ectx.sync_assets().expect("COULD NOT LOAD ASSETS");
        // BUILD GAME CODE
        ectx.build_project();
        // LOAD GAME DYLIB
        if let Err(e) = ectx.load_dylib(None) {
            log::error!("FAILED TO LOAD GAME DYLIB! {e}");
        }

        if let Err(e) = ectx.load_world() {
            log::error!("FAILED TO LOAD GAME WORLD! {e}");
        }

        let panel_context = PanelContext {
            asset_browser: AssetBrowserState::new(),
            viewport: ViewportState::new(
                &state.device,
                &state.queue,
                &mut egui_renderer,
                egui::Vec2::new(800.0, 600.0),
            ),
        };

        if let Err(e) = ectx.asset_manager.init(&ectx.project_path) {
            log::error!("FAILED TO INITIALIZE ASSET MANAGER! {e}");
        }

        let editor_theme = EditorTheme::default();
        editor_theme.apply(egui_renderer.context());
        self.state = Some(state);
        self.panel_context = Some(panel_context);
        self.egui_renderer = Some(egui_renderer);
    }
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        event_loop.set_control_flow(ControlFlow::Poll);

        if let Some(egui_renderer) = &mut self.egui_renderer {
            if let Some(state) = &self.state {
                egui_renderer.handle_input(&state.window, &event);
            }
        }

        if let Some(ectx) = self.editor_context.as_mut() {
            if let Some(input_system) = ectx.input_system.as_mut() {
                input_system.handle_event(&event);
            }
        }

        match event {
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size.width, physical_size.height);
                }
            }
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                self.is_closing = true;

                drop(self.egui_renderer.take());
                // Give threads time to cleanup
                std::thread::sleep(std::time::Duration::from_millis(50));

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
            WindowEvent::Focused(focused) => {
                // When the window loses focus the OS may capture the pointer
                // (e.g. for a drag-and-drop from an external app) and swallow
                // the mouse-button-release event.  Egui never sees the release
                // so it keeps thinking the button is held, which causes text
                // selection / dragging to get stuck.  Sending PointerGone
                // clears all pointer state so egui starts fresh on the next
                // interaction.
                if !focused {
                    if let Some(egui_renderer) = &mut self.egui_renderer {
                        egui_renderer.inject_event(egui::Event::PointerGone);
                    }
                }
            }
            WindowEvent::DroppedFile(file_path) => {
                log::debug!("FILE DROPPED");
                if let (Some(ectx), Some(pctx)) = (&mut self.editor_context, &self.panel_context) {
                    ectx.asset_manager
                        .import_asset(file_path, pctx.asset_browser.current_dir())
                        .expect("FAILED TO IMPORT");
                    self.panel_context
                        .as_mut()
                        .unwrap()
                        .asset_browser
                        .should_create_entries = true;
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
                // (KeyCode::Escape, true) => event_loop.exit(),
                _ => {}
            },
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.state.is_none() || self.is_closing {
            return;
        }

        self.handle_redraw();

        let viewport_info = self.panel_context.as_ref().and_then(|v| v.viewport.info());
        if let Some(ectx) = self.editor_context.as_mut() {
            if let Some(input_system) = ectx.input_system.as_mut() {
                if let Some(info) = viewport_info {
                    input_system.set_viewport(info);
                }
                input_system.end_frame();
            }
            if let Some(time_system) = ectx.time_system.as_mut() {
                time_system.tick();
            }
        }
    }
}

impl EditorApp {
    pub fn new() -> Self {
        Self {
            state: None,
            egui_renderer: None,
            scale_factor: 1.0,
            is_closing: false,
            dock: DockManager::new(),
            editor_context: None,
            panel_context: None,
            actions: Actions::new(),
            editor_theme: None,
        }
    }

    pub fn run(project_path: PathBuf) -> anyhow::Result<()> {
        let (project, init_world) = ProjectManager::load(project_path.clone())?;

        let max_level = Level::Debug;
        let logger = PhantomLogger::new(max_level);
        let buffer = logger.buffer.clone();
        log::set_boxed_logger(Box::new(logger)).unwrap();
        log::set_max_level(max_level.to_level_filter());

        let editor_context = EditorContext::new(project_path, project, init_world, buffer);
        let event_loop = EventLoop::with_user_event().build()?;
        let mut app = EditorApp::new();
        app.editor_context = Some(editor_context);
        event_loop.run_app(&mut app)?;
        Ok(())
    }

    fn create_window_icon(&self) -> anyhow::Result<Icon> {
        let icon_bytes = include_bytes!("../images/phantom_engine_icon_256.png");

        let icon_image = match image::load_from_memory(icon_bytes) {
            Ok(image) => image.to_rgba8(),
            Err(e) => return Err(anyhow!("FAILED TO LOAD ICON IMAGE! {e}")),
        };

        let (width, height) = icon_image.dimensions();
        let icon = match Icon::from_rgba(icon_image.into_raw(), width, height) {
            Ok(icon) => icon,
            Err(e) => return Err(anyhow!("FAILED TO GENERATE ICON! {e}")),
        };

        Ok(icon)
    }

    fn handle_redraw(&mut self) {
        // Attempt to handle minimizing window
        if let Some(state) = &self.state {
            if let Some(true) = state.window.is_minimized() {
                return;
            }
        }

        let scale_factor = self.scale_factor;

        // All borrows of `self` are confined to this block; the post-frame resize
        // runs once they're released.
        'redraw: {
            let Self {
                state,
                egui_renderer,
                editor_context,
                panel_context,
                actions,
                dock,
                ..
            } = self;

            let state = state.as_mut().unwrap();
            let egui_renderer = egui_renderer.as_mut().unwrap();
            let editor_context = editor_context.as_mut().unwrap();
            let panel_context = panel_context.as_mut().unwrap();

            let screen_descriptor = ScreenDescriptor {
                size_in_pixels: [state.config.width, state.config.height],
                pixels_per_point: state.window.as_ref().scale_factor() as f32 * scale_factor,
            };

            let output = match state.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(texture) => texture,
                wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                _ => {
                    log::warn!("Surface error, skipping frame");
                    break 'redraw;
                }
            };

            let surface_view = output.texture.create_view(&Default::default());

            let mut encoder = state.device.create_command_encoder(&Default::default());

            // Pin anchored UI to the camera before drawing (runs in edit mode too,
            // so anchored sprites preview correctly).
            let viewport = &mut panel_context.viewport;
            let viewport_size = glam::Vec2::new(viewport.size().x, viewport.size().y);
            phantom_core::ui::update_anchors(&mut editor_context.active_world, viewport_size);

            // SCENE RENDER
            viewport.render_scene(
                &state.device,
                &state.queue,
                &mut encoder,
                &editor_context.active_world,
                &editor_context.asset_manager,
                editor_context.is_playing,
            );

            if editor_context.is_playing {
                let delta;
                {
                    let (active_world, input_system, time_system, audio_ctx) = editor_context
                        .get_world_and_systems()
                        .expect("FAILED TO GET WORLD AND SYSTEMS!");
                    delta = time_system.time_ctx.delta;
                    let script_ctx = ScriptContext {
                        input: &input_system.input_ctx,
                        time: &time_system.time_ctx,
                        audio: audio_ctx,
                    };
                    phantom_core::scripting::script_scheduler::ScriptScheduler::run_all_update_scripts(
                        active_world,
                        &script_ctx,
                    );
                }
                phantom_core::animation::AnimationSystem::update(
                    &mut editor_context.active_world,
                    delta,
                );
                phantom_core::collision::CollisionSystem::update(&mut editor_context.active_world);
                // Drain sounds the update scripts queued, then reap finished ones.
                editor_context.audio_system.update();
            }

            // Try to sync assets if there are none to sync this will skip.
            if let Err(e) = editor_context.sync_assets() {
                log::error!("Failed to sync assets: {}", e);
            }
            viewport.upload_textures(
                &state.device,
                &state.queue,
                &editor_context.texture_loader.textures,
            );

            // EGUI FRAME
            let window = state.window.as_ref();
            egui_renderer.run(window, |ctx| {
                //EditorShortcuts::handle(ctx, actions, editor_context);

                let screen_rect = ctx.viewport_rect();

                // DRAW HERE
                egui::Area::new("main_dock_area".into()).show(ctx, |ui| {
                    ui.set_max_size(screen_rect.size());

                    ui.vertical(|ui| {
                        TopBar::show(ui, editor_context, dock);
                        dock.ui(ui, editor_context, actions, panel_context);
                    });
                });
            });

            //END DRAW
            egui_renderer.end_frame_and_draw(
                &state.device,
                &state.queue,
                &mut encoder,
                window,
                &surface_view,
                screen_descriptor,
            );

            state.queue.submit(Some(encoder.finish()));
            output.present();
        }

        // POST-FRAME (no borrows of `self` alive)
        // The viewport recreates its render target if the panel was resized this frame.
        let Self {
            state,
            egui_renderer,
            panel_context,
            ..
        } = self;
        if let (Some(state), Some(egui_renderer), Some(panel_context)) = (
            state.as_ref(),
            egui_renderer.as_mut(),
            panel_context.as_mut(),
        ) {
            let viewport = &mut panel_context.viewport;
            viewport.apply_resize(&state.device, egui_renderer);
        }
    }
}
