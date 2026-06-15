// SPDX-License-Identifier: MIT

use std::fs::{self, File};

use huffman_archiver::bitio::{BitReader, BitWriter};
use huffman_archiver::test_path;

fn make_writer(path: &str) -> BitWriter {
    BitWriter::new(File::create(path).unwrap())
}

fn make_reader(path: &str) -> BitReader {
    BitReader::new(File::open(path).unwrap())
}

fn collect_all_bits(r: &mut BitReader) -> Vec<bool> {
    let mut bits = Vec::new();
    while let Some(bit) = r.read_bit().unwrap() {
        bits.push(bit);
    }
    bits
}

fn write_all_bits(w: &mut BitWriter, bits: &[bool]) {
    for &b in bits {
        w.write_bit(b).unwrap();
    }
}

#[test]
fn write_read_bits() {
    let path = test_path("test_bits.huf");
    {
        let mut w = make_writer(&path);
        w.write_bit(true).unwrap();
        w.write_bit(false).unwrap();
        w.write_bit(true).unwrap();
        w.write_bit(true).unwrap();
        w.write_bit(false).unwrap();
        w.write_bit(false).unwrap();
        w.write_bit(true).unwrap();
        w.write_bit(false).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        assert_eq!(r.read_bit().unwrap(), Some(true));
        assert_eq!(r.read_bit().unwrap(), Some(false));
        assert_eq!(r.read_bit().unwrap(), Some(true));
        assert_eq!(r.read_bit().unwrap(), Some(true));
        assert_eq!(r.read_bit().unwrap(), Some(false));
        assert_eq!(r.read_bit().unwrap(), Some(false));
        assert_eq!(r.read_bit().unwrap(), Some(true));
        assert_eq!(r.read_bit().unwrap(), Some(false));
        assert_eq!(r.read_bit().unwrap(), None);
    }
}

#[test]
fn write_byte_aligned() {
    let path = test_path("test_aligned.huf");
    {
        let mut w = make_writer(&path);
        w.write_byte(0xAB).unwrap();
        w.write_byte(0xCD).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        assert_eq!(r.read_byte().unwrap(), Some(0xAB));
        assert_eq!(r.read_byte().unwrap(), Some(0xCD));
        assert_eq!(r.read_byte().unwrap(), None);
    }
}

#[test]
fn bits_roundtrip() {
    let path = test_path("test_bits_rt.huf");
    let input: Vec<bool> = vec![true, false, true, true, false, false, true, false];
    {
        let mut w = make_writer(&path);
        write_all_bits(&mut w, &input);
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        let output = collect_all_bits(&mut r);
        assert_eq!(input, output);
    }
}

#[test]
fn bytes_aligned_roundtrip() {
    let path = test_path("test_bytes_rt.huf");
    let input: Vec<u8> = vec![0xAB, 0xCD, 0xEF, 0x01, 0xFF, 0x00];
    {
        let mut w = make_writer(&path);
        for &b in &input {
            w.write_byte(b).unwrap();
        }
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        for &b in &input {
            assert_eq!(r.read_byte().unwrap(), Some(b));
        }
        assert_eq!(r.read_byte().unwrap(), None);
    }
}

#[test]
fn mixed_bits_bytes_roundtrip() {
    let path = test_path("test_mixed_rt.huf");
    let input_bits: Vec<bool> = vec![true, false, true, true, false];
    let input_bytes: Vec<u8> = vec![0xAA, 0x55];
    {
        let mut w = make_writer(&path);
        write_all_bits(&mut w, &input_bits);
        for &b in &input_bytes {
            w.write_byte(b).unwrap();
        }
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        let all_bits = collect_all_bits(&mut r);
        let mut expected_bits: Vec<bool> = input_bits.clone();
        for &b in &input_bytes {
            for i in 0..8 {
                expected_bits.push((b >> (7 - i)) & 1 != 0);
            }
        }
        assert!(
            all_bits.len() >= expected_bits.len(),
            "got {} bits, expected at least {}",
            all_bits.len(),
            expected_bits.len()
        );
        assert_eq!(
            &all_bits[..expected_bits.len()],
            &expected_bits,
            "first {} bits should match",
            expected_bits.len()
        );
    }
}

