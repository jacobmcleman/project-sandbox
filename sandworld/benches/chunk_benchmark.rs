use sandworld::{Chunk, Particle, ParticleType};
use gridmath::{GridBounds, GridVec};
use rand::Rng;
use rand::thread_rng;

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    Criterion
};

fn single_chunk_mark_dirty_once(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "single chunk mark dirty once",
        |bench| bench.iter(|| {
            let pos = chunk_bounds.get_random_within(&mut thread_rng());
            chunk.mark_dirty(pos.x, pos.y);
        })
    );
}

fn single_chunk_empty_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    
    c.bench_function(
        "single chunk minimal flush",
        |bench| bench.iter(|| {
            chunk.commit_updates();
        })
    );
}

fn single_chunk_mark_dirty_once_and_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "single chunk mark dirty once and flush",
        |bench| bench.iter(|| {
            let pos = chunk_bounds.get_random_within(&mut thread_rng());
            chunk.mark_dirty(pos.x, pos.y);
            chunk.commit_updates();
        })
    );
}

fn single_chunk_mark_dirty_10_and_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "single chunk mark dirty 10 times and flush",
        |bench| bench.iter(|| {
            for i in 0..10 {
                let pos = chunk_bounds.get_random_within(&mut thread_rng());
                chunk.mark_dirty(pos.x, pos.y);
            }
            chunk.commit_updates();
        })
    );
}

fn single_chunk_mark_dirty_100_and_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "single chunk mark dirty 100 times and flush",
        |bench| bench.iter(|| {
            for i in 0..100 {
                let pos = chunk_bounds.get_random_within(&mut thread_rng());
                chunk.mark_dirty(pos.x, pos.y);
            }
            chunk.commit_updates();
        })
    );
}

fn single_chunk_set_particle(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "single chunk set particle",
        |bench| bench.iter(|| {
            let pos = chunk_bounds.get_random_within(&mut thread_rng());
            chunk.set_particle(pos.x as u8, pos.y as u8, Particle::new(ParticleType::Sand));
        })
    );
}

fn middle_chunk_mark_dirty_once_and_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let mut neighbors_storage: [Chunk; 8] = [
        Chunk::new(GridVec{x:-1, y: 0}),
        Chunk::new(GridVec{x:-1, y: 1}),
        Chunk::new(GridVec{x:0, y: 1}),
        Chunk::new(GridVec{x:1, y: 1}),
        Chunk::new(GridVec{x:1, y: 0}),
        Chunk::new(GridVec{x:1, y: -1}),
        Chunk::new(GridVec{x:0, y: -1}),
        Chunk::new(GridVec{x:-1, y: -1}),
    ];
    for neighbor in neighbors_storage.iter_mut() {
        chunk.check_add_neighbor(neighbor);
    }
    
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "middle chunk mark dirty once and flush",
        |bench| bench.iter(|| {
            let pos = chunk_bounds.get_random_within(&mut thread_rng());
            chunk.mark_dirty(pos.x, pos.y);
            chunk.commit_updates();
        })
    );
}

fn middle_chunk_mark_dirty_10_and_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let mut neighbors_storage: [Chunk; 8] = [
        Chunk::new(GridVec{x:-1, y: 0}),
        Chunk::new(GridVec{x:-1, y: 1}),
        Chunk::new(GridVec{x:0, y: 1}),
        Chunk::new(GridVec{x:1, y: 1}),
        Chunk::new(GridVec{x:1, y: 0}),
        Chunk::new(GridVec{x:1, y: -1}),
        Chunk::new(GridVec{x:0, y: -1}),
        Chunk::new(GridVec{x:-1, y: -1}),
    ];
    for neighbor in neighbors_storage.iter_mut() {
        chunk.check_add_neighbor(neighbor);
    }
    
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "middle chunk mark dirty 10 times and flush",
        |bench| bench.iter(|| {
            for i in 0..10 {
                let pos = chunk_bounds.get_random_within(&mut thread_rng());
                chunk.mark_dirty(pos.x, pos.y);
            }
            chunk.commit_updates();
        })
    );
}

fn middle_chunk_mark_dirty_100_and_flush(c: &mut Criterion) {
    let mut chunk = Chunk::new(GridVec{x:0, y: 0});
    let mut neighbors_storage: [Chunk; 8] = [
        Chunk::new(GridVec{x:-1, y: 0}),
        Chunk::new(GridVec{x:-1, y: 1}),
        Chunk::new(GridVec{x:0, y: 1}),
        Chunk::new(GridVec{x:1, y: 1}),
        Chunk::new(GridVec{x:1, y: 0}),
        Chunk::new(GridVec{x:1, y: -1}),
        Chunk::new(GridVec{x:0, y: -1}),
        Chunk::new(GridVec{x:-1, y: -1}),
    ];
    for neighbor in neighbors_storage.iter_mut() {
        chunk.check_add_neighbor(neighbor);
    }
    
    let chunk_bounds = GridBounds::new_from_corner(GridVec{x:0, y: 0}, GridVec{x:16, y: 16});
    
    c.bench_function(
        "middle chunk mark dirty 100 times and flush",
        |bench| bench.iter(|| {
            for i in 0..100 {
                let pos = chunk_bounds.get_random_within(&mut thread_rng());
                chunk.mark_dirty(pos.x, pos.y);
            }
            chunk.commit_updates();
        })
    );
}


criterion_group!(benches, 
    single_chunk_mark_dirty_once,
    single_chunk_empty_flush,
    single_chunk_mark_dirty_once_and_flush,
    single_chunk_mark_dirty_10_and_flush,
    single_chunk_mark_dirty_100_and_flush,
    single_chunk_set_particle,
    middle_chunk_mark_dirty_once_and_flush,
    middle_chunk_mark_dirty_10_and_flush,
    middle_chunk_mark_dirty_100_and_flush
);
criterion_main!(benches);