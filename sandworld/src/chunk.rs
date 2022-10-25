pub const CHUNK_SIZE: u8 = 64;
use gridmath::*;
use rand::{Rng, rngs::ThreadRng};
use crate::region::REGION_SIZE;
use crate::particle::*;

pub struct Chunk {
    pub position: GridVec,
    neighbors: Neighbors,
    particles: [Particle; CHUNK_SIZE as usize * CHUNK_SIZE as usize],
    pub(crate) dirty: Option<GridBounds>,
    pub(crate) update_this_frame: Option<GridBounds>,
    pub(crate) updated_last_frame: Option<GridBounds>,
}

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


impl Chunk {
    pub fn new(position: GridVec) -> Self {
        let created = Chunk {
            position,
            neighbors: Neighbors::new(),
            particles: [Particle::default(); CHUNK_SIZE as usize *  CHUNK_SIZE as usize],
            dirty: None,
            update_this_frame: None,
            updated_last_frame: None,
        };

        //println!("Created new chunk at {}", position);

        return created;
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

    fn test_vec(&self, base_x: u8, base_y: u8, test_vec_x: i8, test_vec_y: i8, replace_water: bool) -> bool {
        let test_pos_x = base_x as i16 + test_vec_x as i16;
        let test_pos_y = base_y as i16 + test_vec_y as i16;

        let test_particle;
        
        if self.contains(test_pos_x, test_pos_y) {
            test_particle = self.get_particle(test_pos_x as u8, test_pos_y as u8);
        }
        else {
            let neighbor_direction = Chunk::get_oob_direction(test_pos_x, test_pos_y);
            let neighbor = self.get_neighbor(neighbor_direction);
            if neighbor.is_none() {
                return false;
            }
            else {
                let chunk = neighbor.unwrap();
                let mut other_chunk_x = test_pos_x % (CHUNK_SIZE as i16);
                let mut other_chunk_y = test_pos_y % (CHUNK_SIZE as i16);
                if other_chunk_x < 0 { other_chunk_x += CHUNK_SIZE as i16; }
                if other_chunk_y < 0 { other_chunk_y += CHUNK_SIZE as i16; }
                unsafe {
                    test_particle = (*chunk).get_particle(other_chunk_x as u8, other_chunk_y as u8);
                }
            }
        }

        if test_particle.updated_this_frame { 
            // Need to allow things to fall into spaces otherwise weird air bubbles are allowed to persist
            return test_vec_y < 0; 
        }
        if test_particle.particle_type == ParticleType::Air { return true; }
        else if replace_water && test_particle.particle_type == ParticleType::Water { return true; }
        return false;
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
    }

    pub fn chunkpos_to_local_chunkpos(&self, from_chunk: &Chunk, from_x: u8, from_y: u8) -> GridVec {
        let chunk_diff = (from_chunk.position - self.position) * CHUNK_SIZE as i32;
        GridVec::new(from_x as i32, from_y as i32) + chunk_diff
    }

    pub fn set_particle(&mut self, x: u8, y: u8, val: Particle) {
        self.particles[Chunk::get_index_in_chunk(x, y)] = val;
        self.mark_dirty(x as i32, y as i32);
        self.particles[Chunk::get_index_in_chunk(x, y)].updated_this_frame = true;
    }

    pub fn mark_dirty(&mut self, x: i32, y: i32) {
        let chunk_bounds = GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));
        let dirty_bounds = chunk_bounds.intersect(GridBounds::new(GridVec { x, y }, GridVec { x: 4, y: 4 }));

        self.dirty = chunk_bounds.intersect_option(GridBounds::option_union(self.dirty, dirty_bounds));

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

    pub fn add_particle(&mut self, x: u8, y: u8, val: Particle) {
        if self.get_particle(x, y).particle_type == ParticleType::Air {
            self.particles[Chunk::get_index_in_chunk(x, y)] = val;
            self.mark_dirty(x as i32, y as i32);
        }
    }

    fn _is_border(x: u8, y: u8) -> bool {
        x == 0 || y == 0 || x == CHUNK_SIZE - 1  || y == CHUNK_SIZE - 1
    }

    pub(crate) fn commit_updates(&mut self) {
        self.update_this_frame = self.dirty;
        self.dirty = None;

        if let Some(to_update) = GridBounds::option_union(self.update_this_frame, self.updated_last_frame) {
            for point in to_update.slide_iter() {
                let x = point.x as u8;
                let y = point.y as u8;
                
                self.get_particle_mut(x, y).updated_this_frame = false;
            }
        }
    }

    fn try_erode(&mut self, rng: &mut ThreadRng, x: i16, y: i16, vel: &GridVec) {
        if self.contains(x, y) {
            let part = self.get_particle(x as u8, y as u8);
            if !part.updated_this_frame {
                match part.particle_type {
                    ParticleType::Sand => {
                        let next_x = x as i16 + vel.x as i16;
                        let next_y = y as i16 + vel.y as i16;
                        if self.contains(next_x, next_y) && rng.gen_bool(0.2) {
                            self.set_particle(x as u8, y as u8, self.get_particle(next_x as u8, next_y as u8));
                            self.set_particle(next_x as u8, next_y as u8, part);
                        }
                    }
                    ParticleType::Stone => {
                        if rng.gen_bool(0.01) {
                            self.set_particle(x as u8, y as u8, Particle { particle_type: ParticleType::Sand, updated_this_frame: true })
                        }
                    }
                    _ => ()
                }
            }
        }
        else if let Some(neighbor) = self.get_neighbor( Chunk::get_oob_direction(x, y) ) {
            unsafe {
                (*neighbor).try_erode(rng, x % CHUNK_SIZE as i16, y % CHUNK_SIZE as i16, vel);
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

                if !cur_part.updated_this_frame {
                    if cur_part.particle_type == ParticleType::Source {
                        if x > 0 { self.add_particle(x - 1, y, Particle::new(ParticleType::Water)); }
                        if x < CHUNK_SIZE - 1 { self.add_particle(x + 1, y, Particle::new(ParticleType::Water)); }
                        if y > 0 { self.add_particle(x, y - 1, Particle::new(ParticleType::Water)); }
                        if y < CHUNK_SIZE - 1 { self.add_particle(x, y + 1, Particle::new(ParticleType::Water)); }
                    }
                    
                    let available_moves = Particle::get_possible_moves(cur_part.particle_type);
                    if available_moves.len() > 0 {
                        let mut possible_moves = Vec::<GridVec>::new();
                        let can_replace_water = Particle::can_replace_water(cur_part.particle_type);
                        
                        for vec in available_moves {
                            if self.test_vec(x, y, vec.x as i8, vec.y as i8, can_replace_water) {
                                possible_moves.push(vec.clone());
                            }
                        }
                        
                        if possible_moves.len() > 0 {
                            let chosen_vec = possible_moves[rng.gen_range(0..possible_moves.len())];
                            let chosen_x = x as i16 + chosen_vec.x as i16;
                            let chosen_y = y as i16 + chosen_vec.y as i16;

                            if cur_part.particle_type == ParticleType::Water && chosen_vec.manhattan_length() > 1 {
                                self.try_erode(&mut rng, x as i16, y as i16 - 1, &chosen_vec);
                                self.try_erode(&mut rng, x as i16, y as i16 + 1, &chosen_vec);
                                self.try_erode(&mut rng, x as i16 - 1, y as i16, &chosen_vec);
                                self.try_erode(&mut rng, x as i16 + 1, y as i16, &chosen_vec);
                            }

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