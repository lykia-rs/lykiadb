use std::{
    fs::File,
    io::{BufReader, Read},
    time::Duration,
};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lykiadb_server::engine::{Runtime, RuntimeMode, interpreter::Interpreter};

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
    let mut group = c.benchmark_group("engine");

    // Standard configuration for more stable results
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    // Benchmark scripts are in lykiadb-bench/benches/scripts/
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/scripts/");

    group.bench_function("scan_square", |b| {
        b.iter(|| runtime(black_box(&format!("{}scan_square.ly", base))));
    });

    group.bench_function("loop_square", |b| {
        b.iter(|| runtime(black_box(&format!("{}loop_square.ly", base))));
    });

    group.bench_function("filter_square", |b| {
        b.iter(|| runtime(black_box(&format!("{}filter_square.ly", base))));
    });

    group.bench_function("loop_if_square", |b| {
        b.iter(|| runtime(black_box(&format!("{}loop_if_square.ly", base))));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(5))
        .sample_size(200);
    targets = bench
}

criterion_main!(benches);
