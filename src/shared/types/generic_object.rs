use crate::shared::types::object_type::ObjectType;

pub struct GenericObject {
    pub object_type: ObjectType,
    pub size: usize,
    pub decompressed_content: Vec<u8>,
}
