use std::{any::Any, fmt};
use crate::objects::object_type::ObjectType;

pub trait Object: fmt::Display {
    fn object_type(&self) -> ObjectType;
    fn id(&self) -> String;
    fn serialize(&self) -> Vec<u8>;
    fn as_any(&self) -> &dyn Any;
    fn content(&self) -> Vec<u8>;
}