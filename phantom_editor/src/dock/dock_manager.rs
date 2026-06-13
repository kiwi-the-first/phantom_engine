use std::collections::HashMap;

use egui::Ui;
use egui_dock::{DockArea, DockState, NodeIndex, NodePath, Style, SurfaceIndex};

use crate::{
    actions::Actions,
    context::{EditorContext, panel_context::PanelContext},
    dock::WorkspacePresets,
    panels::ViewportState,
    persitance::layout,
    workspaces::{Workspace, WorkspaceDescriptor, WorkspaceViewer},
};

/// Owns the workspace dock: the live `DockState`, the catalogue of openable
/// workspaces, and all open/save/load operations. Nothing outside this module
/// touches `DockState` directly.
pub struct DockManager {
    dock_state: DockState<Workspace>,
    available_workspaces: HashMap<&'static str, WorkspaceDescriptor>,
}

impl DockManager {
    pub fn new() -> Self {
        let mut available_workspaces = HashMap::new();
        for desc in WorkspacePresets::all() {
            available_workspaces.insert(desc.name, desc);
        }

        // Open the default workspace as the initial tab.
        let initial = Self::build_workspace(&WorkspacePresets::level_editor());
        let mut dock_state = DockState::new(vec![initial]);
        dock_state.set_focused_node_and_surface(NodePath::new(SurfaceIndex(0), NodeIndex(0)));

        Self {
            dock_state,
            available_workspaces,
        }
    }

    /// Build a workspace from a descriptor, restoring its saved layout if one exists.
    fn build_workspace(desc: &WorkspaceDescriptor) -> Workspace {
        let mut workspace = Workspace::new(desc.name.to_string(), desc.panels.clone());
        if let Ok(saved) = layout::load(desc.name.to_string()) {
            workspace.panel_dock_state = saved;
        }
        workspace
    }

    /// Open a registered workspace as a new tab.
    pub fn open(&mut self, name: &str) {
        let Some(desc) = self.available_workspaces.get(name) else {
            log::warn!("Tried to open unknown workspace: {name}");
            return;
        };
        let workspace = Self::build_workspace(desc);
        self.dock_state.push_to_first_leaf(workspace);
    }

    /// Names of all registered workspaces, for menu listing.
    pub fn available_names(&self) -> Vec<&'static str> {
        self.available_workspaces.keys().copied().collect()
    }

    /// Name of the currently focused workspace, if any.
    pub fn active_workspace_name(&mut self) -> Option<String> {
        self.dock_state
            .find_active_focused()
            .map(|(_, ws)| ws.name.clone())
    }

    pub fn save_active_layout(&mut self) {
        if let Some((_, ws)) = self.dock_state.find_active_focused() {
            if let Err(e) = layout::save(ws.name.clone(), &ws.panel_dock_state) {
                log::error!("Failed to save layout: {e}");
            }
        }
    }

    pub fn load_active_default_layout(&mut self) {
        let Some(name) = self.active_workspace_name() else {
            return;
        };
        let Some(kind) = self.available_workspaces.get(name.as_str()).map(|d| d.kind) else {
            return;
        };
        match layout::load_default(kind) {
            Ok(layout) => {
                if let Some((_, ws)) = self.dock_state.find_active_focused() {
                    ws.panel_dock_state = layout;
                }
            }
            Err(e) => log::error!("Failed to load default layout: {e}"),
        }
    }

    pub fn load_active_custom_layout(&mut self) {
        let Some((_, ws)) = self.dock_state.find_active_focused() else {
            return;
        };
        match layout::load(ws.name.clone()) {
            Ok(layout) => ws.panel_dock_state = layout,
            Err(e) => log::error!("Failed to load custom layout: {e}"),
        }
    }

    /// Draw the workspace dock, threading the editor's shared state down to panels.
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        editor: &mut EditorContext,
        actions: &mut Actions,
        panel_context: &mut PanelContext,
    ) {
        let show_close = self.dock_state.iter_all_tabs().count() > 1;
        let mut viewer = WorkspaceViewer {
            editor,
            actions,
            panel_context,
        };
        DockArea::new(&mut self.dock_state)
            .show_leaf_collapse_buttons(false)
            .show_leaf_close_all_buttons(false)
            .show_close_buttons(show_close)
            .draggable_tabs(false)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut viewer);
    }
}
