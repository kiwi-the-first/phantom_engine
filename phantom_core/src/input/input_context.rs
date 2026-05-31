use std::collections::HashSet;

use glam::Vec2;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
pub struct InputContext {
    pub(crate) keys_held: HashSet<KeyCode>,
    pub(crate) mouse_held: HashSet<MouseButton>,
    pub(crate) mouse_pos: Option<Vec2>,

    // Cleared each frame
    pub(crate) keys_pressed: HashSet<KeyCode>,
    pub(crate) keys_released: HashSet<KeyCode>,
    pub(crate) mouse_pressed: HashSet<MouseButton>,
    pub(crate) mouse_released: HashSet<MouseButton>,
    pub(crate) mouse_delta: Vec2,
    pub(crate) scroll_delta: Vec2,

    //Editor Viewport
    pub(crate) viewport: Option<ViewportInfo>,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            keys_held: HashSet::new(),
            mouse_held: HashSet::new(),
            mouse_pos: None,
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_released: HashSet::new(),
            mouse_delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            viewport: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ViewportInfo {
    /// The size of the viewport
    pub size: Vec2,
    /// The offset of the viewport from 0,0 being top left
    pub offset: Vec2,
    pub camera_pos: Vec2,
    pub zoom: f32,
    pub reference_resolution: Vec2,
}

impl InputContext {
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }
    pub fn mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }
    pub fn mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_released.contains(&button)
    }

    /// Cursor position in world space.
    pub fn mouse_pos(&self) -> Vec2 {
        let Some(viewport) = &self.viewport else {
            // No viewport mapping; return the raw screen position (or origin).
            return self.mouse_pos.unwrap_or(Vec2::ZERO);
        };

        let scale_x = viewport.size.x / viewport.reference_resolution.x * viewport.zoom;
        let scale_y = viewport.size.y / viewport.reference_resolution.y * viewport.zoom;
        let scale = scale_x.min(scale_y);

        // A non-positive / non-finite scale (zero zoom, zero-size viewport, etc.)
        // would divide to infinity below — fall back to the camera position.
        if !scale.is_finite() || scale <= 0.0 {
            return viewport.camera_pos;
        }

        // Until the cursor has actually moved, treat it as the viewport centre,
        // which maps to the camera position in world space.
        let screen = match self.mouse_pos {
            Some(pos) => pos - viewport.offset,
            None => viewport.size / 2.0,
        };

        let ndc = Vec2::new(
            screen.x - viewport.size.x / 2.0,
            -(screen.y - viewport.size.y / 2.0),
        );
        ndc / scale + viewport.camera_pos
    }
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }
}
