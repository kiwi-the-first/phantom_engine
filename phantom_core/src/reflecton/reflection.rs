use crate::reflecton::fields::Field;

pub trait Reflection {
    fn get_fields(&self) -> Vec<Field>;
    fn set_feilds(&mut self, fields: Vec<Field>);
}
