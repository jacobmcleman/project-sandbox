use crate::gridmath::*;
use rand::Rng;
use std::collections::HashMap;

const CHUNK_SIZE: usize = 64;

pub struct World {
    chunks: HashMap<u64, Chunk>,
}

struct Chunk {
    position: GridVec,
    particles: [Particle; CHUNK_SIZE * CHUNK_SIZE],
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ParticleType {
    Boundary,
    Air,
    Sand,
    Water,
    Stone,
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
        Chunk {
            position,
            particles: [Particle::default(); CHUNK_SIZE *  CHUNK_SIZE],
        }
    }
    
    fn get_index_in_chunk(pos: GridVec) -> usize {
        return pos.y as usize * CHUNK_SIZE as usize + pos.x as usize;
    }

    fn get_particle(&self, pos: GridVec) -> Particle {
        return self.particles[Chunk::get_index_in_chunk(pos)];
    }

    fn set_particle(&mut self, pos: GridVec, val: Particle) {
        self.particles[Chunk::get_index_in_chunk(pos)] = val;
    }

    fn add_particle(&mut self, pos: GridVec, val: Particle) {
        if self.get_particle(pos).particle_type == ParticleType::Air {
            self.particles[Chunk::get_index_in_chunk(pos)] = val;
        }
    }

    fn contains(&self, pos: GridVec) -> bool {
        pos.x >= 0 && pos.x < CHUNK_SIZE as i32 && pos.y >= 0 && pos.y < CHUNK_SIZE as i32
    }

    fn test_vec(&self, base_pos: GridVec, test_vec: GridVec, replace_water: bool) -> bool {
        let test_pos = base_pos + test_vec;
        if !self.contains(test_pos) { return false; }

        let material_at_test = self.get_particle(test_pos).particle_type;

        if material_at_test == ParticleType::Air { return true; }
        else if replace_water && material_at_test == ParticleType::Water { return true; }
        return false;
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        
        for y in 0..CHUNK_SIZE as i32 {
            let flip = rng.gen_bool(0.5);
            for mut x in 0..CHUNK_SIZE as i32 {
                if flip { x = CHUNK_SIZE as i32 - x - 1; }

                let base_pos = GridVec{x, y};
                let cur_part = self.get_particle(base_pos);

                let available_moves = Particle::get_possible_moves(cur_part.particle_type);

                if available_moves.len() > 0 {
                    let mut possible_moves = Vec::<GridVec>::new();
                    let can_replace_water = Particle::can_replace_water(cur_part.particle_type);

                    for vec in available_moves {
                        if self.test_vec(base_pos, vec, can_replace_water) {
                            possible_moves.push(vec.clone());
                        }
                    }

                    if possible_moves.len() > 0 {
                        let chosen_vec = possible_moves[rng.gen_range(0..possible_moves.len())];
                        let chosen_pos = base_pos + chosen_vec;
                        self.set_particle(base_pos, self.get_particle(chosen_pos));
                        self.set_particle(chosen_pos, cur_part);
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
        GridVec::new(pos.x % CHUNK_SIZE as i32, pos.y % CHUNK_SIZE as i32)
    }

    pub fn get_particle(&self, pos: GridVec) -> Particle {
        if !self.contains(pos) {
            return Particle { particle_type: ParticleType::Boundary };
        }

        let chunk_pos = World::get_chunkpos(pos);
        let chunklocal = World::get_chunklocal(pos);
        return self.chunks.get(&chunk_pos.combined()).unwrap().get_particle(chunklocal);
    }

    pub fn replace_particle(&mut self, pos: GridVec, new_val: Particle) {
        if !self.contains(pos) {
            return;
        }

        let chunk_pos = World::get_chunkpos(pos);
        let chunklocal = World::get_chunklocal(pos);
        self.chunks.get_mut(&chunk_pos.combined()).unwrap().set_particle(chunklocal, new_val);
    }

    pub fn add_particle(&mut self, pos: GridVec, new_val: Particle) {
        if !self.contains(pos) {
            return;
        }

        let chunk_pos = World::get_chunkpos(pos);
        let chunklocal = World::get_chunklocal(pos);
        self.chunks.get_mut(&chunk_pos.combined()).unwrap().add_particle(chunklocal, new_val);
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

    pub fn update(&mut self) {
        let mut rng = rand::thread_rng();
        
        
        for (_pos, chunk) in self.chunks.iter_mut() {
            chunk.update();
        }
    }
}