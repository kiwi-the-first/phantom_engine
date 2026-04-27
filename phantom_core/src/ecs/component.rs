use std::any::Any;

pub trait Component: Any + 'static {
    const NAME: &'static str;
}
