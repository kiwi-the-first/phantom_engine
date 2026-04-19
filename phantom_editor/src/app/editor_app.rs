use std::collections::HashMap;
use std::sync::Arc;

use egui_dock::DockState;
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::wgpu;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use log::*;

use phantom_runtime::renderer::state::State;

use crate::egui::egui_renderer::EguiRenderer;
use crate::menus::view::ViewMenu;
use crate::panels::Panels;
use crate::workspaces::Workspace;
use crate::workspaces::WorkspaceConfig;
use crate::workspaces::WorkspaceKind;
use crate::workspaces::WorkspaceViewer;

pub struct EditorApp {
    state: Option<State>,
    egui_renderer: Option<EguiRenderer>,
    scale_factor: f32,
    is_closing: bool,
    avalible_workspaces: HashMap<String, WorkspaceConfig>,
    dock_state: DockState<Workspace>,
    workspace_viewer: WorkspaceViewer,
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
        event_loop.set_control_flow(ControlFlow::Poll);

        if let Some(egui_renderer) = &mut self.egui_renderer {
            if let Some(state) = &self.state {
                egui_renderer.handle_input(&state.window, &event);
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
        if self.state.is_none() || self.is_closing {
            // ← CHECK FLAG
            return; // Don't render if closing!
        }
        self.handle_redraw();
    }
}

impl EditorApp {
    pub fn new() -> Self {
        let mut available_workspaces = HashMap::new();

        available_workspaces.insert(
            "Level Editor".to_string(),
            WorkspaceConfig {
                name: "Level Editor".to_string(),
                kind: WorkspaceKind::BuiltIn,
                panels: vec![
                    Panels::Viewport,
                    Panels::Hierarchy,
                    Panels::Inspector,
                    Panels::Console,
                    Panels::AssetBrowser,
                ],
            },
        );

        let level_editor = Workspace::new(
            "Level Editor".to_string(),
            vec![
                Panels::Viewport,
                Panels::Hierarchy,
                Panels::Inspector,
                Panels::Console,
                Panels::AssetBrowser,
            ],
        );

        let default_open_workspaces = vec![level_editor];
        let dock_state = DockState::new(default_open_workspaces);
        Self {
            state: None,
            egui_renderer: None,
            scale_factor: 1.0,
            is_closing: false,
            dock_state: dock_state,
            avalible_workspaces: available_workspaces,
            workspace_viewer: WorkspaceViewer::new(),
        }
    }

    pub fn run() -> anyhow::Result<()> {
        env_logger::init();

        let event_loop = EventLoop::with_user_event().build()?;
        let mut app = EditorApp::new();
        event_loop.run_app(&mut app)?;

        Ok(())
    }
    pub fn open_workspaces(&mut self, name: &str) {
        if let Some(config) = self.avalible_workspaces.get(name) {
            let workspace = Workspace::new(config.name.clone(), config.panels.clone());
            self.dock_state.push_to_first_leaf(workspace);
        }
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
        let mut workspace_action = None;

        {
            let egui_renderer = self.egui_renderer.as_mut().unwrap();

            egui_renderer.begin_frame(window);
            let ctx = egui_renderer.context();
            let screen_rect = ctx.viewport_rect();
            let show_close = self.dock_state.iter_all_tabs().count() > 1;
            //DRAW HERE

            egui::Area::new("main_dock_area".into()).show(ctx, |ui| {
                ui.set_max_size(screen_rect.size());

                egui::Panel::top("menu_bar").show_inside(ui, |ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("File", |ui| {});
                        ui.menu_button("Edit", |ui| {});
                        ui.menu_button("Tools", |ui| {});
                        ui.menu_button("View", |ui| {
                            workspace_action = ViewMenu::show(ui, &self.avalible_workspaces);
                        });
                        ui.menu_button("Help", |ui| {});
                        ui.menu_button("Editor", |ui| {});
                    });
                });

                egui_dock::DockArea::new(&mut self.dock_state)
                    .show_leaf_collapse_buttons(false)
                    .show_leaf_close_all_buttons(false)
                    .show_close_buttons(show_close)
                    .draggable_tabs(false)
                    .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                    .show_inside(ui, &mut self.workspace_viewer);
            });

            //END DRAW
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

        if let Some(name) = workspace_action {
            self.open_workspaces(&name);
        }
    }
}
