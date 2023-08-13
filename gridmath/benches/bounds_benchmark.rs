use gridmath::{GridBounds, GridVec};


use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion
};

fn bounds_contain_point_positive_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1)));
    let i = black_box(GridVec::new(0, 0));

    c.bench_function(
        "bounds contains point (positive)",
        |bench| bench.iter(|| a.contains(i))
    );
}

fn bounds_contain_point_negative_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1)));
    let i = black_box(GridVec::new(10, 0));

    c.bench_function(
        "bounds contains point (negative)",
        |bench| bench.iter(|| a.contains(i))
    );
}

fn bounds_union_overlap_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(4, 4)));
    let b = black_box(GridBounds::new(GridVec::new(2, 2), GridVec::new(4, 4)));

    c.bench_function(
        "bounds union overlap",
        |bench| bench.iter(|| a.union(b))
    );
}

fn bounds_union_contained_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1)));
    let b = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(10, 10)));

    c.bench_function(
        "bounds union contained",
        |bench| bench.iter(|| a.union(b))
    );
}

fn bounds_union_no_overlap_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(4, 4)));
    let b = black_box(GridBounds::new(GridVec::new(10, 10), GridVec::new(4, 4)));

    c.bench_function(
        "bounds union no overlap",
        |bench| bench.iter(|| a.union(b))
    );
}

fn bounds_intersect_overlap_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(4, 4)));
    let b = black_box(GridBounds::new(GridVec::new(2, 2), GridVec::new(4, 4)));

    c.bench_function(
        "bounds intersect overlap",
        |bench| bench.iter(|| a.intersect(b))
    );
}

fn bounds_intersect_contained_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(1, 1)));
    let b = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(10, 10)));

    c.bench_function(
        "bounds intersect contained",
        |bench| bench.iter(|| a.intersect(b))
    );
}

fn bounds_intersect_no_overlap_benchmark(c: &mut Criterion) {
    let a = black_box(GridBounds::new(GridVec::new(0, 0), GridVec::new(4, 4)));
    let b = black_box(GridBounds::new(GridVec::new(10, 10), GridVec::new(4, 4)));

    c.bench_function(
        "bounds intersect no overlap",
        |bench| bench.iter(|| a.intersect(b))
    );
}

criterion_group!(benches, 
    bounds_contain_point_positive_benchmark, 
    bounds_contain_point_negative_benchmark,
    bounds_union_overlap_benchmark, 
    bounds_union_contained_benchmark, 
    bounds_union_no_overlap_benchmark,
    bounds_intersect_overlap_benchmark,
    bounds_intersect_contained_benchmark,
    bounds_intersect_no_overlap_benchmark
);
criterion_main!(benches);
