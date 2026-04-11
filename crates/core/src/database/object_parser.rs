use crate::database::{object_type::ObjectType, raw_object::RawObject};

pub fn parse(decompressed_data: Vec<u8>) -> anyhow::Result<RawObject> {
    let null_pos = decompressed_data
        .iter()
        .position(|&b| b == b'\0')
        .ok_or_else(|| anyhow::anyhow!("Invalid object format: no null byte found."))?;

    let header = String::from_utf8(decompressed_data[..null_pos].to_vec())?;
    let parts: Vec<&str> = header.split(' ').collect();

    if parts.len() != 2 {
        anyhow::bail!("Invalid object format: invalid header.");
    }

    let object_type = match parts[0] {
        "blob" => ObjectType::Blob,
        "tree" => ObjectType::Tree,
        "commit" => ObjectType::Commit,
        _ => anyhow::bail!("Invalid object format: unsupported object type."),
    };

    let size: u32 = parts[1].parse()?;
    let content = decompressed_data[null_pos + 1..].to_vec();

    if (content.len() as u32) != size {
        anyhow::bail!("Invalid object format: length of content does not match size field");
    }

    Ok(RawObject::new(object_type, size, content))
}
