use flux::utils::{compress, decompress, hash};

#[test]
fn hash_test() {
    let data = b"Hello World!".to_vec();
    let hash = hash(&data).unwrap();

    assert_eq!(
        hash,
        "2ef7bde608ce5404e97d5f042f95f89f1c232871"
    );
}

#[test]
fn compression_decompression_test() {
    let data = b"Hello World!".to_vec();
    let compressed = compress(&data).unwrap();
    let decompressed = decompress(compressed).unwrap();
    assert_eq!(data, decompressed);
}
