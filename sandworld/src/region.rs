pub const REGION_SIZE: usize = 2;

use std::sync::atomic::AtomicU64;

use gridmath::*;
use rayon::prelude::*;
use crate::{chunk::*, World, Particle, ParticleType};

pub struct Region {
    pub position: GridVec,
    chunks: Vec<Box<Chunk>>,
    // Chunks that have been added to the world since last polled
    added_chunks: Vec<GridVec>,
    // Chunks that have been updated since last polled
    updated_chunks: Vec<GridVec>,
}

impl Region {
    pub fn new(position: GridVec) -> Self {
        let mut reg = Region {
            position,
            chunks: vec![],
            added_chunks: vec![],
            updated_chunks: vec![],
        };
        println!("Created new region at {}, creating chunks", position);

        for y in 0..REGION_SIZE as i32 {
            for x in 0..REGION_SIZE as i32 {
                reg.add_chunk(GridVec::new(x, y) + (position * REGION_SIZE as i32));
            }
        }

        println!("Finished creating chunks for region {}", position);
        reg
    }

    fn add_chunk(&mut self, chunkpos: GridVec) {
        let mut added = Box::new(Chunk::new(chunkpos));

        for chunk in self.chunks.iter_mut() {
            chunk.check_add_neighbor(&mut added);
        }

        self.chunks.push(added);
        self.added_chunks.push(chunkpos);
    }

    fn _chunkpos_from_region_index(region_pos: GridVec, index: usize) -> GridVec {
        let x = (index % REGION_SIZE) as i32 + (region_pos.x * REGION_SIZE as i32);
        let y = (index / REGION_SIZE) as i32 + (region_pos.y * REGION_SIZE as i32);
        GridVec { x, y }
    }

    fn chunkpos_to_region_index(&self, chunkpos: &GridVec) -> usize {
        let x = chunkpos.x - (self.position.x * REGION_SIZE as i32);
        let y = chunkpos.y - (self.position.y * REGION_SIZE as i32);

        #[cfg(debug_assertions)] {
            if x < 0 || x >= REGION_SIZE as i32 || y < 0 || y >= REGION_SIZE as i32 {
                println!("Chunk position of {} is not within region at {}", chunkpos, self.position);
            }
        }

        x as usize + (y as usize * REGION_SIZE)
    }

    pub fn contains_chunk(&self, chunkpos: &GridVec) -> bool {
        let x = chunkpos.x - (self.position.x * REGION_SIZE as i32);
        let y = chunkpos.y - (self.position.y * REGION_SIZE as i32);

        x >= 0 && x < REGION_SIZE as i32 && y >= 0 && y < REGION_SIZE as i32
    }

    pub fn contains_point(&self, pos: &GridVec) -> bool {
        self.contains_chunk(&World::get_chunkpos(pos))
    }

    pub fn get_particle(&self, pos: GridVec) -> Particle {
        let chunk_opt = self.get_chunk(&World::get_chunkpos(&pos));
        if let Some(chunk) = chunk_opt {
            let chunklocal = World::get_chunklocal(pos);
            chunk.get_particle(chunklocal.x as u8, chunklocal.y as u8)
        }
        else {
            Particle::new(ParticleType::Boundary)
        }
    }

    pub fn get_added_chunks(&mut self) -> Vec<GridVec> {
        let set = self.added_chunks.clone();
        self.added_chunks.clear();
        return set;
    }

    pub fn get_updated_chunks(&mut self) -> Vec<GridVec> {
        let set = self.updated_chunks.clone();
        self.updated_chunks.clear();
        return set;
    }

    pub fn get_chunk(&self, chunkpos: &GridVec) -> Option<&Box<Chunk>> {
        if self.contains_chunk(chunkpos) {
            Some(&self.chunks[self.chunkpos_to_region_index(chunkpos)])
        }
        else {
            None
        }
    }

    pub fn get_chunk_mut(&mut self, chunkpos: &GridVec) -> Option<&mut Box<Chunk>> {
        if self.contains_chunk(chunkpos) {
            let index = self.chunkpos_to_region_index(chunkpos);
            Some(&mut self.chunks[index])
        }
        else {
            None
        }
    }

    pub fn commit_updates(&mut self) {
        self.chunks.iter().for_each(|chunk| {
            if chunk.dirty.is_some() || chunk.updated_last_frame.is_some()  {
                self.updated_chunks.push(chunk.position);
            }
        });

        self.chunks.par_iter_mut().for_each(|chunk| {
            chunk.commit_updates();
        });
    }

    pub fn update(&mut self, phase: i32) -> u64 {
        let updated_count = AtomicU64::new(0);

        let x_mod = (phase) % 2;
        let y_mod = ((phase) / 2) % 2; 

        self.chunks.par_iter_mut().for_each(|chunk| {
            let chunk_pos = chunk.position;

            if chunk_pos.x % 2 == x_mod && chunk_pos.y % 2 == y_mod {
                if chunk.update_this_frame.is_some() || chunk.updated_last_frame.is_some() { 
                    updated_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    chunk.update(); 
                }
            }
        });
        
        updated_count.load(std::sync::atomic::Ordering::Relaxed)
    }
}