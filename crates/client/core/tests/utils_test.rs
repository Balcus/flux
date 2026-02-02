use flux_core::utils::{compress, decompress, hash};

#[test]
fn hash_empty_data() {
    let data = b"".to_vec();
    let result = hash(&data);
    assert_eq!(result, "da39a3ee5e6b4b0d3255bfef95601890afd80709");
}

#[test]
fn hash_simple_string() {
    let data = b"Hello World!".to_vec();
    let result = hash(&data);
    assert_eq!(result, "2ef7bde608ce5404e97d5f042f95f89f1c232871");
}

#[test]
fn hash_multiline_text() {
    let data = b"line1\nline2\nline3".to_vec();
    let result = hash(&data);
    assert_eq!(result.len(), 40);
    assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn hash_consistency() {
    let data = b"test data".to_vec();
    let hash1 = hash(&data);
    let hash2 = hash(&data);
    assert_eq!(hash1, hash2);
}

#[test]
fn compress_decompress_empty() {
    let data = b"".to_vec();
    let compressed = compress(&data);
    let decompressed = decompress(compressed);
    assert_eq!(data, decompressed);
}

#[test]
fn compress_decompress_simple_string() {
    let data = b"Hello World!".to_vec();
    let compressed = compress(&data);
    let decompressed = decompress(compressed);
    assert_eq!(data, decompressed);
}

#[test]
fn compress_decompress_large_text() {
    let data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
        .repeat(100)
        .into_bytes();
    let compressed = compress(&data);
    let decompressed = decompress(compressed.clone());
    assert_eq!(data, decompressed);
    assert!(compressed.len() < data.len());
}

#[test]
fn compress_decompress_binary_data() {
    let data: Vec<u8> = (0..=255).collect();
    let compressed = compress(&data);
    let decompressed = decompress(compressed);
    assert_eq!(data, decompressed);
}

#[test]
fn compress_decompress_unicode() {
    let data = "Hello 世界! 🌍".as_bytes().to_vec();
    let compressed = compress(&data);
    let decompressed = decompress(compressed);
    assert_eq!(data, decompressed);
}

#[test]
fn compress_decompress_newlines() {
    let data = b"line1\nline2\nline3\n".to_vec();
    let compressed = compress(&data);
    let decompressed = decompress(compressed);
    assert_eq!(data, decompressed);
}

#[test]
fn compress_decompress_null_bytes() {
    let data = vec![0x00, 0x01, 0x00, 0x02, 0x00];
    let compressed = compress(&data);
    let decompressed = decompress(compressed);
    assert_eq!(data, decompressed);
}
