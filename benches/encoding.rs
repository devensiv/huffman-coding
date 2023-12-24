use std::fs::OpenOptions;

use criterion::{criterion_group, criterion_main, Criterion};
use huffman::hencode;
use tempfile::tempfile;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("encode", |bencher| {
        bencher.iter(|| {
            let mut out = tempfile().expect("temfile err");
            let mut raw = OpenOptions::new()
                .read(true)
                .open("flake.lock")
                .expect("file err");
            hencode(&mut raw, &mut out).expect("io err");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);