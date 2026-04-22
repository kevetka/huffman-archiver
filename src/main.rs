use std::io;

mod bitio;
mod encoder;
mod frequency;
mod tree;

use crate::encoder::compress;

fn main() -> io::Result<()> {
    let input_filename = "test_input.txt";
    let output_filename = "test_compressed.huf";

    let text = "Lorem ipsum dolor sit amet ut deserunt culpa id sunt dolore labore ex officia. 
                Qui pariatur eiusmod pariatur cillum ut dolore exercitation in ad elit. 
                Ex est irure minim aliquip. Anim sint in exercitation reprehenderit cupidatat magna velit. 
                Nostrud pariatur proident ad exercitation.";

    let large_text = text.repeat(100);

    std::fs::write(input_filename, &large_text)?;
    println!("Размер исходного файла: {} байт", large_text.len());

    match compress(input_filename, output_filename) {
        Ok(_) => {
            println!("Сжатие завершено успешно");

            let original_metadata = std::fs::metadata(input_filename)?;
            let compressed_metadata = std::fs::metadata(output_filename)?;

            let orig_size = original_metadata.len();
            let comp_size = compressed_metadata.len();

            println!("Исходный размер:   {} байт", orig_size);
            println!("Размер архива:     {} байт", comp_size);

            let ratio = (comp_size as f64 / orig_size as f64) * 100.0;
            println!("Степень сжатия:    {:.2}%", ratio);

            if comp_size < orig_size {
                println!("Файл уменьшился.");
            } else {
                println!("Файл увеличился.");
            }
        }
        Err(e) => {
            eprintln!("Ошибка при сжатии: {}", e);
        }
    }

    Ok(())
}
