use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chessbot5::backend::board::*;
use chessbot5::ai::{interface::AI, quiescence::Quiescence, minimax::MiniMax, 
            memomax::MemoMax, alphabeta::AlphaBeta, memoalpha::MemoAlpha, pvs::PVS,
            alphakiller::AlphaKiller, iddfs::IDDFS};

fn bench_ai(bot: &mut AI, depth: usize){
    let mut b = Board::new();
    bot.set_depth(depth);
    for _ in 0..4{
        let m = bot.search(b.clone());
        b.move_piece(&m);
    }

}

fn bench_ais(c: &mut Criterion) {
    let mut group = c.benchmark_group("AI benchmarks");
    group.sample_size(10);

    //group.bench_function("MiniMax: d=5", |b| b.iter(|| bench_ai(&mut MiniMax::new(), black_box(5))));
    //group.bench_function("MemoMax: d=5", |b| b.iter(|| bench_ai(&mut MemoMax::new(), black_box(5))));
    //group.bench_function("AlphaBeta: d=5", |b| b.iter(|| bench_ai(&mut AlphaBeta::new(), black_box(5))));
    //group.bench_function("AlphaBeta: d=7", |b| b.iter(|| bench_ai(&mut AlphaBeta::new(), black_box(7))));
    //group.bench_function("MemoAlpha: d=7", |b| b.iter(|| bench_ai(&mut MemoAlpha::new(), black_box(7))));
    //group.bench_function("AlphaKiller: d=7", |b| b.iter(|| bench_ai(&mut AlphaKiller::new(), black_box(7))));
    //group.bench_function("Quiescence: d=8", |b| b.iter(|| bench_ai(&mut Quiescence::new(), black_box(8))));
    group.bench_function("PVS: d=8", |b| b.iter(|| bench_ai(&mut PVS::new(), black_box(8))));
    group.bench_function("IDDFS: d=8", |b| b.iter(|| bench_ai(&mut PVS::new(), black_box(8))));

    group.finish();
}

criterion_group!(benches, bench_ais);
criterion_main!(benches);