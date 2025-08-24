use criterion::{black_box, criterion_group, criterion_main, Criterion};
use heisenberg::core::config::Heisenberg;
use heisenberg::core::mode::Mode;
use heisenberg::core::router::Router;
use std::time::Duration;

fn benchmark_router_matching(c: &mut Criterion) {
    let config = Heisenberg::new()
        .spa("./dist")
        .pattern("/*")
        .dev_server("http://localhost:3000")
        .build();

    let mut router =
        Router::new(config.routes().to_vec(), Mode::Development).expect("Failed to create router");

    c.bench_function("router_match_cached", |b| {
        b.iter(|| {
            black_box(router.match_route("/app/dashboard"));
        })
    });

    c.bench_function("router_match_uncached", |b| {
        b.iter(|| {
            let mut fresh_router = Router::new(config.routes().to_vec(), Mode::Development)
                .expect("Failed to create router");
            black_box(fresh_router.match_route("/app/dashboard"));
        })
    });
}

fn benchmark_pattern_compilation(c: &mut Criterion) {
    c.bench_function("compile_patterns", |b| {
        b.iter(|| {
            let config = Heisenberg::new()
                .spa("./admin/dist")
                .pattern("/admin/*")
                .spa("./app/dist")
                .pattern("/app/*")
                .spa("./dist")
                .pattern("/*")
                .build();

            black_box(Router::new(config.routes().to_vec(), Mode::Development));
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = benchmark_router_matching, benchmark_pattern_compilation
);
criterion_main!(benches);
