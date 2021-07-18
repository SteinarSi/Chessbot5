use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chessbot5::backend::board::*;
use chessbot5::ai::{interface::AI, quiescence::Quiescence, pvs::PVS};

fn bench_ai(bot: &mut AI, depth: usize){
    bot.set_depth(depth);
    let m = bot.search(Board::new());
}

fn bench_ais(c: &mut Criterion) {
    //c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    let mut group = c.benchmark_group("AI benchmarks");
    group.sample_size(10);
    group.bench_function("Quiescence: d=9", |b| b.iter(|| bench_ai(&mut Quiescence::new(), black_box(9))));
    group.bench_function("PVS: d=9", |b| b.iter(|| bench_ai(&mut PVS::new(), black_box(9))));
    group.finish();
}

criterion_group!(benches, bench_ais);
criterion_main!(benches);