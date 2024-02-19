use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashSet;
use steam_tradeoffer_manager::types::TradeOfferId;

fn criterion_benchmark(c: &mut Criterion) {
    let ids_hashset: HashSet<TradeOfferId> = (0..1000).collect();
    let ids_vec: Vec<TradeOfferId> = (0..1000).collect();
    let needle_middle: TradeOfferId = 500;
    let needle_last: TradeOfferId = 999;
    
    c.bench_function("contains in HashSet", |b| b.iter(|| {
        ids_hashset.contains(&needle_middle)
    }));
    
    c.bench_function("contains in Vec", |b| b.iter(|| {
        ids_vec.contains(&needle_middle)
    }));
    
    c.bench_function("contains in HashSet (last number)", |b| b.iter(|| {
        ids_hashset.contains(&needle_last)
    }));
    
    c.bench_function("contains in Vec (last number)", |b| b.iter(|| {
        ids_vec.contains(&needle_last)
    }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}

criterion_main!(benches);