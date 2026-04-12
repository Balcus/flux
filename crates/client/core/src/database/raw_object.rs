use crate::database::object_type::ObjectType;

pub struct RawObject {
    pub object_type: ObjectType,
    pub size: u32,
    pub data: Vec<u8>,
}

impl RawObject {
    pub fn new(object_type: ObjectType, size: u32, data: Vec<u8>) -> Self {
        RawObject {
            object_type,
            size,
            data,
        }
    }
}
