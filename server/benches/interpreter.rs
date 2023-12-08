use std::{
    fs::File,
    io::{BufReader, Read},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lykia::runtime::{Runtime, RuntimeMode};

fn runtime(filename: &str) {
    let file = File::open(filename).unwrap();
    let mut content: String = String::new();
    BufReader::new(file)
        .read_to_string(&mut content)
        .expect("File couldn't be read.");
    let mut runtime = Runtime::new(RuntimeMode::File);
    runtime.interpret(&content);
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    // group.bench_function("For", |b| b.iter(|| runtime(black_box("benches/scripts/while.ly"))));
    group.bench_function("Fibonacci 35", |b| {
        b.iter(|| runtime(black_box("benches/scripts/fib.ly")))
    });
    // group.bench_function("While", |b| b.iter(|| runtime(black_box("benches/scripts/while.ly"))));
    // group.bench_function("While (Short)", |b| b.iter(|| runtime(black_box("benches/scripts/while.short.ly"))));
    group.finish();
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = bench
}

criterion_main!(benches);
