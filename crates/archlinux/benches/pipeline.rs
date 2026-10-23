use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use mirrors_arch::{ArchLinux, parse_local, parse_raw};

const MIRRORLIST_JSON: &str = include_str!("../sample/archlinux.json");

fn parsing_benchmarks(criterion: &mut Criterion) {
    criterion.bench_function("parse/raw_json", |bencher| {
        bencher.iter(|| parse_raw(black_box(MIRRORLIST_JSON)).unwrap());
    });

    criterion.bench_function("parse/full_pipeline", |bencher| {
        bencher.iter(|| parse_local(black_box(MIRRORLIST_JSON)).unwrap());
    });
}

fn sorting_benchmark(criterion: &mut Criterion) {
    let raw = parse_raw(MIRRORLIST_JSON).unwrap();

    criterion.bench_function("pipeline/sort_by_country", |bencher| {
        bencher.iter_batched(
            || raw.clone(),
            |raw| black_box(raw.sort_by_country()),
            BatchSize::SmallInput,
        );
    });
}

fn conversion_benchmark(criterion: &mut Criterion) {
    let sorted = parse_raw(MIRRORLIST_JSON).unwrap().sort_by_country();

    criterion.bench_function("pipeline/group_sorted_mirrors", |bencher| {
        bencher.iter_batched(
            || sorted.clone(),
            |sorted| black_box(ArchLinux::from(sorted)),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    parsing_benchmarks,
    sorting_benchmark,
    conversion_benchmark
);
criterion_main!(benches);
