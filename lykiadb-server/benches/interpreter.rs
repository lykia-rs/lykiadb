use std::{
    fs::File,
    io::{BufReader, Read},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lykiadb_server::engine::{interpreter::Interpreter, Runtime, RuntimeMode};

fn runtime(filename: &str) {
    let file = File::open(filename).unwrap();
    let mut content: String = String::new();
    BufReader::new(file)
        .read_to_string(&mut content)
        .expect("File couldn't be read.");
    let mut runtime = Runtime::new(RuntimeMode::File, Interpreter::new(None, true));
    runtime.interpret(&content).unwrap();
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    // group.bench_function("For", |b| b.iter(|| runtime(black_box("benches/scripts/while.ly"))));
    group.bench_function("Fibonacci 15", |b| {
        b.iter(|| runtime(black_box("benches/scripts/fib.ly")))
    });
    // group.bench_function("While", |b| b.iter(|| runtime(black_box("benches/scripts/while.ly"))));
    // group.bench_function("While (Short)", |b| b.iter(|| runtime(black_box("benches/scripts/while.short.ly"))));
    group.finish();
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default();
    targets = bench
}

criterion_main!(benches);
