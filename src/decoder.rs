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

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::decoder::decompress;
    use crate::encoder::compress;
    use crate::test_path;

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
}
