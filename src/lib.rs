// SPDX-License-Identifier: MIT

//! # Huffman Archiver
//!
//! Реализация архиватора на основе канонического алгоритма Хаффмана.
//!
//! ## Модули
//! - `frequency` — подсчёт частот байт во входном файле
//! - `tree` — построение дерева Хаффмана, канонических кодов и восстановление дерева
//! - `bitio` — побитовое чтение и запись
//! - `encoder` — сжатие (compress)
//! - `decoder` — распаковка (decompress)

pub mod bitio;
pub mod decoder;
pub mod encoder;
pub mod frequency;
pub mod tree;

/// Возвращает путь к временному файлу в системной temp-директории.
pub fn test_path(name: &str) -> String {
    std::env::temp_dir()
        .join(name)
        .to_string_lossy()
        .to_string()
}
