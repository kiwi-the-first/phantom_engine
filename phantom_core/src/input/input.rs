use std::collections::HashSet;

use glam::Vec2;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
#[derive(Default)]
pub struct Input {
    keys_held: HashSet<KeyCode>,
    mouse_held: HashSet<MouseButton>,
    mouse_pos: Vec2,

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
    pub fn mouse_pos(&self) -> Vec2 {
        if let Some(viewport) = &self.viewport {
            let scale_x = viewport.size.x / viewport.reference_resolution.x * viewport.zoom;
            let scale_y = viewport.size.y / viewport.reference_resolution.y * viewport.zoom;
            let scale = scale_x.min(scale_y);
            let screen = self.mouse_pos - viewport.offset;
            let ndc = Vec2::new(
                screen.x - viewport.size.x / 2.0,
                -(screen.y - viewport.size.y / 2.0),
            );
            ndc / scale + viewport.camera_pos
        } else {
            self.mouse_pos
        }
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
                self.mouse_delta += new_pos - self.mouse_pos;
                self.mouse_pos = new_pos;
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
