use glam::{Quat, UVec2, Vec2, Vec3};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Field {
    NameString(&'static str, String),
    TransQuat(&'static str, Quat),
    String(&'static str, String),
    F32(&'static str, f32),
    I32(&'static str, i32),
    U32(&'static str, u32),
    Vec3(&'static str, Vec3),
    Vec2(&'static str, Vec2),
    UVec2(&'static str, UVec2),
    Quat(&'static str, Quat),
    Color(&'static str, [u8; 4]),
    Sprite(&'static str, Uuid),
    Audio(&'static str, Uuid),
}
