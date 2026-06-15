// SPDX-License-Identifier: MIT

use huffman_archiver::tree::*;

#[test]
fn empty_frequencies() {
    let freqs = [0u64; 256];
    assert!(build_huffman_tree(freqs).is_none());
}

#[test]
fn single_symbol() {
    let mut freqs = [0u64; 256];
    freqs[0x41] = 10;
    let tree = build_huffman_tree(freqs).unwrap();
    let lengths = get_code_lengths(&tree);
    assert_eq!(lengths[0x41], 1);
}

#[test]
fn two_symbols() {
    let mut freqs = [0u64; 256];
    freqs[0x41] = 5;
    freqs[0x42] = 5;
    let tree = build_huffman_tree(freqs).unwrap();
    let lengths = get_code_lengths(&tree);
    assert!(lengths[0x41] <= 2);
    assert!(lengths[0x42] <= 2);
    assert_eq!(lengths[0x41], lengths[0x42]);
}

#[test]
fn code_table_lengths_match() {
    let mut freqs = [0u64; 256];
    freqs[0x41] = 10;
    freqs[0x42] = 5;
    freqs[0x43] = 3;
    let tree = build_huffman_tree(freqs).unwrap();
    let lengths = get_code_lengths(&tree);
    let codes = code_table_generation(&tree);
    for (&symbol, code) in &codes {
        assert_eq!(lengths[symbol as usize] as usize, code.len());
    }
}

#[test]
fn canonical_roundtrip() {
    let mut freqs = [0u64; 256];
    freqs[b'a' as usize] = 10;
    freqs[b'b' as usize] = 5;
    freqs[b'c' as usize] = 3;
    freqs[b'd' as usize] = 2;
    freqs[b'e' as usize] = 1;

    let tree = build_huffman_tree(freqs).unwrap();
    let lengths = get_code_lengths(&tree);
    let canonical = build_canonical_codes(&lengths).unwrap();

    let reconstructed = build_tree_from_lengths(&lengths).unwrap();
    let reconstructed_lengths = get_code_lengths(&reconstructed);

    assert_eq!(lengths, reconstructed_lengths);
    assert_eq!(canonical.len(), 5);
}

#[test]
fn canonical_property() {
    let mut freqs = [0u64; 256];
    let text = b"this is a test string for huffman coding";
    for &b in text {
        freqs[b as usize] += 1;
    }
    let tree = build_huffman_tree(freqs).unwrap();
    let lengths = get_code_lengths(&tree);
    let canonical = build_canonical_codes(&lengths).unwrap();

    let mut prev: Option<(u8, &Vec<bool>)> = None;
    let mut sorted: Vec<_> = canonical.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    for &(sym, code) in &sorted {
        if let Some((_, prev_code)) = prev {
            if code.len() == prev_code.len() {
                let prev_val = prev_code.iter().fold(0u64, |acc, &b| (acc << 1) | b as u64);
                let cur_val = code.iter().fold(0u64, |acc, &b| (acc << 1) | b as u64);
                assert!(cur_val > prev_val, "codes must be sequential");
            }
        }
        prev = Some((*sym, code));
    }
}

#[test]
fn empty_codes() {
    let lengths = [0u8; 256];
    let table = build_canonical_codes(&lengths).unwrap();
    assert!(table.is_empty());
}

#[test]
fn prefix_property() {
    let mut freqs = [0u64; 256];
    let text = b"test data with various symbols";
    for &b in text {
        freqs[b as usize] += 1;
    }
    let tree = build_huffman_tree(freqs).unwrap();
    let codes = code_table_generation(&tree);

    for (_, code1) in &codes {
        for (_, code2) in &codes {
            if code1 == code2 {
                continue;
            }
            let min_len = code1.len().min(code2.len());
            if &code1[..min_len] == &code2[..min_len] {
                assert_eq!(code1.len(), code2.len(), "prefix violation");
            }
        }
    }
}

#[test]
fn build_tree_invalid_lengths_kraft() {
    let mut lengths = [0u8; 256];
    lengths[0] = 1;
    lengths[1] = 1;
    lengths[2] = 1;
    let tree = build_tree_from_lengths(&lengths);
    assert!(
        tree.is_none(),
        "build_tree_from_lengths should reject Kraft violation"
    );
    let codes = build_canonical_codes(&lengths);
    assert!(codes.is_none());
}

#[test]
fn build_tree_from_empty_lengths() {
    let lengths = [0u8; 256];
    assert!(build_tree_from_lengths(&lengths).is_none());
}

#[test]
fn build_tree_single_symbol() {
    let mut lengths = [0u8; 256];
    lengths[0x42] = 1;
    let tree = build_tree_from_lengths(&lengths).unwrap();
    let result_lengths = get_code_lengths(&tree);
    assert_eq!(result_lengths[0x42], 1);
}

#[test]
fn build_tree_all_lengths_present() {
    let mut lengths = [0u8; 256];
    for i in 0..128 {
        lengths[i] = 8;
    }
    let tree = build_tree_from_lengths(&lengths).unwrap();
    assert!(tree.left.is_some() || tree.right.is_some());
}

#[test]
fn build_canonical_rejects_len_128() {
    let mut lengths = [0u8; 256];
    lengths[0] = 128;
    assert!(build_canonical_codes(&lengths).is_none());
}

#[test]
fn build_canonical_len_127_works() {
    let mut lengths = [0u8; 256];
    lengths[0] = 1;
    lengths[1] = 127;
    let codes = build_canonical_codes(&lengths).unwrap();
    assert_eq!(codes.len(), 2);
    assert_eq!(codes[&0].len(), 1);
    assert_eq!(codes[&1].len(), 127);
}
