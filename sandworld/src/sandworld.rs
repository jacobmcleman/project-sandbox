use gridmath::*;
use rand::{RngCore};
use rayon::prelude::*;
use std::{collections::HashMap, sync::atomic::AtomicU64};

use crate::chunk::*;
use crate::particle::*;

pub const WORLD_WIDTH: i32 = 1440;
pub const WORLD_HEIGHT: i32 = 960;

pub struct World {
    // TODO keep chunks at consistent addresses once allocated
    // option - box all the chunks, store the boxes?
    // option - static self managed array with map of coord to index
    chunks: HashMap<u64, Box<Chunk>>,
}

impl World {
    pub fn new() -> Self {
        let mut created: World = World {
            chunks: HashMap::new(),
        };

        let world_width_chunks = (WORLD_WIDTH / CHUNK_SIZE as i32) + 1;
        let world_height_chunks = (WORLD_HEIGHT / CHUNK_SIZE as i32) + 1;

        for y in 0..world_height_chunks {
            for x in 0..world_width_chunks {
                let chunkpos = GridVec::new(x, y);
                created.add_chunk(chunkpos);
            }
        }

        return created;
    }

    pub fn contains(&self, pos: GridVec) -> bool {
        let chunk_pos = World::get_chunkpos(pos);
        return self.chunks.contains_key(&chunk_pos.combined());
    }

    fn add_chunk(&mut self, chunkpos: GridVec) {
        let mut added = Box::new(Chunk::new(chunkpos));

        for (_pos, chunk) in self.chunks.iter_mut() {
            chunk.check_add_neighbor(&mut added);
        }

        self.chunks.insert(chunkpos.combined(), added);
    }

    fn get_chunkpos(pos: GridVec) -> GridVec {
        GridVec::new(pos.x / CHUNK_SIZE as i32, pos.y / CHUNK_SIZE as i32)
    }

    fn get_chunklocal(pos: GridVec) -> GridVec {
        let mut modded = GridVec::new(pos.x % CHUNK_SIZE as i32, pos.y % CHUNK_SIZE as i32);
        if modded.x < 0 { 
            modded.x += CHUNK_SIZE as i32; 
        }
        if modded.y < 0 { 
            modded.y += CHUNK_SIZE as i32;
        }
        return modded;
    }

    pub fn _get_particle(&self, pos: GridVec) -> Particle {
        if !self.contains(pos) {
            return Particle::new(ParticleType::Boundary);
        }

        let chunk_pos = World::get_chunkpos(pos);
        let chunklocal = World::get_chunklocal(pos);
        return self.chunks.get(&chunk_pos.combined()).unwrap().get_particle(chunklocal.x as u8, chunklocal.y as u8);
    }

    pub fn replace_particle(&mut self, pos: GridVec, new_val: Particle) {
        if !self.contains(pos) {
            return;
        }

        let chunk_pos = World::get_chunkpos(pos);
        let chunklocal = World::get_chunklocal(pos);
        self.chunks.get_mut(&chunk_pos.combined()).unwrap().set_particle(chunklocal.x as u8, chunklocal.y as u8, new_val);
    }

    pub fn add_particle(&mut self, pos: GridVec, new_val: Particle) {
        if !self.contains(pos) {
            return;
        }

        let chunk_pos = World::get_chunkpos(pos);
        let chunklocal = World::get_chunklocal(pos);
        self.chunks.get_mut(&chunk_pos.combined()).unwrap().add_particle(chunklocal.x as u8, chunklocal.y as u8, new_val);
    }

    pub fn clear_circle(&mut self, pos: GridVec, radius: i32) {
        self.place_circle(pos, radius, Particle::new(ParticleType::Air), true);
    }

    pub fn place_circle(&mut self, pos: GridVec, radius: i32, new_val: Particle, replace: bool) {
        let left = pos.x - radius;
        let right = pos.x + radius;
        let bottom = pos.y - radius;
        let top = pos.y + radius;

        for y in bottom..top {
            for x in left..right {
                if replace { self.replace_particle(GridVec{x, y}, new_val.clone()); }
                else { self.add_particle(GridVec{x, y}, new_val.clone()); }
            }
        }
    }

    pub fn render(&self, screen_bounds: &GridBounds, draw_debug: bool) -> Vec<Particle> {
        let mut outbuffer = Vec::new();
        let top_right = screen_bounds.top_right();
        let bottom_left = screen_bounds.bottom_left();
        let buffer_height = top_right.y - bottom_left.y;
        let buffer_width = top_right.x - bottom_left.x;

        outbuffer.resize(buffer_height as usize * buffer_width as usize, Particle::new(ParticleType::Boundary));

        self.chunks.iter().for_each(|(chunk_pos_combined, chunk)| {
            let chunk_pos = GridVec::decombined(*chunk_pos_combined);
            let chunk_bottom_left = chunk_pos * CHUNK_SIZE as i32;
            let chunk_bounds = GridBounds::new_from_corner(chunk_bottom_left, GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));

            if let Some(overlap) = screen_bounds.intersect(chunk_bounds) {
                for overlap_pos in overlap.iter() {
                    let chunk_local = World::get_chunklocal(overlap_pos);
                    let buffer_local = overlap_pos - bottom_left;
                    let buffer_index = (buffer_local.x + (buffer_width * buffer_local.y)) as usize;

                    let mut write_val = chunk.get_particle(chunk_local.x as u8, chunk_local.y as u8);

                    if draw_debug {
                        if chunk_local.x == 0 || chunk_local.y == 0 || chunk_local.x as u8 == CHUNK_SIZE - 1 || chunk_local.y as u8 == CHUNK_SIZE - 1 {
                            write_val = Particle::new(ParticleType::Boundary);
                        }
                        if let Some(updated_bounds) = chunk.updated_last_frame {
                            if updated_bounds.is_boundary(chunk_local){
                                write_val = Particle::new(ParticleType::Dirty);
                            }
                        }
                    }

                    outbuffer[buffer_index] = write_val;
                }
            }
        });

        return outbuffer;
    }

    pub fn update(&mut self) -> u64 {
        let updated_count = AtomicU64::new(0);

        self.chunks.par_iter_mut().for_each(|(_pos, chunk)| {
            chunk.commit_updates();
        });

        let shift = (rand::thread_rng().next_u32() % 4) as i32;

        for i in 0..4{
            
            let x_mod = (i + shift) % 2;
            let y_mod = ((i + shift) / 2) % 2; 

            self.chunks.par_iter_mut().for_each(|(pos, chunk)| {
                let chunk_pos = GridVec::decombined(*pos);

                if chunk_pos.x % 2 == x_mod && chunk_pos.y % 2 == y_mod {
                    if chunk.update_this_frame.is_some() || chunk.updated_last_frame.is_some() { 
                        updated_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        chunk.update(); 
                    }
                }
            });
        }
        
        updated_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}
