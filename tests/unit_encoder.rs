// SPDX-License-Identifier: MIT

use std::fs;

use huffman_archiver::decoder::decompress;
use huffman_archiver::encoder::compress;
use huffman_archiver::test_path;

fn roundtrip(data: &[u8], name: &str) {
    let input = test_path(&format!("enc_{}_in.txt", name));
    let comp = test_path(&format!("enc_{}.huf", name));
    let decomp = test_path(&format!("enc_{}_out.txt", name));

    fs::write(&input, data).unwrap();
    compress(&input, &comp).unwrap();
    decompress(&comp, &decomp).unwrap();

    let result = fs::read(&decomp).unwrap();
    assert_eq!(data, &result, "roundtrip mismatch for {}", name);
}

#[test]
fn roundtrip_small_text() {
    roundtrip(b"Hello, Huffman!", "small_text");
}

#[test]
fn roundtrip_single_repeated() {
    roundtrip(&[0xAB; 100], "single_repeated");
}

#[test]
fn roundtrip_binary() {
    let data: Vec<u8> = (0..=255).cycle().take(512).collect();
    roundtrip(&data, "binary");
}

#[test]
fn roundtrip_all_255() {
    roundtrip(&[0xFF; 128], "all_255");
}

#[test]
fn roundtrip_empty_file() {
    let input = test_path("enc_empty_in.txt");
    let comp = test_path("enc_empty.huf");
    fs::write(&input, b"").unwrap();
    assert!(compress(&input, &comp).is_err());
}

#[test]
fn roundtrip_large_random() {
    let data: Vec<u8> = (0..10000).map(|i| (i ^ (i >> 4)) as u8).collect();
    roundtrip(&data, "large_random");
}

#[test]
fn roundtrip_single_repeated_1000() {
    roundtrip(&[0xAA; 1000], "single_1000");
}

#[test]
fn roundtrip_all_256_symbols() {
    let data: Vec<u8> = (0..=255).collect();
    roundtrip(&data, "all_256");
}

#[test]
fn roundtrip_single_byte() {
    roundtrip(b"\x42", "single_byte");
}
