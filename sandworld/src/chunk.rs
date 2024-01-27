pub const CHUNK_SIZE: u8 = 64;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use gridmath::*;
use gridmath::gridline::GridLine;
use rand::{Rng, rngs::ThreadRng};
use crate::collisions::HitInfo;
use crate::region::REGION_SIZE;
use crate::{collisions, particle::*, World, WorldGenerator};

#[derive(Debug)]
pub struct Chunk {
    pub position: GridVec,
    neighbors: Neighbors,
    particles: [Particle; CHUNK_SIZE as usize * CHUNK_SIZE as usize],
    pub(crate) dirty: RwLock<Option<GridBounds>>,
    pub(crate) update_this_frame: Option<GridBounds>,
    pub(crate) updated_last_frame: Option<GridBounds>,
}

#[derive(Clone)]
enum CompressedParticleData {
    Uncompressed(Vec<Particle>),
    Monotype(Particle),
    RunLength((HashMap<u8, Particle>, Vec<(u8, u8)>)),
}

#[derive(Clone)]
pub struct CompressedChunk {
    pub position: GridVec,
    particle_data: CompressedParticleData,
}  
#[derive(Debug)]
struct Neighbors {
    top_left: Option<*mut Chunk>,
    top_center: Option<*mut Chunk>,
    top_right: Option<*mut Chunk>,
    mid_left: Option<*mut Chunk>,
    mid_right: Option<*mut Chunk>,
    bottom_left: Option<*mut Chunk>,
    bottom_center: Option<*mut Chunk>,
    bottom_right: Option<*mut Chunk>,
}

unsafe impl Send for Neighbors {}
unsafe impl Sync for Neighbors {}

struct NeighborIterator<'a> {
    neighbor_array: [&'a Option<*mut Chunk>; 8],
    cur_index: usize,
}

impl Iterator for NeighborIterator<'_> {
    type Item = Option<*mut Chunk>;

    fn next(&mut self) -> Option<Option<*mut Chunk>> {
        let i = self.cur_index;
        self.cur_index += 1;

        if i >= 8 {
            return None;
        }
        else {
            return Some(self.neighbor_array[i].clone());
        }
    }
}

impl Neighbors {
    fn new() -> Self {
        Neighbors { 
            top_left: None, 
            top_center: None,
            top_right: None, 
            mid_left: None, 
            mid_right: None, 
            bottom_left: None, 
            bottom_center: None, 
            bottom_right: None 
        }
    }

    fn iter<'a>(&'a self) -> NeighborIterator<'a> {
        NeighborIterator { 
            neighbor_array: [
                &self.top_left, &self.top_center, &self.top_right,
                &self.mid_left, &self.mid_right,
                &self.bottom_left, &self.bottom_center, &self.bottom_right,
            ], 
            cur_index: 0, 
        }
    }
}

impl Drop for Neighbors {
    fn drop(&mut self) {
        self.top_left = None;
        self.top_center = None;
        self.top_right = None;
        self.mid_left = None;
        self.mid_right = None; 
        self.bottom_left = None; 
        self.bottom_center = None; 
        self.bottom_right = None;
    }
}

impl CompressedChunk {
    pub fn decompress(&self) -> Chunk {
        let mut created = Chunk {
            position: self.position,
            neighbors: Neighbors::new(),
            particles: [Particle::default(); CHUNK_SIZE as usize *  CHUNK_SIZE as usize],
            dirty: RwLock::new(None),
            update_this_frame: None,
            updated_last_frame: None,
        };

        match &self.particle_data {
            CompressedParticleData::Monotype(part) => {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        created.set_particle_sloppy(x, y, *part)
                    }
                }
            }
            CompressedParticleData::Uncompressed(data) => {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let index = y as usize * CHUNK_SIZE as usize + x as usize;
                        created.set_particle_sloppy(x, y, data[index])
                    }
                }
            }
            CompressedParticleData::RunLength((map, data)) => {
                let mut index = 0;
                for (id, length) in data {
                    let part = map[id];
                    for _ in 0..*length {
                        created.particles[index] = part;
                        index += 1;
                    }
                }
            }
        }

        // created.mark_self_dirty();

        created
    }
}