#[test]
fn partial_byte_flush() {
    let path = test_path("test_partial_flush.huf");
    {
        let mut w = make_writer(&path);
        w.write_bit(true).unwrap();
        w.write_bit(false).unwrap();
        w.write_bit(true).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        assert_eq!(r.read_bit().unwrap(), Some(true));
        assert_eq!(r.read_bit().unwrap(), Some(false));
        assert_eq!(r.read_bit().unwrap(), Some(true));
        let remaining = collect_all_bits(&mut r);
        assert_eq!(remaining.len(), 5);
        assert!(remaining.iter().all(|&b| !b));
    }
}

#[test]
fn multiple_flushes() {
    let path = test_path("test_multi_flush.huf");
    {
        let mut w = make_writer(&path);
        w.write_bit(true).unwrap();
        w.flush().unwrap();
        w.write_bit(false).unwrap();
        w.flush().unwrap();
        w.write_bit(true).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        let bits = collect_all_bits(&mut r);
        assert_eq!(bits[0], true);
        assert_eq!(bits[8], false);
        assert_eq!(bits[16], true);
    }
}

#[test]
fn read_bit_empty_file() {
    let path = test_path("test_empty.huf");
    fs::write(&path, b"").unwrap();
    let mut r = make_reader(&path);
    assert_eq!(r.read_bit().unwrap(), None);
}

#[test]
fn read_bit_eof_after_full_byte() {
    let path = test_path("test_eof_byte.huf");
    {
        let mut w = make_writer(&path);
        w.write_byte(0xAB).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        for _ in 0..8 {
            assert!(r.read_bit().unwrap().is_some());
        }
        assert_eq!(r.read_bit().unwrap(), None);
    }
}

#[test]
fn read_bit_eof_mid_byte() {
    let path = test_path("test_eof_mid.huf");
    {
        let mut w = make_writer(&path);
        w.write_bit(true).unwrap();
        w.write_bit(false).unwrap();
        w.write_bit(true).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        assert_eq!(r.read_bit().unwrap(), Some(true));
        assert_eq!(r.read_bit().unwrap(), Some(false));
        assert_eq!(r.read_bit().unwrap(), Some(true));
        let remaining = collect_all_bits(&mut r);
        assert_eq!(remaining.len(), 5);
        assert!(remaining.iter().all(|&b| !b), "padding bits should be 0");
    }
}

#[test]
fn read_byte_empty_file() {
    let path = test_path("test_empty_byte.huf");
    fs::write(&path, b"").unwrap();
    let mut r = make_reader(&path);
    assert_eq!(r.read_byte().unwrap(), None);
}

#[test]
fn read_byte_eof_mid_read() {
    let path = test_path("test_eof_mid_byte.huf");
    fs::write(&path, b"\xAB").unwrap();
    let mut r = make_reader(&path);
    assert!(r.read_byte().unwrap().is_some());
    assert_eq!(r.read_byte().unwrap(), None);
}

#[test]
fn write_byte_misaligned_recover() {
    let path = test_path("test_misaligned_recover.huf");
    {
        let mut w = make_writer(&path);
        w.write_bit(true).unwrap();
        w.write_byte(0xAA).unwrap();
        w.write_bit(false).unwrap();
        w.flush().unwrap();
    }
    {
        let mut r = make_reader(&path);
        let bits = collect_all_bits(&mut r);
        assert_eq!(bits[0], true);
        assert_eq!(bits[1], true);
        assert_eq!(bits[2], false);
        assert_eq!(bits[3], true);
        assert_eq!(bits[4], false);
        assert_eq!(bits[5], true);
        assert_eq!(bits[6], false);
        assert_eq!(bits[7], true);
        assert_eq!(bits[8], false);
        assert_eq!(bits[9], false);
    }
}
