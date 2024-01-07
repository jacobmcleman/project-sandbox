pub const REGION_SIZE: usize = 16;

use std::sync::{atomic::AtomicU64, Arc};

use gridmath::*;
use rayon::prelude::*;
use crate::{chunk::*, World, Particle, ParticleType, WorldGenerator};

pub struct Region {
    pub position: GridVec,
    pub staleness: u64, // Number of updates this region has been skipped
    pub last_chunk_updates: u64, // Number of chunks updated last time this region updated
    chunks: Vec<Box<Chunk>>,
    // Chunks that have been added to the world since last polled
    added_chunks: Vec<GridVec>,
    // Chunks that have been updated since last polled
    updated_chunks: Vec<GridVec>,
    pub update_priority: u64,
    generator: Arc<dyn WorldGenerator + Send + Sync>,
}

impl Region {
    pub fn new(position: GridVec, generator: Arc<dyn WorldGenerator + Send + Sync>) -> Self {
        let mut reg = Region {
            position,
            staleness: 0,
            last_chunk_updates: 0,
            chunks: vec![],
            added_chunks: vec![],
            updated_chunks: vec![],
            update_priority: 0,
            generator
        };
        
        for y in 0..REGION_SIZE as i32 {
            for x in 0..REGION_SIZE as i32 {
                reg.add_chunk(GridVec::new(x, y) + (position * REGION_SIZE as i32));
            }
        }

        reg.chunks.par_iter_mut().for_each(|chunk| {
            chunk.regenerate(&reg.generator);
        });

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

    pub(crate) fn check_add_neighbor(&mut self, other_reg: &mut Region) {
        if !self.position.is_adjacent(other_reg.position) {
            return;
        }

        let delta = other_reg.position - self.position;

        let mut self_chunks = Vec::new();
        let mut other_chunks = Vec::new();

        if delta.y == -1 {
            if delta.x == -1 { 
                self_chunks.push(GridVec::new(0, 0));
                other_chunks.push(GridVec::new(REGION_SIZE as i32 - 1 , REGION_SIZE as i32 - 1));
            }
            else if delta.x == 0 {
                for x in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(x, 0));
                    other_chunks.push(GridVec::new(x , REGION_SIZE as i32 - 1));
                }
            }
            else if delta.x == 1 {
                self_chunks.push(GridVec::new(REGION_SIZE as i32 - 1, 0));
                other_chunks.push(GridVec::new(0 , REGION_SIZE as i32 - 1));
            }
        }
        else if delta.y == 0 {
            if delta.x == -1 { 
                for y in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(0, y));
                    other_chunks.push(GridVec::new(REGION_SIZE as i32 - 1 , y));
                }
            }
            else if delta.x == 1 {
                for y in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(REGION_SIZE as i32 - 1, y));
                    other_chunks.push(GridVec::new(0 , y));
                }
            }
        }
        else if delta.y == 1 {
            if delta.x == -1 { 
                self_chunks.push(GridVec::new(0 , REGION_SIZE as i32 - 1));
                other_chunks.push(GridVec::new(REGION_SIZE as i32 - 1, 0));
            }
            else if delta.x == 0 {
                for x in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(x, REGION_SIZE as i32 - 1));
                    other_chunks.push(GridVec::new(x , 0));
                }
            }
            else if delta.x == 1 {
                self_chunks.push(GridVec::new(REGION_SIZE as i32 - 1 , REGION_SIZE as i32 - 1));
                other_chunks.push(GridVec::new(0, 0));
            }
        }

        for self_chunk_pos in self_chunks.iter() {
            for other_chunk_pos in other_chunks.iter() {
                let self_chunk = &mut self.chunks[Region::local_chunkpos_to_region_index(self_chunk_pos)];
                let other_chunk = &mut other_reg.chunks[Region::local_chunkpos_to_region_index(other_chunk_pos)];
                
                self_chunk.check_add_neighbor(other_chunk);
            }
        }
    }

    pub(crate) fn check_remove_neighbor(&mut self, other_regpos: &GridVec) {
        if !self.position.is_adjacent(*other_regpos) {
            return;
        }

        let delta = *other_regpos - self.position;

        let mut self_chunks = Vec::new();
        let mut other_chunks = Vec::new();

        if delta.y == -1 {
            if delta.x == -1 { 
                self_chunks.push(GridVec::new(0, 0));
                other_chunks.push(GridVec::new(REGION_SIZE as i32 - 1 , REGION_SIZE as i32 - 1));
            }
            else if delta.x == 0 {
                for x in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(x, 0));
                    other_chunks.push(GridVec::new(x , REGION_SIZE as i32 - 1));
                }
            }
            else if delta.x == 1 {
                self_chunks.push(GridVec::new(REGION_SIZE as i32 - 1, 0));
                other_chunks.push(GridVec::new(0 , REGION_SIZE as i32 - 1));
            }
        }
        else if delta.y == 0 {
            if delta.x == -1 { 
                for y in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(0, y));
                    other_chunks.push(GridVec::new(REGION_SIZE as i32 - 1 , y));
                }
            }
            else if delta.x == 1 {
                for y in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(REGION_SIZE as i32 - 1, y));
                    other_chunks.push(GridVec::new(0 , y));
                }
            }
        }
        else if delta.y == 1 {
            if delta.x == -1 { 
                self_chunks.push(GridVec::new(0 , REGION_SIZE as i32 - 1));
                other_chunks.push(GridVec::new(REGION_SIZE as i32 - 1, 0));
            }
            else if delta.x == 0 {
                for x in 0..REGION_SIZE as i32 {
                    self_chunks.push(GridVec::new(x, REGION_SIZE as i32 - 1));
                    other_chunks.push(GridVec::new(x , 0));
                }
            }
            else if delta.x == 1 {
                self_chunks.push(GridVec::new(REGION_SIZE as i32 - 1 , REGION_SIZE as i32 - 1));
                other_chunks.push(GridVec::new(0, 0));
            }
        }

        for self_chunk_pos in self_chunks.iter() {
            for other_chunk_pos in other_chunks.iter() {
                let self_chunk = &mut self.chunks[Region::local_chunkpos_to_region_index(self_chunk_pos)];
                
                self_chunk.check_remove_neighbor(*other_chunk_pos);
            }
        }
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
                return 0;
            }
        }

        Self::local_chunkpos_to_region_index(&GridVec{x, y})
    }

    fn local_chunkpos_to_region_index(local_chunkpos: &GridVec) -> usize {
        #[cfg(debug_assertions)] {
            if local_chunkpos.x < 0 || local_chunkpos.x >= REGION_SIZE as i32 || local_chunkpos.y < 0 || local_chunkpos.y >= REGION_SIZE as i32 {
                println!("Chunk position of {} is not within region", local_chunkpos);
            }
        }

        local_chunkpos.x as usize + (local_chunkpos.y as usize * REGION_SIZE)
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

    pub fn get_bounds(&self) -> GridBounds {
        GridBounds::new_from_corner(
            self.position * CHUNK_SIZE as i32 * REGION_SIZE as i32, 
            GridVec { x: CHUNK_SIZE as i32 * REGION_SIZE as i32, y: CHUNK_SIZE as i32 * REGION_SIZE as i32 }
        )
    }

    fn calc_update_priority(&mut self) {
        self.update_priority = (self.staleness + 1) * (self.staleness + 1) * (self.last_chunk_updates + 1);
    }

    pub fn skip_update(&mut self) {
        self.staleness += 1;

        self.calc_update_priority();
    }



    pub fn commit_updates(&mut self) {
        self.staleness = 0;
        self.last_chunk_updates = 0;
        self.calc_update_priority();

        self.chunks.iter().for_each(|chunk| {
            if chunk.dirty.read().unwrap().is_some() || chunk.updated_last_frame.is_some()  {
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

            if (chunk_pos.x % 2).abs() == x_mod && (chunk_pos.y % 2).abs() == y_mod {
                if chunk.update_this_frame.is_some() || chunk.updated_last_frame.is_some() { 
                    updated_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    chunk.update(); 
                }
            }
        });
        
        let updated = updated_count.load(std::sync::atomic::Ordering::Relaxed);
        self.last_chunk_updates += updated;
        updated
    }
}
