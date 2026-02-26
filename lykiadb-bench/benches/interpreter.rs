use std::{
    fs::File,
    io::{BufReader, Read},
    time::Duration,
};

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lykiadb_server::{
    interpreter::Interpreter,
    session::{Session, SessionMode},
};

fn session(filename: &str) {
    let file = File::open(filename).unwrap();
    let mut content: String = String::new();
    BufReader::new(file)
        .read_to_string(&mut content)
        .expect("File couldn't be read.");
    let mut session = Session::new(SessionMode::File, Interpreter::new(None, true));
    session.interpret(&content).unwrap();
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

criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .warm_up_time(Duration::from_secs(5))
        .sample_size(200);
    targets = bench
}

criterion_main!(benches);
