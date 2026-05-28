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
}

impl Input {
    pub fn is_down(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_held.contains(&button)
    }
    pub fn mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }
    pub fn mouse_just_released(&self, button: MouseButton) -> bool {
        self.mouse_released.contains(&button)
    }
    pub fn mouse_pos(&self) -> Vec2 {
        self.mouse_pos
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

    pub fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_pressed.clear();
        self.mouse_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = Vec2::ZERO;
    }
}
