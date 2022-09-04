use crate::gridmath::*;
use rand::Rng;
use std::collections::HashMap;

const CHUNK_SIZE: u8 = 64;

pub struct World {
    // TODO keep chunks at consistent addresses once allocated
    // option - box all the chunks, store the boxes?
    // option - static self managed array with map of coord to index
    chunks: HashMap<u64, Box<Chunk>>,
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

struct NeighborIterator<'a> {
    neighbor_array: [&'a Option<*mut Chunk>; 8],
    cur_index: usize,
}

struct Chunk {
    position: GridVec,
    neighbors: Neighbors,
    particles: [Particle; CHUNK_SIZE as usize * CHUNK_SIZE as usize],
    dirty: Option<GridBounds>,
    update_this_frame: Option<GridBounds>,
    updated_last_frame: Option<GridBounds>,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ParticleType {
    Air,
    Sand,
    Water,
    Stone,
    Source,
    Boundary,
    Dirty,
}

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub particle_type: ParticleType
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

impl Particle {
    pub fn new(particle_type: ParticleType) -> Self {
        Particle{particle_type}
    }

    fn get_possible_moves(particle_type: ParticleType) -> Vec::<GridVec> {
        match particle_type {
            ParticleType::Sand => vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -1}],
            ParticleType::Water => vec![GridVec{x: 1, y: -1}, GridVec{x: 0, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0} ],
            _ => Vec::<GridVec>::new(),
        }
    }

    fn can_replace_water(particle_type: ParticleType) -> bool {
        match particle_type {
            ParticleType::Sand => true,
            _ => false,
        }
    }
}

impl Default for Particle {
    fn default() -> Self { Particle{particle_type: ParticleType::Air} }
}

impl Chunk {
    fn new(position: GridVec) -> Self {
        let created = Chunk {
            position,
            neighbors: Neighbors::new(),
            particles: [Particle::default(); CHUNK_SIZE as usize *  CHUNK_SIZE as usize],
            dirty: None,
            update_this_frame: None,
            updated_last_frame: None,
        };

        return created;
    }

    fn chunkpos_to_local_chunkpos(&self, from_chunk: &Chunk, from_x: u8, from_y: u8) -> GridVec {
        let chunk_diff = (from_chunk.position - self.position) * CHUNK_SIZE as i32;
        GridVec::new(from_x as i32, from_y as i32) + chunk_diff
    }
    
    fn get_index_in_chunk(x: u8, y: u8) -> usize {
        return y as usize * CHUNK_SIZE as usize + x as usize;
    }

    fn get_particle(&self, x: u8, y: u8) -> Particle {
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

    fn set_particle(&mut self, x: u8, y: u8, val: Particle) {
        self.particles[Chunk::get_index_in_chunk(x, y)] = val;
        self.mark_dirty(x as i32, y as i32);
    }

    fn mark_dirty(&mut self, x: i32, y: i32) {
        let chunk_bounds = GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));
        let dirty_bounds = chunk_bounds.intersect(GridBounds::new(GridVec { x, y }, GridVec { x: 4, y: 4 }));

        if let Some(cur_bounds) = self.dirty {
            if let Some(add_bounds) = dirty_bounds {
                self.dirty = Some(cur_bounds.union(add_bounds));
            }
        }
        else {
            self.dirty = dirty_bounds;
        }

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

    fn add_particle(&mut self, x: u8, y: u8, val: Particle) {
        if self.get_particle(x, y).particle_type == ParticleType::Air {
            self.particles[Chunk::get_index_in_chunk(x, y)] = val;
            self.mark_dirty(x as i32, y as i32);
        }
    }

    fn contains(&self, x: i16, y: i16) -> bool {
        x >= 0 && y >= 0 && x < CHUNK_SIZE as i16 && y < CHUNK_SIZE as i16
    }

    fn _is_border(x: u8, y: u8) -> bool {
        x == 0 || y == 0 || x == CHUNK_SIZE - 1  || y == CHUNK_SIZE - 1
    }

