use crate::ecs::{AnyStorage, Component, SparseSet};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Name {
    pub name: String,
}

impl Component for Name {
    const NAME: &'static str = "Name";
}

#[::ctor::ctor]
fn __register_name() {
    crate::ecs::component_registry::register_component("Name", __deserialize_name);
}

fn __deserialize_name(data: &[u8]) -> Box<dyn AnyStorage> {
    Box::new(bincode::deserialize::<SparseSet<Name>>(data).unwrap())
}

impl Default for Name {
    fn default() -> Self {
        Self {
            name: "empty entity".to_string(),
        }
    }
}

impl Name {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
