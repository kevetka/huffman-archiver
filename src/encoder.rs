// SPDX-License-Identifier: MIT

use std::{
    fs::{File, metadata},
    io::{self, BufReader, Read},
};

use crate::{
    bitio::BitWriter,
    frequency,
    tree::{build_canonical_codes, build_huffman_tree, get_code_lengths},
};

/// Сжимает файл по алгоритму Хаффмана.
///
/// Формат архива:
/// - 8 байт: исходный размер файла (big-endian u64)
/// - 256 байт: длины канонических кодов для каждого байта (0 = символ отсутствует)
/// - битовый поток: закодированные данные (с выравнивающим паддингом)
pub fn compress(input_path: &str, output_path: &str) -> io::Result<()> {
    let frequencies = frequency::count_frequencies(input_path)?;

    let file_size = metadata(input_path)?.len();

    let root = build_huffman_tree(frequencies).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Empty input file",
    ))?;

    let code_lengths = get_code_lengths(&root);
    let codes = build_canonical_codes(&code_lengths).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Failed to build canonical codes",
    ))?;

    let output_file = File::create(output_path)?;
    let mut writer = BitWriter::new(output_file);

    let size_bytes = file_size.to_be_bytes();
    for &byte in &size_bytes {
        writer.write_byte(byte)?;
    }

    for &len in code_lengths.iter() {
        writer.write_byte(len)?;
    }

    if file_size > 0 {
        let input_file = File::open(input_path)?;
        let mut reader = BufReader::new(input_file);

        let mut buffer = [0u8; 4096];

        loop {
            let bytes_red = reader.read(&mut buffer)?;
            if bytes_red == 0 {
                break;
            }

            for &byte in &buffer[..bytes_red] {
                if let Some(code) = codes.get(&byte) {
                    for &bit in code {
                        writer.write_bit(bit)?;
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Symbol not found in Huffman tree",
                    ));
                }
            }
        }
    }

    writer.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::decoder::decompress;
    use crate::encoder::compress;
    use crate::test_path;

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
}
