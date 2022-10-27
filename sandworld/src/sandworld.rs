use gridmath::*;
use rand::{RngCore};
use rayon::prelude::*;
use std::{sync::atomic::AtomicU64};

use crate::chunk::*;
use crate::particle::*;
use crate::region;
use crate::region::*;

pub const WORLD_WIDTH: i32 = 1440;
pub const WORLD_HEIGHT: i32 = 960;

pub const TRUE_REGION_SIZE: usize = REGION_SIZE as usize * CHUNK_SIZE as usize;

pub struct World {
    regions: Vec<Region>
}

pub struct WorldUpdateStats {
    pub chunk_updates: u64,
    pub loaded_regions: usize,
    pub region_updates: u64,
}

impl World {
    pub fn new() -> Self {
        let created: World = World {
            regions: Vec::new()
        };

        // created.add_region(GridVec::new(0, 0));
        // created.add_region(GridVec::new(1, 0));
        // created.add_region(GridVec::new(-1, 0));

        // created.add_region(GridVec::new(0, -1));
        // created.add_region(GridVec::new(1, -1));
        // created.add_region(GridVec::new(-1, -1));

        // created.add_region(GridVec::new(0, 1));
        // created.add_region(GridVec::new(1, 1));
        // created.add_region(GridVec::new(-1, 1));

        // created.remove_region(GridVec::new(0, 0));
        
        return created;
    }

    fn add_region(&mut self, regpos: GridVec) {
        let mut added = Region::new(regpos);

        for region in self.regions.iter_mut() {
            region.check_add_neighbor(&mut added);
        }

        self.regions.push(added);

        println!("Added region {}", regpos);
    }

    fn add_region_if_needed(&mut self, regpos: GridVec) {
        if !self.has_region(regpos) {
            self.add_region(regpos);
        }
    }

    fn remove_region(&mut self, regpos: GridVec) {
        if let Some(index) = self.get_region_index(regpos) {
            self.regions.remove(index);

            for region in self.regions.iter_mut() {
                region.check_remove_neighbor(&regpos);
            }
        }
    }

    fn get_region_index(&self, regpos: GridVec) -> Option<usize> {
        for i in 0..self.regions.len() {
            if self.regions[i].position == regpos {
                return Some(i);
            }
        }
        return None;
    }

    fn get_regionpos_for_chunkpos(chunkpos: &GridVec) -> GridVec {
        let mut modpos = chunkpos.clone();
        if modpos.x < 0 {
            modpos.x -= REGION_SIZE as i32 - 1;
        }
        if modpos.y < 0 {
            modpos.y -= REGION_SIZE as i32 - 1;
        }
        GridVec::new(modpos.x / REGION_SIZE as i32, modpos.y / REGION_SIZE as i32)
    }

    fn get_regionpos_for_pos(pos: &GridVec) -> GridVec {
        Self::get_regionpos_for_chunkpos(&Self::get_chunkpos(pos))
    }

    fn has_region(&self, regpos: GridVec) -> bool {
        self.get_region_index(regpos).is_some()
    }

    pub fn contains(&self, pos: GridVec) -> bool {
        for reg in self.regions.iter() {
            if reg.contains_point(&pos) {
                return true;
            }
        }
        return false;
    }

    pub(crate) fn get_chunk_mut(&mut self, chunkpos: &GridVec) -> Option<&mut Box<Chunk>> {
        for reg in self.regions.iter_mut() {
            if reg.contains_chunk(chunkpos) {
                return reg.get_chunk_mut(chunkpos);
            }
        }
        return None;
    }

    pub fn get_chunk(&self, chunkpos: &GridVec) -> Option<&Box<Chunk>> {
        for reg in self.regions.iter() {
            if reg.contains_chunk(chunkpos) {
                return reg.get_chunk(chunkpos);
            }
        }
        return None;
    }

    pub fn get_added_chunks(&mut self) -> Vec<GridVec> {
        let mut set = Vec::new();
        for reg in self.regions.iter_mut() {
            set.append(&mut reg.get_added_chunks());
        }
        return set;
    }

    pub fn get_updated_chunks(&mut self) -> Vec<GridVec> {
        let mut set = Vec::new();
        for reg in self.regions.iter_mut() {
            set.append(&mut &mut reg.get_updated_chunks());
        }
        return set;
    }

    pub(crate) fn get_chunkpos(pos: &GridVec) -> GridVec {
        let mut modpos = pos.clone();
        if modpos.x < 0 {
            modpos.x -= CHUNK_SIZE as i32 - 1;
        }
        if modpos.y < 0 {
            modpos.y -= CHUNK_SIZE as i32 - 1;
        }
        GridVec::new(modpos.x / CHUNK_SIZE as i32, modpos.y / CHUNK_SIZE as i32)
    }

