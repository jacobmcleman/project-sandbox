use std::sync::{Arc, Mutex};

use noise::{Simplex, NoiseFn, Perlin};
use sandworld::{Particle, ParticleType};
use gridmath::GridVec;

pub fn blankworld_generator(_worldpos: GridVec) -> Particle {
    Particle::new(ParticleType::Air)
}

pub fn stone_plain(worldpos: GridVec) -> Particle {
    Particle::new(if worldpos.y > 0 { ParticleType::Air } else { ParticleType::Stone })
}

pub fn basic_simplex(worldpos: GridVec) -> Particle {
    let sample_pos = [worldpos.x as f64 * 0.01, worldpos.y as f64 * 0.01];
    let noise_val = Simplex::new(0).get(sample_pos);
    
    Particle::new(if noise_val < 0.05 { ParticleType::Air } else { ParticleType::Stone })    
}

pub fn basic_perlin(worldpos: GridVec) -> Particle {
    let noise = Perlin::new(0);
    let sample_pos = [worldpos.x as f64 * 0.01, worldpos.y as f64 * 0.01];
    let noise_val = noise.get(sample_pos);
        
    Particle::new(if noise_val < 0.05 { ParticleType::Air } else { ParticleType::Stone })   
}