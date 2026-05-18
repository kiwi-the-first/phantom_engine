use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use egui::Id;

use egui::Key;
use egui::Modifiers;
use egui::include_image;
use egui_dock::DockState;
use egui_dock::NodeIndex;
use egui_dock::NodePath;
use egui_dock::SurfaceIndex;
use egui_wgpu::ScreenDescriptor;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::TextureView;
use egui_wgpu::wgpu::wgt::TextureViewDescriptor;
use phantom_common::dirs;
use phantom_runtime::asset_manager::asset_manager;
use phantom_runtime::asset_manager::asset_manager::AssetManager;
use phantom_runtime::renderer::scene_renderer::SceneRenderer;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use log::*;

use phantom_runtime::renderer::state::State;

use crate::actions::Actions;
use crate::context::EditorContext;
use crate::context::editor_context;
use crate::egui::egui_renderer::EguiRenderer;
use crate::logger::Logger;
use crate::menus::view::ViewMenu;
use crate::menus::view::ViewMenuAction;
use crate::panels::Panels;
use crate::persitance::layout;
use crate::resources::ResourceKey;
use crate::workspaces::BuiltInWorkspace;
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
    view_port_texture: Option<wgpu::Texture>,
    scene_renderer: Option<SceneRenderer>,
    viewport_view: Option<TextureView>,
    editor_context: Option<Arc<Mutex<EditorContext>>>,
}

