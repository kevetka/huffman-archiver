// SPDX-License-Identifier: MIT

use std::{fs, process::Command};

fn tp(name: &str) -> String {
    std::env::temp_dir()
        .join(name)
        .to_string_lossy()
        .to_string()
}

fn binary_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("haffman-archiver");
    path.to_str().unwrap().to_string()
}

#[test]
fn cli_compress_decompress_text() {
    let input = tp("int_text_in.txt");
    let comp = tp("int_text_comp.huf");
    let decomp = tp("int_text_decomp.txt");

    fs::write(&input, b"Integration test for Huffman archiver CLI").unwrap();

    let bin = binary_path();
    let output = Command::new(&bin)
        .args(["compress", &input, &comp])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "compress failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(&bin)
        .args(["decompress", &comp, &decomp])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "decompress failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let original = fs::read(input).unwrap();
    let result = fs::read(decomp).unwrap();
    assert_eq!(original, result, "decompressed data doesn't match original");
}

#[test]
fn cli_compress_decompress_binary() {
    let input = tp("int_bin_in.bin");
    let comp = tp("int_bin_comp.huf");
    let decomp = tp("int_bin_decomp.bin");

    let data: Vec<u8> = (0..=255).cycle().take(1024).collect();
    fs::write(&input, &data).unwrap();

    let bin = binary_path();
    let output = Command::new(&bin)
        .args(["compress", &input, &comp])
        .output()
        .unwrap();
    assert!(output.status.success());

    let output = Command::new(&bin)
        .args(["decompress", &comp, &decomp])
        .output()
        .unwrap();
    assert!(output.status.success());

    let result = fs::read(decomp).unwrap();
    assert_eq!(data, result);
}

#[test]
fn cli_wrong_args() {
    let bin = binary_path();
    let output = Command::new(&bin).arg("invalid_command").output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn cli_missing_args() {
    let bin = binary_path();
    let output = Command::new(&bin).args(["compress"]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn cli_nonexistent_input() {
    let bin = binary_path();
    let nonexistent = tp("nonexistent_file_12345.txt");
    let out = tp("out.huf");
    let output = Command::new(&bin)
        .args(["compress", &nonexistent, &out])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn cli_decompress_missing_args() {
    let bin = binary_path();
    let output = Command::new(&bin).args(["decompress"]).output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn cli_decompress_nonexistent_input() {
    let bin = binary_path();
    let nonexistent = tp("nonexistent_file_12345.huf");
    let out = tp("out.txt");
    let output = Command::new(&bin)
        .args(["decompress", &nonexistent, &out])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn cli_decompress_invalid_file() {
    let input = tp("int_bad.huf");
    let output_path = tp("int_bad_out.txt");
    fs::write(&input, b"this is not a valid huffman archive").unwrap();

    let bin = binary_path();
    let result = Command::new(&bin)
        .args(["decompress", &input, &output_path])
        .output()
        .unwrap();
    assert!(!result.status.success());
}
