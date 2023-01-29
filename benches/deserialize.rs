use criterion::{criterion_group, criterion_main, Criterion};
use steam_tradeoffer_manager::{
    response::ClassInfo,
};

fn criterion_benchmark(c: &mut Criterion) {
    let classinfo_bytes = include_bytes!("fixtures/classinfos/440_101785959_11040578.json");
    
    c.bench_function("deserializes classinfo", |b| b.iter(|| {
        serde_json::from_slice::<ClassInfo>(classinfo_bytes).ok();
    }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}

criterion_main!(benches);