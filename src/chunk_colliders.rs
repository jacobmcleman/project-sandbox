use std::{hash::{DefaultHasher, Hash, Hasher}, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex}};

use bevy::prelude::*;
use sandworld::*;
use gridmath::{GridVec, GridBounds};
use bevy_xpbd_2d::{parry::shape::SharedShape, prelude::*};

use crate::sandsim::Sandworld;
use crate::chunk_display::ChunkDisplay;

const COLLIDES: ParticleSet = particle_set!(ParticleType::Stone, ParticleType::Sand, ParticleType::Gravel, ParticleType::Ice, ParticleType::Glass);

#[derive(Component, Default)]
pub struct ChunkColliderManager {
    last_data_source_time: f32,
    last_data_hash: Option<u64>,
}

#[derive(Resource, Default)]
struct AsyncColliderManager {
    in_progress: Vec<ChunkColliderGenerator>,
}

struct ChunkColliderGenerator {
    position: GridVec,
    request_time: f32,
    chunk_data: Vec<u8>,
    ready: Arc<AtomicBool>,
    result: Arc<Mutex<Option<(Vec<Vec2>, Vec<[u32; 2]>)>>>,
}

pub struct SandworldColliderPlugin;

impl Plugin for SandworldColliderPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(AsyncColliderManager::default())
        .add_systems(Update,(apply_generated_colliders, update_chunk_colliders))
        ;
    }
}

fn update_chunk_colliders(
    world: Res<Sandworld>,
    mut chunk_display_query: Query<(&ChunkDisplay, &mut ChunkColliderManager), With<Collider>>,
    mut collider_gen: ResMut<AsyncColliderManager>,
    time: Res<Time>,
) {
    let updated_chunks = world.world.get_updated_chunks();

    chunk_display_query.iter_mut().for_each(|(chunk_display, mut col_man)| {
        // Is this display entity currently representing a chunk
        if let Some(chunk_pos) = chunk_display.chunk_pos {
            // If the chunk this entity is representing needs to show an update
            if chunk_display.redraw || updated_chunks.contains(&chunk_pos) {
                // Get the chunk from the world, may fail if the world doesn't have the chunk yet
                if let Some(world_chunk) = world.world.get_chunk(&chunk_pos) {
                    let vals = world_chunk.get_marching_square_vals(COLLIDES);
                    let mut hasher = DefaultHasher::new();
                    vals.hash(&mut hasher);
                    let hash = hasher.finish();

                    if !(col_man.last_data_hash == Some(hash)) {
                        collider_gen.queue_collider_gen(chunk_pos, world_chunk, time.elapsed_seconds());
                        col_man.last_data_hash = Some(hash);
                    }
                }
            }
        }
    });
}

fn apply_generated_colliders(
    mut collider_gen: ResMut<AsyncColliderManager>,
    mut chunks_query: Query<(&ChunkDisplay, &mut ChunkColliderManager, &mut Collider)>,
) {
    let max_updates = 64;
    let mut ready_cols = Vec::with_capacity(max_updates);
    let mut ready_indices = Vec::with_capacity(max_updates);

    for i in 0..collider_gen.in_progress.len() {
        let building = &collider_gen.in_progress[i];
        if building.ready.load(Ordering::Relaxed) {
            ready_cols.push(building);
            ready_indices.push(i);

            if ready_indices.len() >= max_updates {
                break;
            }
        }
    }

    if ready_indices.len() > 0 {
        chunks_query.par_iter_mut().for_each(|(chunk_display, mut colman, mut collider)| {
            for ready_collider in ready_cols.iter() {
                if let Some(rep_pos) = chunk_display.chunk_pos {
                    // if this is the right position and this new data is actually newer than what we've got (wheeee async is fun)
                    if ready_collider.position == rep_pos && colman.last_data_source_time < ready_collider.request_time {
                        // println!("recieved new collider for {}", rep_pos);

                        let mut guard = ready_collider.result.as_ref().lock().unwrap();
                        if let Some((vertices, indices)) = guard.take() {
                            *collider = Collider::polyline(vertices, Some(indices));
                        }
                    }
                }
            }
        });

        // Remove all the jobs that were ready this update
        for remove_index in ready_indices.iter().rev() {
            collider_gen.in_progress.remove(*remove_index);
        }
    }
}

impl ChunkColliderGenerator {
    fn new(position: GridVec, request_time: f32, chunk_data: Vec<u8>) -> Self {
        ChunkColliderGenerator {
            position,
            request_time,
            chunk_data,
            ready: Arc::new(false.into()),
            result: Arc::new(Mutex::new(None)),
        }
    }

    fn start_build(&mut self) {
        let ready = self.ready.clone();
        let result = self.result.clone();
        let chunk_data = self.chunk_data.clone();
        
        rayon::spawn(move || {
            let mut polyline = marching_squares_polylines_from_chunkdata(&chunk_data);
            polyline.simplify(2.);
            let (vertices, indices) = polyline.to_verts_and_inds();

            result.lock().unwrap().replace((vertices, indices));
            ready.store(true, std::sync::atomic::Ordering::Relaxed);
        });
    }
}

impl AsyncColliderManager {
    fn queue_collider_gen(&mut self, chunk_position: GridVec, chunk: &sandworld::Chunk, time: f32) {
        self.queue_collider_gen_data(chunk_position, chunk.get_marching_square_vals(COLLIDES), time);
    }

    fn queue_collider_gen_data(&mut self, chunk_position: GridVec, chunk_data: Vec<u8>, time: f32) {
        self.in_progress.push(ChunkColliderGenerator::new(
            chunk_position,
            time,
            chunk_data,
        ));

        self.in_progress.last_mut().unwrap().start_build();
    }
}

fn marching_squares_polylines_from_chunkdata(chunk_data: &Vec<u8>) -> crate::polyline::PolylineSet {
    let mut polyline = crate::polyline::PolylineSet::new();

    let chunk_bounds = GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));

    for point in chunk_bounds.iter() {
        let x = point.x;
        let y = point.y;
        let index = y as usize * CHUNK_SIZE as usize + x as usize;

        let offset = Vec2::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32) * -0.5;

        let bottom = offset + Vec2::new(x as f32 + 0.5, y as f32);
        let top = offset + Vec2::new(x as f32 + 0.5, y as f32 + 1.0);
        let left = offset + Vec2::new(x as f32, y as f32 + 0.5);
        let right = offset + Vec2::new(x as f32 + 1.0, y as f32 + 0.5);

        match chunk_data[index] {
            1 => { polyline.add(bottom, left); },
            2 => { polyline.add(right, bottom); },
            3 => { polyline.add(right, left); },
            4 => { polyline.add(top, right); },
            5 => { polyline.add(top, left); polyline.add(bottom,right);},
            6 => { polyline.add(top, bottom) ;},
            7 => { polyline.add(top, left); },
            8 => { polyline.add(left, top) ;},
            9 => { polyline.add(bottom, top); },
            10 => { polyline.add(left, bottom); polyline.add(right, top);},
            11 => { polyline.add(right, top); },
            12 => { polyline.add(left, right); },
            13 => { polyline.add(bottom, right); },
            14 => { polyline.add(left, bottom); },
            _ => ()
        }
    }

    polyline
}