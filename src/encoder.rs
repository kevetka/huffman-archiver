use std::{
    collections::HashMap,
    fs::{File, metadata},
    io::{self, BufReader, Read},
};

use haffman_archiver::{
    bitio::BitWriter,
    tree::{build_huffman_tree, code_table_generation},
};

use crate::frequency;

pub fn compress(input_path: &str, output_path: &str) -> io::Result<()> {
    let frequencies = frequency::count_frequencies(input_path)?;

    let file_size = metadata(input_path)?.len();

    let codes = if let Some(root) = build_huffman_tree(frequencies) {
        code_table_generation(&root)
    } else {
        HashMap::new()
    };

    let output_file = File::create(output_path)?;
    let mut writer = BitWriter::new(output_file);

    let size_bytes = file_size.to_be_bytes();
    for &byte in &size_bytes {
        writer.write_byte(byte)?;
    }

    for &freq in frequencies.iter() {
        let freq_bytes = freq.to_be_bytes();
        for &byte in &freq_bytes {
            writer.write_byte(byte)?;
        }
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

    Ok(())
}
