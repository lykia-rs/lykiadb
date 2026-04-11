use std::{
    fs::File,
    io::{BufReader, Read},
    time::Duration,
};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lykiadb_common::memory::alloc_shared;
use lykiadb_server::{execution::session::Session, interpreter::output::Output};

fn session(filename: &str) {
    let file = File::open(filename).unwrap();
    let mut content: String = String::new();
    BufReader::new(file)
        .read_to_string(&mut content)
        .expect("File couldn't be read.");
    let mut session = Session::new(false);
    session.interpret(&content, alloc_shared(Output::new())).unwrap();
}

fn bench_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("join");

    // Standard configuration for more stable results
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    // Benchmark scripts are in lykiadb-bench/benches/scripts/
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/scripts/join/");

    group.bench_function("join_10k", |b| {
        b.iter(|| session(black_box(&format!("{base}join_10k.ly"))));
    });

    group.bench_function("scan_flat_10k", |b| {
        b.iter(|| session(black_box(&format!("{base}scan_flat_10k.ly"))));
    });
    group.bench_function("loop_nested_10k", |b| {
        b.iter(|| session(black_box(&format!("{base}loop_nested_10k.ly"))));
    });

    group.bench_function("loop_flat_10k", |b| {
        b.iter(|| session(black_box(&format!("{base}loop_flat_10k.ly"))));
    });

    group.finish();
}

fn bench_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");

    // Standard configuration for more stable results
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    // Benchmark scripts are in lykiadb-bench/benches/scripts/
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/scripts/scan/");

    group.bench_function("scan_square", |b| {
        b.iter(|| session(black_box(&format!("{base}scan_square.ly"))));
    });
    group.bench_function("loop_square", |b| {
        b.iter(|| session(black_box(&format!("{base}loop_square.ly"))));
    });

    group.bench_function("filter_square", |b| {
        b.iter(|| session(black_box(&format!("{base}filter_square.ly"))));
    });

    group.bench_function("loop_if_square", |b| {
        b.iter(|| session(black_box(&format!("{base}loop_if_square.ly"))));
    });

    group.finish();
}

fn bench_agg(c: &mut Criterion) {
    let mut group = c.benchmark_group("agg");

    // Standard configuration for more stable results
    group.warm_up_time(Duration::from_secs(5));
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(100);

    // Benchmark scripts are in lykiadb-bench/benches/scripts/
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/benches/scripts/agg/");

    group.bench_function("group_by_mod", |b| {
        b.iter(|| session(black_box(&format!("{base}group_by_mod.ly"))));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(5))
        .sample_size(200);
    targets = bench_scan, bench_join, bench_agg
}

criterion_main!(benches);