    pub(crate) fn get_chunklocal(pos: GridVec) -> GridVec {
        let mut modded = GridVec::new(pos.x % CHUNK_SIZE as i32, pos.y % CHUNK_SIZE as i32);
        if modded.x < 0 { 
            modded.x += CHUNK_SIZE as i32; 
        }
        if modded.y < 0 { 
            modded.y += CHUNK_SIZE as i32;
        }
        return modded;
    }

    pub fn get_particle(&self, pos: GridVec) -> Particle {
        for reg in self.regions.iter() {
            if reg.contains_point(&pos) {
                return reg.get_particle(pos);
            }
        }

        return Particle::new(ParticleType::Boundary);
    }

    pub fn replace_particle(&mut self, pos: GridVec, new_val: Particle) {
        if !self.contains(pos) {
            let chunkpos = World::get_chunkpos(&pos);
            let regpos = World::get_regionpos_for_chunkpos(&chunkpos);
            self.add_region(regpos);
        }

        let chunkpos = World::get_chunkpos(&pos);
        let chunklocal = World::get_chunklocal(pos);
        self.get_chunk_mut(&chunkpos).unwrap().set_particle(chunklocal.x as u8, chunklocal.y as u8, new_val);
    }

    pub fn add_particle(&mut self, pos: GridVec, new_val: Particle) {
        if !self.contains(pos) {
            let chunkpos = World::get_chunkpos(&pos);
            let regpos = World::get_regionpos_for_chunkpos(&chunkpos);
            self.add_region(regpos);
        }

        let chunkpos = World::get_chunkpos(&pos);
        let chunklocal = World::get_chunklocal(pos);

        self.get_chunk_mut(&chunkpos).unwrap().add_particle(chunklocal.x as u8, chunklocal.y as u8, new_val);
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

    pub fn update(&mut self, visible: GridBounds) -> WorldUpdateStats {
        let visible_regions = GridBounds::new_from_extents(
            Self::get_regionpos_for_pos(&visible.bottom_left()),
            Self::get_regionpos_for_pos(&visible.top_right()) + GridVec::new(1, 1)
        );

        for regpos in visible_regions.iter() {
            self.add_region_if_needed(regpos);
        }

        let updated_chunk_count = AtomicU64::new(0);
        let updated_region_count = AtomicU64::new(0);

        let target_chunk_updates = 128;

        self.regions.sort_unstable_by(|a, b| {
            let mut a_val = if a.get_bounds().overlaps(visible) { 1024 } else { 0 };
            let mut b_val = if b.get_bounds().overlaps(visible) { 1024 } else { 0 };

            a_val += (a.staleness as u64 + 1) * (a.staleness as u64 + 1) * (a.last_chunk_updates + 1);
            b_val += (b.staleness as u64 + 1) * (a.staleness as u64 + 1) * (b.last_chunk_updates + 1);
            
            b_val.cmp(&a_val)
        });

        let mut to_update = Vec::new();
        let mut to_skip = Vec::new();

        let mut estimated_chunk_updates = 0;

        for region in self.regions.iter_mut() {
            // Check level of commitment for this update
            if estimated_chunk_updates < target_chunk_updates {
                estimated_chunk_updates += region.last_chunk_updates;
                &mut to_update
            } 
            else {
                &mut to_skip
            }.push(region);
            
        }

        to_update.par_iter_mut().for_each(|region| {
            region.commit_updates();
            updated_region_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        });

        to_skip.par_iter_mut().for_each(|region| {
            region.skip_update();
        });

        let shift = (rand::thread_rng().next_u32() % 4) as i32;
        for i in 0..4 {
            let phase = i + shift;
            to_update.par_iter_mut().for_each(|region| {
                if region.staleness == 0 {
                    let region_chunk_updates = region.update(phase);
                    updated_chunk_count.fetch_add(region_chunk_updates, std::sync::atomic::Ordering::Relaxed); 
                }
            });
        }

        let chunk_updates = updated_chunk_count.load(std::sync::atomic::Ordering::Relaxed);

        // println!("Estimated {} chunk updates, actual was {} - a factor of {}", 
        //     estimated_chunk_updates, 
        //     chunk_updates,
        //     estimated_chunk_updates as f32 / chunk_updates as f32
        // );
        
        WorldUpdateStats {
            chunk_updates,
            loaded_regions: self.regions.len(),
            region_updates: updated_region_count.load(std::sync::atomic::Ordering::Relaxed),
        }
    }
}
