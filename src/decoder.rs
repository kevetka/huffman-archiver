// SPDX-License-Identifier: MIT

use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use crate::{bitio::BitReader, tree::build_tree_from_lengths};

/// Распаковывает файл, сжатый `compress`.
///
/// Читает заголовок (8 байт размера + 256 байт длин кодов),
/// восстанавливает дерево, побитово декодирует данные и записывает
/// результат в выходной файл.
pub fn decompress(input_path: &str, output_path: &str) -> io::Result<()> {
    let input_file = File::open(input_path)?;
    let mut reader = BitReader::new(input_file);
    let mut output = BufWriter::new(File::create(output_path)?);

    let mut size_bytes = [0u8; 8];
    for byte in &mut size_bytes {
        *byte = reader.read_byte()?.ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected EOF while reading file size",
        ))?;
    }
    let file_size = u64::from_be_bytes(size_bytes);

    let mut code_lengths = [0u8; 256];
    for byte in code_lengths.iter_mut() {
        *byte = reader.read_byte()?.ok_or(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected EOF while reading code lengths",
        ))?;
    }

    let root = build_tree_from_lengths(&code_lengths).ok_or(io::Error::new(
        io::ErrorKind::InvalidData,
        "Failed to build Huffman tree from code lengths",
    ))?;

    let mut remaining = file_size;
    while remaining > 0 {
        let mut current = &root;
        loop {
            match reader.read_bit()? {
                Some(bit) => {
                    current = if !bit {
                        current.left.as_ref().ok_or(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid bit sequence: missing left child",
                        ))?
                    } else {
                        current.right.as_ref().ok_or(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid bit sequence: missing right child",
                        ))?
                    };

                    if let Some(symbol) = current.symbol {
                        output.write_all(&[symbol])?;
                        remaining -= 1;
                        break;
                    }
                }
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Unexpected EOF while reading compressed data",
                    ));
                }
            }
        }
    }

    output.flush()?;
    Ok(())
}
