// SPDX-License-Identifier: MIT

use std::fs;

use criterion::{Criterion, criterion_group, criterion_main};

fn tp(name: &str) -> String {
    std::env::temp_dir()
        .join(name)
        .to_string_lossy()
        .to_string()
}

fn generate_data(data_type: &str, size: usize) -> Vec<u8> {
    match data_type {
        "english" => {
            let text = "The quick brown fox jumps over the lazy dog. Huffman coding assigns shorter codes to more frequent symbols. This is a sample English text for compression testing purposes. ";
            text.as_bytes().iter().cycle().take(size).cloned().collect()
        }
        "repeated" => vec![0xAA; size],
        "all_bytes" => (0..=255).cycle().take(size).collect(),
        "random" => {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut data = Vec::with_capacity(size);
            for i in 0..size {
                let mut h = DefaultHasher::new();
                i.hash(&mut h);
                data.push(h.finish() as u8);
            }
            data
        }
        _ => vec![0u8; size],
    }
}

fn bench_compress(c: &mut Criterion) {
    let data_types = &["english", "repeated", "all_bytes", "random"];
    let sizes = [100, 1000, 10000, 100000];

    for &size in &sizes {
        for &data_type in data_types {
            let data = generate_data(data_type, size);
            let input = tp(&format!("bench_compress_{}_{}_in.txt", data_type, size));
            let comp = tp(&format!("bench_compress_{}_{}.huf", data_type, size));
            fs::write(&input, &data).unwrap();

            c.bench_function(&format!("compress/{}/{}", data_type, size), |b| {
                b.iter(|| {
                    huffman_archiver::encoder::compress(&input, &comp).unwrap();
                })
            });
        }
    }
}

fn bench_decompress(c: &mut Criterion) {
    let data_types = &["english", "repeated", "all_bytes", "random"];
    let sizes = [100, 1000, 10000, 100000];

    for &size in &sizes {
        for &data_type in data_types {
            let data = generate_data(data_type, size);
            let input = tp(&format!("bench_decompress_{}_{}_in.txt", data_type, size));
            let comp = tp(&format!("bench_decompress_{}_{}.huf", data_type, size));
            let decomp = tp(&format!("bench_decompress_{}_{}_out.txt", data_type, size));
            fs::write(&input, &data).unwrap();
            huffman_archiver::encoder::compress(&input, &comp).unwrap();

            c.bench_function(&format!("decompress/{}/{}", data_type, size), |b| {
                b.iter(|| {
                    huffman_archiver::decoder::decompress(&comp, &decomp).unwrap();
                    let _ = fs::read(&decomp).unwrap();
                })
            });
        }
    }
}

fn bench_compress_ratio(c: &mut Criterion) {
    let tests: &[(&str, usize)] = &[
        ("english", 1000),
        ("english", 100000),
        ("repeated", 1000),
        ("repeated", 100000),
        ("all_bytes", 1000),
        ("all_bytes", 100000),
        ("random", 1000),
        ("random", 100000),
        ("tiny_english", 10),
        ("tiny_repeated", 5),
    ];

    println!("\n=== Коэффициенты сжатия ===");
    println!(
        "{:<20} {:>10} {:>12} {:>8}",
        "Тип", "Исходный", "Сжатый", "Ratio"
    );
    println!("{:-<52}", "");

    for &(name, size) in tests {
        let data = generate_data(name, size);
        let input = tp(&format!("bench_ratio_{}_{}_in.txt", name, size));
        let comp = tp(&format!("bench_ratio_{}_{}.huf", name, size));
        fs::write(&input, &data).unwrap();
        huffman_archiver::encoder::compress(&input, &comp).unwrap();

        let orig_size = fs::metadata(&input).unwrap().len();
        let comp_size = fs::metadata(&comp).unwrap().len();
        let ratio = comp_size as f64 / orig_size as f64;

        println!(
            "{:<20} {:>10} {:>12} {:>7.2}%",
            name,
            orig_size,
            comp_size,
            ratio * 100.0
        );
    }
    println!();

    c.bench_function("ratio_dummy", |b| b.iter(|| std::hint::black_box(1 + 1)));
}

criterion_group!(
    benches,
    bench_compress,
    bench_decompress,
    bench_compress_ratio
);
criterion_main!(benches);