    fn get_neighbor(&self, dir: GridVec) -> Option<*mut Chunk> {
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

        let material_at_test;
        
        if self.contains(test_pos_x, test_pos_y) {
            material_at_test = self.get_particle(test_pos_x as u8, test_pos_y as u8).particle_type;
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
                    material_at_test = (*chunk).get_particle(other_chunk_x as u8, other_chunk_y as u8).particle_type;
                }
            }
        }

        if material_at_test == ParticleType::Air { return true; }
        else if replace_water && material_at_test == ParticleType::Water { return true; }
        return false;
    }

    fn commit_updates(&mut self) {
        self.update_this_frame = self.dirty;
        self.dirty = None;
    }

    fn update(&mut self) {      
        let mut rng = rand::thread_rng();

        if let Some(to_update) = GridBounds::option_union(self.update_this_frame, self.updated_last_frame) {
            for point in to_update.slide_iter() {
                let x = point.x as u8;
                let y = point.y as u8;
                
                let cur_part = self.get_particle(x, y);
                
                let available_moves = Particle::get_possible_moves(cur_part.particle_type);
                
                if cur_part.particle_type == ParticleType::Source {
                    if x > 0 { self.add_particle(x - 1, y, Particle::new(ParticleType::Water)); }
                    if x < CHUNK_SIZE - 1 { self.add_particle(x + 1, y, Particle::new(ParticleType::Water)); }
                    if y > 0 { self.add_particle(x, y - 1, Particle::new(ParticleType::Water)); }
                    if y < CHUNK_SIZE - 1 { self.add_particle(x, y + 1, Particle::new(ParticleType::Water)); }
                }

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
                        if self.contains(chosen_x, chosen_y) {
                            self.set_particle(x, y, self.get_particle(chosen_x as u8, chosen_y as u8));
                            self.set_particle(chosen_x as u8, chosen_y as u8, cur_part);
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
                                    (*chunk).set_particle(other_chunk_x as u8, other_chunk_y as u8, cur_part);
                                }
                            }
                        }
                    }
                }
            }
        }

        self.updated_last_frame = self.update_this_frame;
    }

    fn check_add_neighbor(&mut self, new_chunk: &mut Chunk) {
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

    fn _check_remove_neighbor(&mut self, removed_position: GridVec) {
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
            return Particle { particle_type: ParticleType::Boundary };
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
        self.place_circle(pos, radius, Particle{particle_type:ParticleType::Air}, true);
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
        let mut outbuffer: Vec<Particle> = Vec::new();
        let top_right = screen_bounds.top_right();
        let bottom_left = screen_bounds.bottom_left();
        let buffer_height = top_right.y - bottom_left.y;
        let buffer_width = top_right.x - bottom_left.x;
        outbuffer.resize(buffer_height as usize * buffer_width as usize, Particle::new(ParticleType::Boundary));

        for (chunk_pos_combined, chunk) in self.chunks.iter() {
            let chunk_pos = GridVec::decombined(*chunk_pos_combined);
            let chunk_bottom_left = chunk_pos * CHUNK_SIZE as i32;
            let chunk_bounds = GridBounds::new_from_corner(chunk_bottom_left, GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));

            if let Some(overlap) = screen_bounds.intersect(chunk_bounds) {
                for overlap_pos in overlap.iter() {
                    let chunk_local = World::get_chunklocal(overlap_pos);
                    let buffer_local = overlap_pos - bottom_left;
                    let buffer_index = (buffer_local.x + (buffer_width * buffer_local.y)) as usize;

                    outbuffer[buffer_index] = chunk.get_particle(chunk_local.x as u8, chunk_local.y as u8);

                    if draw_debug {
                        if chunk_local.x == 0 || chunk_local.y == 0 || chunk_local.x as u8 == CHUNK_SIZE - 1 || chunk_local.y as u8 == CHUNK_SIZE - 1 {
                            outbuffer[buffer_index] = Particle::new(ParticleType::Boundary);
                        }
                        if let Some(updated_bounds) = chunk.updated_last_frame {
                            if updated_bounds.is_boundary(chunk_local){
                                outbuffer[buffer_index] = Particle::new(ParticleType::Dirty);
                            }
                        }
                    }
                }
            }
        }

        return outbuffer;
    }

    pub fn update(&mut self) -> u64 {
        let mut updated_count = 0;
        for (_pos, chunk) in self.chunks.iter_mut() {
            chunk.commit_updates();
        }

        for (_pos, chunk) in self.chunks.iter_mut() {
            if chunk.update_this_frame.is_some() { 
                updated_count += 1;
                chunk.update(); 
            }
            else {
                chunk.updated_last_frame = None;
            }
        }
        updated_count
    }
}