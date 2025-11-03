use std::{
    fs::File,
    io::{BufReader, Read}, time::Duration,
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
    /*
    group.bench_function("fib25", |b| {
        b.iter(|| runtime(black_box("benches/scripts/fib.ly")));
    }); */

    group.bench_function("scan_square", |b| {
        b.iter(|| runtime(black_box("benches/scripts/scan_square.ly")));
    });

    group.bench_function("loop_square", |b| {
        b.iter(|| runtime(black_box("benches/scripts/loop_square.ly")));
    });

    group.finish();
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().measurement_time(Duration::from_secs(50));
    targets = bench
}

criterion_main!(benches);