impl ApplicationHandler<State> for EditorApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create Window Icon
        let icon_bytes = include_bytes!("../images/phantom_engine_icon_256.png");
        let icon_image = image::load_from_memory(icon_bytes).unwrap().to_rgba8();
        let (width, height) = icon_image.dimensions();
        let icon = winit::window::Icon::from_rgba(icon_image.into_raw(), width, height).unwrap();

        let window_attributes = Window::default_attributes()
            .with_title("Phantom Engine")
            .with_window_icon(Some(icon));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let state = pollster::block_on(State::new(window.clone())).unwrap();

        let egui_renderer =
            EguiRenderer::new(&window, &state.device, state.surface_format(), None, 1);

        self.state = Some(state);
        self.egui_renderer = Some(egui_renderer);

        self.editor_context
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .sync_assets()
            .expect("COULD NOT LOAD ASSETS");

        if self.state.is_some() {
            let view_port_texture =
                self.state
                    .as_ref()
                    .unwrap()
                    .device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some("viewport"),
                        size: wgpu::Extent3d {
                            width: 800,
                            height: 600,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
                    });
            self.view_port_texture = Some(view_port_texture);

            self.scene_renderer = Some(SceneRenderer::new(
                &self.state.as_ref().unwrap().device,
                wgpu::TextureFormat::Rgba8UnormSrgb,
            ));

            let viewport_view = self
                .view_port_texture
                .as_ref()
                .unwrap()
                .create_view(&Default::default());

            let texture_id = self
                .egui_renderer
                .as_mut()
                .unwrap()
                .egui_renderer
                .register_native_texture(
                    &self.state.as_ref().unwrap().device,
                    &viewport_view,
                    wgpu::FilterMode::Linear,
                );

            // Send TextureId to ctx
            self.egui_renderer
                .as_mut()
                .unwrap()
                .context()
                .data_mut(|d| {
                    d.insert_temp(Id::new(ResourceKey::ViewportTexture), texture_id);
                });

            self.viewport_view = Some(viewport_view);

            let actions = Arc::new(Mutex::new(Actions::new()));

            self.egui_renderer
                .as_mut()
                .unwrap()
                .context()
                .data_mut(|w| {
                    w.insert_temp(Id::new(ResourceKey::Actions), actions);
                    w.insert_temp(
                        Id::new(ResourceKey::EditorContext),
                        self.editor_context.take().unwrap(),
                    );
                });
        }
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
            return;
        }

        self.prepare_ui_context();
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
                kind: WorkspaceKind::BuiltIn(BuiltInWorkspace::LevelEditor),
                panels: vec![
                    Panels::Viewport,
                    Panels::Hierarchy,
                    Panels::Inspector,
                    Panels::Console,
                    Panels::AssetBrowser,
                ],
            },
        );

        let mut level_editor = Workspace::new(
            "Level Editor".to_string(),
            vec![
                Panels::Viewport,
                Panels::Hierarchy,
                Panels::Inspector,
                Panels::Console,
                Panels::AssetBrowser,
            ],
        );
        if let Some(layout) = layout::load("Level Editor".to_string()).ok() {
            level_editor.panel_dock_state = layout;
        }

        let default_open_workspaces = vec![level_editor];
        let mut dock_state = DockState::new(default_open_workspaces);
        // Set active tab to first tab
        dock_state.set_focused_node_and_surface(NodePath::new(SurfaceIndex(0), NodeIndex(0)));
        Self {
            state: None,
            egui_renderer: None,
            scale_factor: 1.0,
            is_closing: false,
            dock_state: dock_state,
            avalible_workspaces: available_workspaces,
            workspace_viewer: WorkspaceViewer::new(),
            view_port_texture: None,
            viewport_view: None,
            scene_renderer: None,
            editor_context: None,
        }
    }

    pub fn run(editor_context: EditorContext) -> anyhow::Result<()> {
        if let Some(file) = Logger::create_log_file() {
            env_logger::Builder::new()
                .target(env_logger::Target::Stdout)
                //.target(env_logger::Target::Pipe(Box::new(file)))
                .filter_module("phantom_editor", log::LevelFilter::Trace)
                .filter_module("phantom_runtime", log::LevelFilter::Trace)
                .init();
        } else {
            log::trace!(
                "FAILED TO CREATE LOG FILE AT {}",
                dirs::cache().unwrap().to_str().unwrap()
            );
            env_logger::init();
        }

        let event_loop = EventLoop::with_user_event().build()?;
        let mut app = EditorApp::new();
        app.editor_context = Some(Arc::new(Mutex::new(editor_context)));
        event_loop.run_app(&mut app)?;

        Ok(())
    }

    pub fn open_workspaces(&mut self, name: &str) {
        if let Some(config) = self.avalible_workspaces.get(name) {
            let workspace = Workspace::new(config.name.clone(), config.panels.clone());
            self.dock_state.push_to_first_leaf(workspace);
        }
    }

    fn prepare_ui_context(&mut self) {
        let active_workspace_name = self
            .dock_state
            .find_active_focused()
            .unwrap()
            .1
            .name
            .clone();

        let active_builtin_workspace_type: Option<BuiltInWorkspace> = match self
            .avalible_workspaces
            .get(&active_workspace_name)
            .unwrap()
            .kind
        {
            WorkspaceKind::BuiltIn(wtype) => Some(wtype),
            WorkspaceKind::Custom => None,
        };

        let available_workspaces: Vec<String> = self.avalible_workspaces.keys().cloned().collect();

        // INSERT RESOURCES TO BE USED BY EGUI PANELS AND MENUS
        self.egui_renderer
            .as_mut()
            .unwrap()
            .context()
            .data_mut(|w| {
                w.insert_temp(
                    Id::new(ResourceKey::ActiveWorkspaceName),
                    active_workspace_name,
                );
                w.insert_temp(
                    Id::new(ResourceKey::AvailableWorkspaces),
                    available_workspaces,
                );
                w.insert_temp(
                    Id::new(ResourceKey::ActiveWorkspaceBuiltInType),
                    active_builtin_workspace_type,
                )
            });
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

        let viewport_render_view =
            self.view_port_texture
                .as_ref()
                .unwrap()
                .create_view(&TextureViewDescriptor {
                    format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                    ..Default::default()
                });

        let mut encoder = state.device.create_command_encoder(&Default::default());
        self.scene_renderer
            .as_mut()
            .unwrap()
            .render(&mut encoder, &viewport_render_view)
            .expect("RENDERING FAILED"); // this a terrible way to handle this error but rn im lazy

        let window = state.window.as_ref();

        let mut view_menu_action = None;

        {
            let show_close = self.dock_state.iter_all_tabs().count() > 1;
            let egui_renderer = self.egui_renderer.as_mut().unwrap();

            egui_renderer.begin_frame(window);
            let ctx = egui_renderer.context();

            let (should_undo, should_redo) = ctx.input_mut(|i| {
                let redo = i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
                let undo = i.consume_key(Modifiers::COMMAND, Key::Z);
                (undo, redo)
            });

            if should_undo {
                if let Some(actions) = ctx
                    .data_mut(|w| w.get_temp::<Arc<Mutex<Actions>>>(Id::new(ResourceKey::Actions)))
                {
                    let mut actions = actions.lock().unwrap();
                    actions.undo(
                        &ctx.data_mut(|w| {
                            w.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(
                                ResourceKey::EditorContext,
                            ))
                        })
                        .unwrap(),
                    );
                }
            }
            if should_redo {
                if let Some(actions) = ctx
                    .data_mut(|w| w.get_temp::<Arc<Mutex<Actions>>>(Id::new(ResourceKey::Actions)))
                {
                    let mut actions = actions.lock().unwrap();
                    actions.redo(
                        &ctx.data_mut(|w| {
                            w.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(
                                ResourceKey::EditorContext,
                            ))
                        })
                        .unwrap(),
                    );
                }
            }

            // Try to sync assets if there are no assets to sync this will skip
            if let Some(ectx) = &ctx.data_mut(|w| {
                w.get_temp::<Arc<Mutex<EditorContext>>>(Id::new(ResourceKey::EditorContext))
            }) {
                if let Err(e) = ectx.lock().unwrap().sync_assets() {
                    log::error!("Failed to sync assets: {}", e);
                }
            }

            let mut visuals = ctx.style().visuals.clone();
            let black = egui::Color32::BLACK;
            visuals.window_fill = black.clone();
            visuals.panel_fill = black.clone();
            visuals.extreme_bg_color = black.clone();
            visuals.selection.bg_fill = egui::Color32::from_hex("#8959d5").unwrap();
            visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);

            let screen_rect = ctx.viewport_rect();

            ctx.set_visuals(visuals);
            //DRAW HERE

            egui::Area::new("main_dock_area".into()).show(ctx, |ui| {
                ui.set_max_size(screen_rect.size());

                ui.vertical(|ui| {
                    egui::Panel::top("menu_bar").show_inside(ui, |ui| {
                        ui.allocate_ui(egui::vec2(ui.available_width(), 0.0), |ui| {
                            ui.horizontal_top(|ui| {
                                ui.add(
                                    egui::Image::new(include_image!(
                                        "../images/phantom_engine_icon_glow_256.png"
                                    ))
                                    .fit_to_exact_size(egui::vec2(48.0, 48.0)),
                                );

                                egui::MenuBar::new().ui(ui, |ui| {
                                    ui.menu_button("File", |ui| {});
                                    ui.menu_button("Edit", |ui| {});
                                    ui.menu_button("Tools", |ui| {});
                                    ui.menu_button("View", |ui| {
                                        view_menu_action = ViewMenu::show(ui)
                                    });
                                    ui.menu_button("Help", |ui| {});
                                    ui.menu_button("Editor", |ui| {});
                                });
                            });
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

        match view_menu_action {
            Some(ViewMenuAction::OpenWorkspace(name)) => {
                self.open_workspaces(&name);
            }
            Some(ViewMenuAction::SaveLayout) => {
                let workspace = self.dock_state.find_active_focused().unwrap();
                let name = workspace.1.name.clone();
                let dock_state = workspace.1.panel_dock_state.clone();
                layout::save(name, &dock_state).expect("failed to save");
            }
            Some(ViewMenuAction::LoadCustomLayout(name)) => {
                let layout = layout::load(name).unwrap();
                self.dock_state
                    .find_active_focused()
                    .unwrap()
                    .1
                    .panel_dock_state = layout;
            }
            Some(ViewMenuAction::LoadDefaultLayout(workspace_type)) => {
                let layout = layout::load_default(workspace_type).unwrap();
                self.dock_state
                    .find_active_focused()
                    .unwrap()
                    .1
                    .panel_dock_state = layout;
            }
            None => {}
        }
    }
}
