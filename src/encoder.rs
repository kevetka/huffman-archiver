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
