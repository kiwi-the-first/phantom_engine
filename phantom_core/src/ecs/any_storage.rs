use std::any::Any;
pub trait AnyStorage: Any {
    fn serialize(&self) -> Vec<u8>;
    fn remove(&mut self, entity_id: u32);
    //fn has(&mut self, entity_id: u32) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
