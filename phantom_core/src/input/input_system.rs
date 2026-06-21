use glam::Vec2;
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::PhysicalKey,
};

use crate::input::{InputContext, input_context::ViewportInfo};

pub struct InputSystem {
    pub input_ctx: InputContext,
    // DPI scale factor — keeps mouse_pos in logical pixels to match egui/viewport coords.
    scale_factor: f32,
}

impl Default for InputSystem {
    fn default() -> Self {
        Self {
            input_ctx: InputContext::default(),
            scale_factor: 1.0,
        }
    }
}

impl InputSystem {
    pub fn set_scale_factor(&mut self, sf: f64) {
        self.scale_factor = sf as f32;
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = *scale_factor as f32;
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    match key_event.state {
                        ElementState::Pressed => {
                            if !key_event.repeat && self.input_ctx.keys_held.insert(code) {
                                self.input_ctx.keys_pressed.insert(code);
                            }
                        }
                        ElementState::Released => {
                            if self.input_ctx.keys_held.remove(&code) {
                                self.input_ctx.keys_released.insert(code);
                            }
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                // CursorMoved gives physical pixels; divide by scale_factor to match
                // the logical-pixel space used by egui (editor) and viewport info.
                let new_pos = Vec2::new(
                    position.x as f32 / self.scale_factor,
                    position.y as f32 / self.scale_factor,
                );
                // Only accumulate delta once we have a previous position, so the
                // first event doesn't report a huge spurious jump.
                if let Some(prev) = self.input_ctx.mouse_pos {
                    self.input_ctx.mouse_delta += new_pos - prev;
                }
                self.input_ctx.mouse_pos = Some(new_pos);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if self.input_ctx.mouse_held.insert(*button) {
                        self.input_ctx.mouse_pressed.insert(*button);
                    }
                }
                ElementState::Released => {
                    if self.input_ctx.mouse_held.remove(button) {
                        self.input_ctx.mouse_released.insert(*button);
                    }
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.input_ctx.scroll_delta += Vec2::new(*x, *y);
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        const PIXELS_PER_LINE: f32 = 50.0; // rough convention
                        self.input_ctx.scroll_delta +=
                            Vec2::new(p.x as f32, p.y as f32) / PIXELS_PER_LINE;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn set_viewport(&mut self, info: ViewportInfo) {
        self.input_ctx.viewport = Some(info);
    }

    pub fn end_frame(&mut self) {
        self.input_ctx.keys_pressed.clear();
        self.input_ctx.keys_released.clear();
        self.input_ctx.mouse_pressed.clear();
        self.input_ctx.mouse_released.clear();
        self.input_ctx.mouse_delta = Vec2::ZERO;
        self.input_ctx.scroll_delta = Vec2::ZERO;
    }
}
