
use noise::{NoiseFn, Perlin};
use sandworld::{WorldGenerator, Particle, ParticleType};
use gridmath::GridVec;


pub struct Blankworld {}

pub struct FlatPlain {
    pub stone_height: i32,
    pub sand_height: i32,
}

pub struct BasicPerlin {
    noise: Perlin,
    threshold: f64,
    scale_x: f64,
    scale_y: f64,
}

impl WorldGenerator for Blankworld {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        Particle::new(ParticleType::Air)
    }
}

impl WorldGenerator for FlatPlain {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        Particle::new(
            if world_pos.y > self.stone_height { 
                if world_pos.y > self.sand_height {
                    ParticleType::Air 
                }
                else {
                    ParticleType::Sand
                }
            } 
            else { 
                ParticleType::Stone 
            }
        )
        
    }
}

impl BasicPerlin {
    pub fn new(seed: u32) -> Self {
        BasicPerlin {
            noise: Perlin::new(seed),
            threshold: 0.05,
            scale_x: 0.01,
            scale_y: 0.01,
        }
    }
}

impl WorldGenerator for BasicPerlin {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        let sample_pos = [world_pos.x as f64 * self.scale_x, world_pos.y as f64 * self.scale_y];
        let noise_val = self.noise.get(sample_pos);
            
        Particle::new(if noise_val < self.threshold { ParticleType::Air } else { ParticleType::Stone }) 
    }
}