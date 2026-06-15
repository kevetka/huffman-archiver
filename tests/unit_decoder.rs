// SPDX-License-Identifier: MIT

use std::fs;

use huffman_archiver::decoder::decompress;
use huffman_archiver::encoder::compress;
use huffman_archiver::test_path;

#[test]
fn decompress_from_known_compress() {
    let input = test_path("dec_known_in.txt");
    let comp = test_path("dec_known.huf");
    let decomp = test_path("dec_known_out.txt");

    let data = b"Known data for decoder test";
    fs::write(&input, data).unwrap();
    compress(&input, &comp).unwrap();
    decompress(&comp, &decomp).unwrap();

    assert_eq!(fs::read(decomp).unwrap(), data);
}

#[test]
fn decompress_invalid_file() {
    let comp = test_path("dec_bad.huf");
    let decomp = test_path("dec_bad_out.txt");
    fs::write(&comp, b"not a valid huffman file").unwrap();
    assert!(decompress(&comp, &decomp).is_err());
}

#[test]
fn decompress_empty_compressed() {
    let comp = test_path("dec_empty.huf");
    let decomp = test_path("dec_empty_out.txt");
    fs::write(&comp, b"").unwrap();
    assert!(decompress(&comp, &decomp).is_err());
}

#[test]
fn decompress_truncated_payload() {
    let input = test_path("dec_trunc_in.txt");
    let comp = test_path("dec_trunc_comp.huf");
    let truncated = test_path("dec_trunc_cut.huf");
    let decomp = test_path("dec_trunc_out.txt");

    let data = b"This is a test file with enough data to have meaningful compressed content!";
    fs::write(&input, data).unwrap();
    compress(&input, &comp).unwrap();

    let compressed = fs::read(comp).unwrap();
    let header_size = 8 + 256;
    assert!(
        compressed.len() > header_size + 1,
        "compressed file must have data beyond header"
    );
    fs::write(&truncated, &compressed[..header_size + 1]).unwrap();

    assert!(
        decompress(&truncated, &decomp).is_err(),
        "truncated payload should fail"
    );
}

#[test]
fn decompress_header_only() {
    let input = test_path("dec_head_only_in.txt");
    let comp = test_path("dec_head_only_comp.huf");
    let truncated = test_path("dec_head_only_cut.huf");
    let decomp = test_path("dec_head_only_out.txt");

    let data = b"Some data for header-only truncation test";
    fs::write(&input, data).unwrap();
    compress(&input, &comp).unwrap();

    let compressed = fs::read(comp).unwrap();
    let header_size = 8 + 256;
    fs::write(&truncated, &compressed[..header_size.min(compressed.len())]).unwrap();

    assert!(
        decompress(&truncated, &decomp).is_err(),
        "header-only file should fail"
    );
}
