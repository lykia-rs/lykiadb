use criterion::{Criterion, black_box, criterion_group, criterion_main};
use lykiadb_lang::{
    Locals, Scopes,
    parser::{Parser, program::Program, resolver::Resolver},
    tokenizer::scanner::Scanner,
};
use rustc_hash::FxHashMap;

pub struct ParserBenchmark {
    scopes: Scopes,
    locals: Locals,
}

impl Default for ParserBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserBenchmark {
    pub fn new() -> ParserBenchmark {
        ParserBenchmark {
            scopes: vec![],
            locals: FxHashMap::default(),
        }
    }

    pub fn process(&mut self, source: &str) -> Program {
        let tokens = Scanner::scan(source).unwrap();
        let mut program = Parser::parse(&tokens).unwrap();
        let mut resolver = Resolver::new(self.scopes.clone(), &program, Some(self.locals.clone()));
        let (scopes, locals) = resolver.resolve().unwrap();

        self.scopes = scopes;
        self.locals.clone_from(&locals);
        program.set_locals(self.locals.clone());

        program
    }
}

fn runtime() {
    let content: String = "
    SELECT * FROM books b
    INNER JOIN
    (
      categories c
      INNER JOIN
      publishers AS p
      ON b.category_id = c.id
    )
    ON b.publisher_id = p.id
    WHERE p.name = 'Springer'
    UNION
    SELECT * FROM books
    INTERSECT
    SELECT * FROM books
    EXCEPT
    SELECT * FROM books;
    
    "
    .to_string();
    let mut parser = black_box(ParserBenchmark::new());
    black_box(parser.process(black_box(&content)));
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("sample-size-example");
    group.bench_function("2-way join", |b| b.iter(runtime));
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench
}

criterion_main!(benches);
