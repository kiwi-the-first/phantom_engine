use std::any::Any;

use crate::reflecton::fields::Field;

pub trait AnyStorage: Any + Send {
    fn serialize(&self) -> Vec<u8>;
    fn remove(&mut self, entity_id: u32);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn has(&self, entity_id: u32) -> bool;
    fn get_feilds(&self, entity_id: u32) -> Vec<Field>;
    fn set_fields(&mut self, entity_id: u32, fields: Vec<Field>);
}
