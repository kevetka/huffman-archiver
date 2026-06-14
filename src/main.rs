// SPDX-License-Identifier: MIT

use std::{env, fs, io, process};

use haffman_archiver::decoder::decompress;
use haffman_archiver::encoder::compress;

/// Выводит справку по использованию архиватора.
fn print_usage() {
    eprintln!("Использование:");
    eprintln!("  compress <input> <output>   — сжать файл");
    eprintln!("  decompress <input> <output> — распаковать файл");
}

/// Точка входа: разбирает аргументы командной строки и запускает `compress`
/// или `decompress`.
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    match command.as_str() {
        "compress" => {
            let orig_size = fs::metadata(input_path)?.len();
            compress(input_path, output_path)?;

            let comp_size = fs::metadata(output_path)?.len();
            let ratio = comp_size as f64 / orig_size as f64 * 100.0;

            println!("Исходный размер: {} байт", orig_size);
            println!("Размер архива:   {} байт", comp_size);
            println!("Степень сжатия:  {:.2}%", ratio);
        }
        "decompress" => {
            decompress(input_path, output_path)?;
            println!("Распаковка завершена");
        }
        _ => {
            eprintln!("Неизвестная команда: {}", command);
            print_usage();
            process::exit(1);
        }
    }

    Ok(())
}
