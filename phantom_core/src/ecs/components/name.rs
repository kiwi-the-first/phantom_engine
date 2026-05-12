use crate::ecs::{AnyStorage, Component, SparseSet};
use crate::reflecton::Reflection;
use crate::reflecton::fields::Field;
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Name {
    pub name: String,
}

impl Reflection for Name {
    fn get_fields(&self) -> Vec<Field> {
        vec![Field::NameString("name", self.name.clone())]
    }
    fn set_feilds(&mut self, fields: Vec<Field>) {
        match fields.get(0).unwrap() {
            Field::NameString(name, name_field) => {
                self.name = name_field.to_string();
            }
            _ => {}
        };
    }
}

impl Component for Name {
    const NAME: &'static str = "Name";
}

#[ctor::ctor]
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
