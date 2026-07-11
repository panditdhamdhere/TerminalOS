use criterion::{Criterion, black_box, criterion_group, criterion_main};
use terminalos_search::{SearchEngine, SearchQuery};

fn search_benchmark(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("tempdir");
    let index_path = dir.path().join("index");
    let mut engine = SearchEngine::open(&index_path).expect("open index");

    for i in 0..100 {
        engine
            .index_document(
                &format!("src/file_{i}.rs"),
                &format!("fn handler_{i}() {{ terminalos search benchmark }}"),
            )
            .expect("index");
    }

    c.bench_function("search_keyword_hit", |b| {
        b.iter(|| {
            engine
                .search(black_box(&SearchQuery {
                    text: "terminalos".to_string(),
                    limit: 10,
                }))
                .expect("search")
        })
    });
}

criterion_group!(benches, search_benchmark);
criterion_main!(benches);