impl Chunk {
    pub fn new(position: GridVec) -> Self {
        let created = Chunk {
            position,
            neighbors: Neighbors::new(),
            particles: [Particle::default(); CHUNK_SIZE as usize *  CHUNK_SIZE as usize],
            dirty: RwLock::new(None),
            update_this_frame: None,
            updated_last_frame: None,
        };

        return created;
    }
    
    pub fn generate(position: GridVec, generator: &Arc<dyn WorldGenerator + Send + Sync>) -> Self{
        let mut chunk = Chunk::new(position);
        chunk.regenerate(generator);
        chunk
    }

    pub fn regenerate(&mut self, generator: &Arc<dyn WorldGenerator + Send + Sync>) {
        // println!("generating chunk {}", self.position);
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let worldpos = GridVec::new(
                    x as i32 + (CHUNK_SIZE as i32 * self.position.x),
                     y as i32 + (CHUNK_SIZE as i32 * self.position.y));
                
                self.set_particle_sloppy(x, y, generator.get_particle(worldpos));
                self.mark_self_dirty();
            }
        }
    }

    pub fn compress(&self) -> CompressedChunk {
        let mut different_types = 0;
        let mut seen_types = HashSet::<Particle>::new();

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let part = self.get_particle(x, y);
                if !seen_types.contains(&part) {
                    seen_types.insert(part);
                    different_types += 1;
                }
            }
        }

        CompressedChunk {
            position: self.position,
            particle_data: if different_types == 1 { 
                // All the same type, so just take the first particle as archetype
                CompressedParticleData::Monotype(self.particles[0])
            }
            else if different_types < 16 {
                let mut next_id = 1;
                let mut map: HashMap<Particle, u8> = HashMap::new();
                let mut data: Vec<(u8, u8)> = Vec::new();

                let mut run_part = Particle::new(ParticleType::Air);
                let mut cur_run_length = 0;

                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let cur_part = self.get_particle(x, y);
                        
                        if cur_part == run_part && cur_run_length < u8::MAX {
                            cur_run_length += 1;
                        }
                        else {
                            if cur_run_length > 0 {
                                if !map.contains_key(&run_part) {
                                    map.insert(run_part, next_id);
                                    next_id += 1;
                                }
                                data.push((map[&run_part], cur_run_length));
                            }

                            run_part = cur_part;
                            cur_run_length = 1;
                        }
                    }
                }

                // Save the final run
                if !map.contains_key(&run_part) {
                    map.insert(run_part, next_id);
                }
                data.push((map[&run_part], cur_run_length));

                let mut flipmap: HashMap<u8, Particle> = HashMap::new();

                for (part, id) in map {
                    flipmap.insert(id, part);
                }

                CompressedParticleData::RunLength((flipmap, data))
            }
            else {
                CompressedParticleData::Uncompressed(self.part_data_vec())
            }
        }
    }

    fn part_data_vec(&self) -> Vec<Particle> {
        let mut data = Vec::new();
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                data.push(self.get_particle(x, y))
            }
        }
        data
    }
    
    fn get_index_in_chunk(x: u8, y: u8) -> usize {
        return y as usize * CHUNK_SIZE as usize + x as usize;
    }

    pub fn get_particle(&self, x: u8, y: u8) -> Particle {
        #[cfg(debug_assertions)] {
            if x >= CHUNK_SIZE {
                println!("X VALUE OF {} IS TOO LARGE", x);
                return Particle::default();
            }
            if y >= CHUNK_SIZE {
                println!("Y VALUE OF {} IS TOO LARGE", y);
                return Particle::default();
            }
        }

        return self.particles[Chunk::get_index_in_chunk(x, y)];
    }

    pub fn render_to_color_array(&self, draw_dirty: bool, draw_borders: bool) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(CHUNK_SIZE as usize * CHUNK_SIZE as usize * 4);

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let part = self.get_particle(x, CHUNK_SIZE - y - 1).particle_type;
                let mut color = get_color_for_type(part);

                if draw_borders {
                    if (x == 0 && self.position.x % REGION_SIZE as i32 == 0) 
                    || (x == CHUNK_SIZE - 1 && (self.position.x + 1) % REGION_SIZE as i32 == 0) {
                        color = get_color_for_type(ParticleType::RegionBoundary);
                    } 
                    else if x == 0 || x == CHUNK_SIZE - 1 {
                        color = get_color_for_type(ParticleType::Boundary);
                    }
                    else if (y == CHUNK_SIZE - 1 && self.position.y % REGION_SIZE as i32 == 0) 
                    || (y == 0 && (self.position.y + 1) % REGION_SIZE as i32 == 0) {
                        color = get_color_for_type(ParticleType::RegionBoundary);
                    } 
                    else if y == 0 || y == CHUNK_SIZE - 1 {
                        color = get_color_for_type(ParticleType::Boundary);
                    }
                }

                if draw_dirty {
                    if let Some(updated_bounds) = self.update_this_frame {
                        if updated_bounds.is_boundary(GridVec::new(x as i32, (CHUNK_SIZE - y - 1) as i32)) {
                            color = get_color_for_type(ParticleType::Dirty);
                        }
                    }
                }

                bytes.push(color[0]);
                bytes.push(color[1]);
                bytes.push(color[2]);
                bytes.push(color[3]);
            }
        }

        bytes
    }

    pub fn get_particle_mut(&mut self, x: u8, y: u8) -> &mut Particle {
        #[cfg(debug_assertions)] {
            if x >= CHUNK_SIZE {
                println!("X VALUE OF {} IS TOO LARGE", x);
            }
            if y >= CHUNK_SIZE {
                println!("Y VALUE OF {} IS TOO LARGE", y);
            }
        }

        return &mut self.particles[Chunk::get_index_in_chunk(x, y)];
    }

    pub fn contains(&self, x: i16, y: i16) -> bool {
        x >= 0 && y >= 0 && x < CHUNK_SIZE as i16 && y < CHUNK_SIZE as i16
    }

    pub fn get_neighbor(&self, dir: GridVec) -> Option<*mut Chunk> {
        if dir.y < 0 {
            if dir.x < 0 { 
                return self.neighbors.bottom_left;
            }
            else if dir.x == 0 {
                return self.neighbors.bottom_center;
            }
            else {
                return self.neighbors.bottom_right;
            }
        }
        else if dir.y == 0 {
            if dir.x < 0  { 
                return self.neighbors.mid_left;
            }
            else if dir.x > 0 {
                return self.neighbors.mid_right;
            }
            else {
                return None;
            }
        }
        else {
            if dir.x < 0 { 
                return self.neighbors.top_left;
            }
            else if dir.x == 0 {
                return self.neighbors.top_center;
            }
            else {
                return self.neighbors.top_right;
            }
        }
    }

    fn get_oob_direction(test_pos_x: i16, test_pos_y: i16) -> GridVec {
        GridVec { x: if test_pos_x < 0 { -1 } else if test_pos_x >= CHUNK_SIZE as i16 { 1 } else { 0 }, 
                y: if test_pos_y < 0 { -1 } else if test_pos_y >= CHUNK_SIZE as i16 { 1 } else { 0 } }
    }
    
    fn get_test_particle(&self, test_pos_x: i16, test_pos_y: i16) -> Option<Particle> {
        if self.contains(test_pos_x, test_pos_y) {
            Some(self.get_particle(test_pos_x as u8, test_pos_y as u8))
        }
        else {
            let neighbor_direction = Chunk::get_oob_direction(test_pos_x, test_pos_y);
            let neighbor = self.get_neighbor(neighbor_direction);
            if neighbor.is_none() {
                None
            }
            else {
                let chunk = neighbor.unwrap();
                let mut other_chunk_x = test_pos_x % (CHUNK_SIZE as i16);
                let mut other_chunk_y = test_pos_y % (CHUNK_SIZE as i16);
                if other_chunk_x < 0 { other_chunk_x += CHUNK_SIZE as i16; }
                if other_chunk_y < 0 { other_chunk_y += CHUNK_SIZE as i16; }
                unsafe {
                    Some((*chunk).get_particle(other_chunk_x as u8, other_chunk_y as u8))
                }
            }
        }
    }
    
    fn get_part_can_move(&self, test_pos_x: i16, test_pos_y: i16, priority_movement: bool, test_type: ParticleType) -> bool {
        if let Some(test_particle) = self.get_test_particle(test_pos_x, test_pos_y) {
            if test_particle.updated_this_frame() { 
                // Need to allow things to fall into spaces otherwise weird air bubbles are allowed to persist
                return priority_movement || Particle::get_can_replace(test_type, test_particle.particle_type); 
            }
            if test_particle.particle_type == ParticleType::Air { return true; }
            else if Particle::get_can_replace(test_type, test_particle.particle_type) { 
                return true; 
            }
        }
        return false;
    }

    pub fn cast_ray(&self, hitmask: &ParticleSet, line: GridLine) -> Option<HitInfo> {
        let chunk_world_root = self.position * CHUNK_SIZE as i32;
        // Verify line crosses this chunk
        // Pull out relevant section of the line
        let bounds = GridBounds::new_from_corner(
            chunk_world_root, 
            GridVec::new(CHUNK_SIZE as i32 - 1, CHUNK_SIZE as i32 - 1));

        if let Some(clipped_line) = bounds.clip_line(line) {
            // Convert intersection segment coords to local coords
            let local_line = GridLine::new(
                World::get_chunklocal(clipped_line.a),
                World::get_chunklocal(clipped_line.b)
            );

            // Run local version raycast
            if let Some((hit_x, hit_y)) = self.cast_ray_local(hitmask, local_line) {
                let world_hit_pos = chunk_world_root + GridVec::new(hit_x as i32, hit_y as i32);
                Some(HitInfo {
                    point: world_hit_pos,
                    part: self.get_particle(hit_x, hit_y),
                })
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    fn cast_ray_local(&self, hitmask: &ParticleSet, local_line: GridLine) -> Option<(u8, u8)> {
        #[cfg(debug_assertions)] {
            if !self.contains(local_line.a.x as i16, local_line.a.y as i16) {
                println!("Start of line {} is outside chunk!", local_line);
            }
            if !self.contains(local_line.b.x as i16, local_line.b.y as i16) {
                println!("End of line {} is outside chunk!", local_line);
            }
        }

        for point in local_line.along() {
            let test_part = self.get_particle(point.x as u8, point.y as u8);

            if hitmask.test(test_part.particle_type) {
                return Some((point.x as u8, point.y as u8));
            }
        }

        return None;
    }


    fn test_vec(&self, base_x: i16, base_y: i16, test_vec_x: i8, test_vec_y: i8, test_type: ParticleType) -> bool {
        if test_vec_x.abs() > 1 || test_vec_y.abs() > 1 {
            // need to step
            let test_pos_x = base_x + test_vec_x.signum() as i16;
            let test_pos_y = base_y + test_vec_y.signum() as i16;
            
            if self.get_part_can_move(test_pos_x, test_pos_y, test_vec_y < 0, test_type) {
                // Recurse to call next step if this step was clear
                self.test_vec(test_pos_x, test_pos_y, 
                    test_vec_x - test_vec_x.signum(), test_vec_y - test_vec_y.signum(), test_type)
            }
            else if test_vec_x != 0 && test_vec_y != 0 {
                if test_vec_x.abs() > test_vec_y.abs() && self.get_part_can_move(test_pos_x, base_y, test_vec_y < 0, test_type) {
                    self.test_vec(test_pos_x, base_y, 
                        test_vec_x - test_vec_x.signum(), test_vec_y, test_type)
                }
                else if  test_vec_x.abs() < test_vec_y.abs() && self.get_part_can_move(base_x, test_pos_y, test_vec_y < 0, test_type) {
                    self.test_vec(base_x, test_pos_y, 
                        test_vec_x, test_vec_y - test_vec_y.signum(), test_type)
                }
                else {
                    false
                }
            }
            else { 
                false
            }
        }
        else {
            let test_pos_x = base_x as i16 + test_vec_x as i16;
            let test_pos_y = base_y as i16 + test_vec_y as i16;
            
            self.get_part_can_move(test_pos_x, test_pos_y, test_vec_y < 0, test_type)
        }
    }

    pub(crate) fn check_add_neighbor(&mut self, new_chunk: &mut Chunk) {
        if !self.position.is_adjacent(new_chunk.position) {
            return;
        }

        let delta = new_chunk.position - self.position;

        if delta.y == -1 {
            if delta.x == -1 { 
                self.neighbors.bottom_left = Some(new_chunk);
                new_chunk.neighbors.top_right = Some(self);
            }
            else if delta.x == 0 {
                self.neighbors.bottom_center = Some(new_chunk);
                new_chunk.neighbors.top_center = Some(self);
            }
            else if delta.x == 1 {
                self.neighbors.bottom_right = Some(new_chunk);
                new_chunk.neighbors.top_left = Some(self);
            }
        }
        else if delta.y == 0 {
            if delta.x == -1 { 
                self.neighbors.mid_left = Some(new_chunk);
                new_chunk.neighbors.mid_right = Some(self);
            }
            else if delta.x == 1 {
                self.neighbors.mid_right = Some(new_chunk);
                new_chunk.neighbors.mid_left = Some(self);
            }
        }
        else if delta.y == 1 {
            if delta.x == -1 { 
                self.neighbors.top_left = Some(new_chunk);
                new_chunk.neighbors.bottom_right = Some(self);
            }
            else if delta.x == 0 {
                self.neighbors.top_center = Some(new_chunk);
                new_chunk.neighbors.bottom_center = Some(self);
            }
            else if delta.x == 1 {
                self.neighbors.top_right = Some(new_chunk);
                new_chunk.neighbors.bottom_left = Some(self);
            }
        }

        self.mark_self_dirty();
    }

    pub fn chunkpos_to_local_chunkpos(&self, from_chunk: &Chunk, from_x: u8, from_y: u8) -> GridVec {
        let chunk_diff = (from_chunk.position - self.position) * CHUNK_SIZE as i32;
        GridVec::new(from_x as i32, from_y as i32) + chunk_diff
    }
    
    pub fn set_particle(&mut self, x: u8, y: u8, val: Particle) {
        self.set_particle_sloppy(x, y, val);
        self.mark_dirty(x as i32, y as i32);
    }

    // Do a set operation without handling dirty markings
    // Only use for things like a batch set where the dirty bits will be handled later in one batch
    pub fn set_particle_sloppy(&mut self, x: u8, y: u8, val: Particle) {
        self.particles[Chunk::get_index_in_chunk(x, y)] = val;
        self.particles[Chunk::get_index_in_chunk(x, y)].set_updated_this_frame(true);
    }
    
    pub fn mark_region_dirty(&mut self, bounds: GridBounds) {
        let chunk_bounds = GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));
        let dirty_bounds = chunk_bounds.intersect(bounds);
        
        if dirty_bounds.is_some() {
            let new_bounds = GridBounds::option_union(*self.dirty.read().unwrap(), dirty_bounds);
            *self.dirty.write().unwrap() = new_bounds; 
        }
    }
    
    pub fn mark_self_dirty(&mut self) {
        self.mark_region_dirty(GridBounds::new_from_corner(GridVec::new(-2, -2), GridVec::new(CHUNK_SIZE as i32 + 4, CHUNK_SIZE as i32 + 4)));
    }

    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        let dirty_bounds = GridBounds::new(GridVec { x, y }, GridVec { x: 4, y: 4 });        
        self.mark_region_dirty(dirty_bounds);

        if self.contains(x as i16, y as i16){
            let local_x = x as u8;
            let local_y = y as u8;

            for neighbor_opt in self.neighbors.iter() {
                if let Some(neighbor) = neighbor_opt {
                    unsafe {
                        let local = (*neighbor).chunkpos_to_local_chunkpos(self, local_x, local_y);
                        (*neighbor).mark_dirty(local.x, local.y);
                    }
                }
            }
        }
    }

    pub fn add_particle(&mut self, x: i16, y: i16, val: Particle) {
        self.replace_particle_filtered(x, y, val, ParticleType::Air)
    }
    
    pub fn replace_particle_filtered(&mut self, x: i16, y: i16, val: Particle, replace_type: ParticleType) {
        if self.get_local_part(x, y) == replace_type {
            self.set_local_part(x, y, val);
        }
    }
    
        
    pub(crate) fn try_state_change(&mut self, x: u8, y: u8, temp: i32, rng: &mut ThreadRng) {
        let cur_part = self.get_particle(x, y);
        if let Some(new_state) = try_state_change(cur_part.particle_type, temp, rng) {
            self.set_particle(x, y, Particle::new(new_state));
        }
    }

    fn _is_border(x: u8, y: u8) -> bool {
        x == 0 || y == 0 || x == CHUNK_SIZE - 1  || y == CHUNK_SIZE - 1
    }

    pub(crate) fn commit_updates(&mut self) {
        self.update_this_frame = *self.dirty.read().unwrap();
        *self.dirty.write().unwrap() = None;

        if let Some(to_update) = GridBounds::option_union(self.update_this_frame, self.updated_last_frame) {
            for point in to_update.slide_iter() {
                let x = point.x as u8;
                let y = point.y as u8;
                
                self.get_particle_mut(x, y).set_updated_this_frame(false);
            }
        }
    }
    
    pub fn set_local_part(&mut self, x: i16, y: i16, val: Particle) {
        if self.contains(x, y) {
            self.set_particle(x as u8, y as u8, val);
        }
        else if let Some(neighbor) = self.get_neighbor( Chunk::get_oob_direction(x, y) ) {
            let dir = Chunk::get_oob_direction(x, y);
            let adjusted_x = x - (dir.x as i16 * CHUNK_SIZE as i16);
            let adjusted_y = y - (dir.y as i16 * CHUNK_SIZE as i16);
            
            unsafe {
                (*neighbor).set_particle(adjusted_x as u8, adjusted_y as u8, val)
            }
        }
    }
    
    pub fn get_local_part(&self, x: i16, y: i16) -> ParticleType {
        if self.contains(x, y) {
            self.get_particle(x as u8, y as u8).particle_type
        }
        else if let Some(neighbor) = self.get_neighbor( Chunk::get_oob_direction(x, y) ) {
            let dir = Chunk::get_oob_direction(x, y);
            let adjusted_x = x - (dir.x as i16 * CHUNK_SIZE as i16);
            let adjusted_y = y - (dir.y as i16 * CHUNK_SIZE as i16);
            
            unsafe {
                (*neighbor).get_particle(adjusted_x as u8, adjusted_y as u8).particle_type
            }
        }
        else {
            ParticleType::Air
        }
    }
    
    fn iterate_neighbor_parts(&self, x: i16, y: i16, call: &mut dyn FnMut(ParticleType) -> ()) {
        call(self.get_local_part(x, y + 1));
        call(self.get_local_part(x + 1, y + 1));
        call(self.get_local_part(x + 1, y) );
        call(self.get_local_part(x + 1, y - 1));
        call(self.get_local_part(x, y - 1));
        call(self.get_local_part(x - 1, y - 1));
        call(self.get_local_part(x - 1, y));
        call(self.get_local_part(x - 1, y + 1));
    }
    
    fn count_neighbors_of_type(&self, x: i16, y: i16, search: &ParticleSet) -> u8 {
        let mut count = 0;
        self.iterate_neighbor_parts(x, y, &mut |part_type: ParticleType| {
            if search.test(part_type){
                count += 1;
            }
        });
        return count;
    }
    
    fn get_neighbors(&self, x: i16, y: i16) -> [ParticleType; 8] {
        let mut neighbors: [ParticleType; 8] = [ParticleType::Air; 8];
        let mut index = 0;
        
        self.iterate_neighbor_parts(x, y, &mut |part_type: ParticleType| {
            neighbors[index] = part_type;
            index += 1;
        });
        return neighbors;
    }
    
    fn caclulate_local_temp(&self, x: i16, y: i16) -> i32 {
        let mut total = 0;
        let own_temp = get_heat_for_type(self.get_local_part(x, y));
        self.iterate_neighbor_parts(x, y, &mut |part_type: ParticleType| {
            total += if part_type == ParticleType::Air { own_temp / 2 } else { get_heat_for_type(part_type) };
        });
        return total;
    }

    fn try_erode(&mut self, rng: &mut ThreadRng, x: i16, y: i16, vel: &GridVec) {
        if self.contains(x, y) {
            let part = self.get_particle(x as u8, y as u8);
            if !part.updated_this_frame() {
                match part.particle_type {
                    ParticleType::Sand => {
                        let next_x = x as i16 + vel.x as i16;
                        let next_y = y as i16 + vel.y as i16;
                        if self.contains(next_x, next_y) && rng.gen_bool(0.1) {
                            self.set_particle(x as u8, y as u8, self.get_particle(next_x as u8, next_y as u8));
                            self.set_particle(next_x as u8, next_y as u8, part);
                        }
                    }
                    ParticleType::Gravel => {
                        if rng.gen_bool(0.001) {
                            self.set_particle(x as u8, y as u8, Particle::new_already_updated(ParticleType::Sand))
                        }
                        else {
                            let next_x = x as i16 + vel.x as i16;
                            let next_y = y as i16 + vel.y as i16;
                            if self.contains(next_x, next_y) && rng.gen_bool(0.001) {
                                self.set_particle(x as u8, y as u8, self.get_particle(next_x as u8, next_y as u8));
                                self.set_particle(next_x as u8, next_y as u8, part);
                            }
                        }
                    }
                    ParticleType::Stone => {
                        if rng.gen_bool(0.0005) {
                            self.set_particle(x as u8, y as u8, Particle::new_already_updated(ParticleType::Gravel));
                        }
                    }
                    _ => ()
                }
            }
        }
        else if let Some(neighbor) = self.get_neighbor( Chunk::get_oob_direction(x, y) ) {
            let dir = Chunk::get_oob_direction(x, y);
            let adjusted_x = x - (dir.x as i16 * CHUNK_SIZE as i16);
            let adjusted_y = y - (dir.y as i16 * CHUNK_SIZE as i16);
            
            unsafe {
                (*neighbor).try_erode(rng, adjusted_x, adjusted_y, vel);
            }
        }
    }

    fn neighbors_direction_map(index: usize)-> GridVec {
        match index {
            0 => GridVec::new(0, 1),
            1 => GridVec::new(1, 1),
            2 => GridVec::new(1, 0),
            3 => GridVec::new(1, -1),
            4 => GridVec::new(0, -1),
            5 => GridVec::new(-1, -1),
            6 => GridVec::new(-1, 0),
            7 => GridVec::new(-1, 1),
            _ => GridVec::new(0,0)
        }
    }
    
    fn particle_movement(&mut self, x: u8, y: u8, cur_part: &Particle, rng: &mut ThreadRng, move_override: Option<Vec<GridVec>>, neighbors: &[ParticleType; 8], local_temp: i32) -> GridVec {
        let available_moves = if let Some(movement) = move_override { 
            vec![movement]
        }
        else {
            Particle::get_possible_moves(cur_part.particle_type)
        };
        
        let viscosity_val = get_viscosity_for_type(cur_part.particle_type, local_temp);
        let mut viscosity_vec = GridVec::new(0, 0);
        if viscosity_val != 0 {
            let mut matching_neighbors = 0;
            for i in 0..8 {
                if neighbors[i] == cur_part.particle_type {
                    viscosity_vec = viscosity_vec + Self::neighbors_direction_map(i);
                    matching_neighbors += 1;
                }
            }
            if matching_neighbors > viscosity_val {
                viscosity_vec = GridVec::new(0, 0);
            }
        }
        let viscosity_vec = viscosity_vec * viscosity_val;

        if available_moves.len() > 0 {
            let mut possible_moves = Vec::<GridVec>::new();
            for move_set in available_moves {
                for mut vec in move_set {
                    vec = vec + viscosity_vec;
                    if self.test_vec(x as i16, y as i16, vec.x as i8, vec.y as i8, cur_part.particle_type) {
                        possible_moves.push(vec.clone());
                    }
                }
                
                if !possible_moves.is_empty() { break; }
            }
            
            if possible_moves.len() > 0 {
                let chosen_vec = possible_moves[rng.gen_range(0..possible_moves.len())];
                let chosen_x = x as i16 + chosen_vec.x as i16;
                let chosen_y = y as i16 + chosen_vec.y as i16;
                
                self.make_move(x, y, chosen_x, chosen_y, cur_part);
                
                chosen_vec
            }
            else {
                GridVec::new(0, 0)
            }
        }
        else {
            GridVec::new(0, 0)
        }
    }
    
    fn make_move(&mut self, x: u8, y: u8, chosen_x: i16, chosen_y: i16, cur_part: &Particle) {
        if self.contains(chosen_x, chosen_y) {
            self.set_particle(x, y, self.get_particle(chosen_x as u8, chosen_y as u8));
            self.set_particle(chosen_x as u8, chosen_y as u8, cur_part.clone());
        }
        else {
            let neighbor_direction = Chunk::get_oob_direction(chosen_x, chosen_y);
            let neighbor = self.get_neighbor(neighbor_direction);
            if let Some(chunk) = neighbor {
                let mut other_chunk_x = chosen_x % (CHUNK_SIZE as i16);
                let mut other_chunk_y = chosen_y % (CHUNK_SIZE as i16);
                if other_chunk_x < 0 { other_chunk_x += CHUNK_SIZE as i16; }
                if other_chunk_y < 0 { other_chunk_y += CHUNK_SIZE as i16; }
                unsafe {
                    self.set_particle(x, y, (*chunk).get_particle(other_chunk_x as u8, other_chunk_y as u8));
                    (*chunk).set_particle(other_chunk_x as u8, other_chunk_y as u8, cur_part.clone());
                }
            }
        }
    }

    pub(crate) fn update(&mut self) {      
        let mut rng = rand::thread_rng();

        if let Some(to_update) = GridBounds::option_union(self.update_this_frame, self.updated_last_frame) {
            for point in to_update.slide_iter() {
                let x = point.x as u8;
                let y = point.y as u8;
                
                let cur_part = self.get_particle(x, y);

                if !cur_part.updated_this_frame() {           
                    let neighbors = self.get_neighbors(x as i16, y as i16);
                    // Custom Logic
                    let mut move_override = None;
                    let mut destroy_if_not_moved = false;
                    if let Some(update_fn) = get_update_fn_for_type(cur_part.particle_type) {
                        let commands = update_fn(GridVec::new(x as i32, y as i32), cur_part, &neighbors);
                        for command in commands {
                            match command {
                                ChunkCommand::Add((position, particle_type, particle_data)) => self.add_particle(position.x as i16, position.y as i16, Particle::new_with_data(particle_type, particle_data)), 
                                ChunkCommand::Move(movement) => move_override = Some(movement),
                                ChunkCommand::MoveOrDestroy(movement) => {
                                    move_override = Some(movement);
                                    destroy_if_not_moved = true;
                                },
                                ChunkCommand::Remove => self.set_particle(x, y, Particle::new(ParticleType::Air)),
                                ChunkCommand::Mutate(particle_type, particle_data) => self.set_particle(x, y, Particle::new_with_data(particle_type, particle_data)),
                            }
                        }
                    }
                    
                    // Temperature
                    let local_temp = self.caclulate_local_temp(x as i16, y as i16);
                    if let Some(mut new_state) = try_state_change(cur_part.particle_type, local_temp, &mut rng) {
                        // Check lonely
                        if get_is_lonely_type(new_state) 
                            && self.count_neighbors_of_type(x as i16, y as i16, &SOLID_MATS) == 0 {
                            new_state = get_lonely_break_type(new_state);
                        }

                        self.set_particle(x, y, Particle::new(new_state));
                    }

                    // Movement
                    let move_amount = self.particle_movement(x, y, &cur_part, &mut rng, move_override, &neighbors, local_temp);
                    
                    // Erosion
                    if cur_part.particle_type == ParticleType::Water && move_amount.manhattan_length() > 1 {
                        self.try_erode(&mut rng, x as i16, y as i16 - 1, &move_amount);
                        self.try_erode(&mut rng, x as i16, y as i16 + 1, &move_amount);
                        self.try_erode(&mut rng, x as i16 - 1, y as i16, &move_amount);
                        self.try_erode(&mut rng, x as i16 + 1, y as i16, &move_amount);
                    }
                    
                    // Tail custom logic 
                    if destroy_if_not_moved && move_amount.manhattan_length() == 0 {
                        self.set_particle(x, y, Particle::new(ParticleType::Air))
                    }
                }
            }
        }

        self.updated_last_frame = self.update_this_frame;
    }

    pub(crate) fn check_remove_neighbor(&mut self, removed_position: GridVec) {
        if !self.position.is_adjacent(removed_position) {
            return;
        }

        let delta = removed_position - self.position;

        if delta.y == -1 {
            if delta.x == -1 { 
                self.neighbors.bottom_left = None;
            }
            else if delta.x == 0 {
                self.neighbors.bottom_center = None;
            }
            else if delta.x == 1 {
                self.neighbors.bottom_right = None;
            }
        }
        else if delta.y == 0 {
            if delta.x == -1 { 
                self.neighbors.mid_left = None;
            }
            else if delta.x == 1 {
                self.neighbors.mid_right = None;
            }
        }
        else if delta.y == 1 {
            if delta.x == -1 { 
                self.neighbors.top_left = None;
            }
            else if delta.x == 0 {
                self.neighbors.top_center = None;
            }
            else if delta.x == 1 {
                self.neighbors.top_right = None;
            }
        }
    }
}