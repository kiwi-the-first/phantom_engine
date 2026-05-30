use std::collections::HashSet;

use glam::Vec2;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
pub struct Input {
    keys_held: HashSet<KeyCode>,
    mouse_held: HashSet<MouseButton>,
    /// Raw cursor position in screen pixels. `None` until the first `CursorMoved`
    /// event, so we never invent a bogus position (the old `f32::MAX` sentinel
    /// overflowed to infinity in the screen->world transform, producing NaN in
    /// any script that normalized a direction from the cursor).
    mouse_pos: Option<Vec2>,

    // cleared each frame
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_released: HashSet<MouseButton>,
    mouse_delta: Vec2,
    scroll_delta: Vec2,

    //Editor Viewport
    viewport: Option<ViewportInfo>,
}

impl Default for Input {
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
    pub size: Vec2,
    pub offset: Vec2,
    pub camera_pos: Vec2,
    pub zoom: f32,
    pub reference_resolution: Vec2,
}

impl Input {
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
    /// Cursor position in world space. Always finite.
    ///
    /// Note: this can legitimately coincide with another point (e.g. the cursor
    /// sitting exactly on the player, or the cursor being unknown and defaulting
    /// to the camera centre). Callers that build a direction from it must still
    /// guard against a zero-length vector before normalizing.
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

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    match key_event.state {
                        ElementState::Pressed => {
                            if !key_event.repeat && self.keys_held.insert(code) {
                                self.keys_pressed.insert(code);
                            }
                        }
                        ElementState::Released => {
                            if self.keys_held.remove(&code) {
                                self.keys_released.insert(code);
                            }
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = Vec2::new(position.x as f32, position.y as f32);
                // Only accumulate delta once we have a previous position, so the
                // first event doesn't report a huge spurious jump.
                if let Some(prev) = self.mouse_pos {
                    self.mouse_delta += new_pos - prev;
                }
                self.mouse_pos = Some(new_pos);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if self.mouse_held.insert(*button) {
                        self.mouse_pressed.insert(*button);
                    }
                }
                ElementState::Released => {
                    if self.mouse_held.remove(button) {
                        self.mouse_released.insert(*button);
                    }
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.scroll_delta += Vec2::new(*x, *y);
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        const PIXELS_PER_LINE: f32 = 50.0; // rough convention
                        self.scroll_delta += Vec2::new(p.x as f32, p.y as f32) / PIXELS_PER_LINE;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn set_viewport(&mut self, info: ViewportInfo) {
        self.viewport = Some(info);
    }

    pub fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = Vec2::ZERO;
    }
}
