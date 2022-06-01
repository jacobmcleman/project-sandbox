use crate::gridmath::*;
use rand::Rng;
use std::collections::HashMap;

const CHUNK_SIZE: u8 = 64;

pub struct World {
    chunks: HashMap<u64, Chunk>,
}

struct Chunk {
    position: GridVec,
    particles: [Particle; CHUNK_SIZE as usize * CHUNK_SIZE as usize],
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ParticleType {
    Air,
    Sand,
    Water,
    Stone,
    Boundary,
}

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub particle_type: ParticleType
}

impl Particle {
    pub fn new(particle_type: ParticleType) -> Self {
        Particle{particle_type}
    }

    fn get_possible_moves(particle_type: ParticleType) -> Vec::<GridVec> {
        match particle_type {
            ParticleType::Sand => vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -1}],
            ParticleType::Water => vec![GridVec{x: 1, y: -1}, GridVec{x: 0, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 1, y: 0}, GridVec{x: 0, y: -2}, GridVec{x: -1, y: 0} ],
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
        let mut created = Chunk {
            position,
            particles: [Particle::default(); CHUNK_SIZE as usize *  CHUNK_SIZE as usize],
        };

        created.particles[16] = Particle::new(ParticleType::Sand);
        return created;
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
    }

    fn add_particle(&mut self, x: u8, y: u8, val: Particle) {
        if self.get_particle(x, y).particle_type == ParticleType::Air {
            self.particles[Chunk::get_index_in_chunk(x, y)] = val;
        }
    }

    fn contains(&self, x: i16, y: i16) -> bool {
        x >= 0 && y >= 0 && x < CHUNK_SIZE as i16 && y < CHUNK_SIZE as i16
    }

    fn test_vec(&self, base_x: u8, base_y: u8, test_vec_x: i8, test_vec_y: i8, replace_water: bool) -> bool {
        let test_pos_x = base_x as i16 + test_vec_x as i16;
        let test_pos_y = base_y as i16 + test_vec_y as i16;
        if !self.contains(test_pos_x, test_pos_y) { return false; }

        let material_at_test = self.get_particle(test_pos_x as u8, test_pos_y as u8).particle_type;

        if material_at_test == ParticleType::Air { return true; }
        else if replace_water && material_at_test == ParticleType::Water { return true; }
        return false;
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        
        for y in 0..CHUNK_SIZE {
            let flip = rng.gen_bool(0.5);
            for mut x in 0..CHUNK_SIZE {
                if flip { x = CHUNK_SIZE - x - 1; }

                let cur_part = self.get_particle(x, y);

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
                        let chosen_x = (x as i16 + chosen_vec.x as i16) as u8;
                        let chosen_y = (y as i16 + chosen_vec.y as i16) as u8;
                        self.set_particle(x, y, self.get_particle(chosen_x, chosen_y));
                        self.set_particle(chosen_x, chosen_y, cur_part);
                    }
                }
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
                created.chunks.insert(chunkpos.combined(), Chunk::new(chunkpos));
            }
        }

        return created;
    }


    pub fn contains(&self, pos: GridVec) -> bool {
        let chunk_pos = World::get_chunkpos(pos);
        return self.chunks.contains_key(&chunk_pos.combined());
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

    pub fn get_particle(&self, pos: GridVec) -> Particle {
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

    pub fn render(&self, screen_bounds: &GridBounds) -> Vec<Particle> {
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
                }
            }
        }

        return outbuffer;
    }

    pub fn update(&mut self) {
        for (_pos, chunk) in self.chunks.iter_mut() {
            chunk.update();
        }
    }
}