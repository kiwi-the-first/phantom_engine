use glam::{Quat, Vec3};

#[derive(Debug, Clone)]
pub enum Field {
    NameString(&'static str, String),
    TransQuat(&'static str, Quat),
    String(&'static str, String),
    F32(&'static str, f32),
    Vec3(&'static str, Vec3),
    Quat(&'static str, Quat),
    Color(&'static str, [u8; 4]),
}
