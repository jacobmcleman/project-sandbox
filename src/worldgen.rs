use gridmath::GridVec;
use noise::{NoiseFn, Perlin};
use sandworld::{Particle, ParticleType, WorldGenerator};

pub struct Blankworld {}

pub struct FlatPlain {
    pub stone_height: i32,
    pub sand_height: i32,
}

pub struct BasicPerlin {
    noise: Perlin,
    stone_threshold: f64,
    scale_x: f64,
    scale_y: f64,
}

pub struct LayeredPerlin {
    noise: Perlin,
    scale_macro: f64,
    scale_detail: f64,
    cave_noisiness: f64,
}

impl WorldGenerator for Blankworld {
    fn get_particle(&self, _world_pos: GridVec) -> Particle {
        Particle::new(ParticleType::Air)
    }
}

impl WorldGenerator for FlatPlain {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        Particle::new(if world_pos.y > self.stone_height {
            if world_pos.y > self.sand_height {
                ParticleType::Air
            } else {
                ParticleType::Sand
            }
        } else {
            ParticleType::Stone
        })
    }
}

impl BasicPerlin {
    pub fn new(seed: u32, scale: f64) -> Self {
        BasicPerlin {
            noise: Perlin::new(seed),
            stone_threshold: 0.05,
            scale_x: scale,
            scale_y: scale,
        }
    }
}

impl WorldGenerator for BasicPerlin {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        let sample_pos = [
            world_pos.x as f64 * self.scale_x,
            world_pos.y as f64 * self.scale_y,
        ];
        let noise_val = self.noise.get(sample_pos);

        Particle::new(if noise_val < self.stone_threshold {
            ParticleType::Air
        } else {
            ParticleType::Stone
        })
    }
}

impl LayeredPerlin {
    pub fn new(seed: u32, scale_macro: f64, scale_detail: f64, cave_noisiness: f64) -> Self {
        LayeredPerlin {
            noise: Perlin::new(seed),
            scale_macro,
            scale_detail,
            cave_noisiness,
        }
    }
}

impl WorldGenerator for LayeredPerlin {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        let macro_sample_pos = [
            world_pos.x as f64 * self.scale_macro,
            world_pos.y as f64 * self.scale_macro,
        ];
        let detail_sample_pos = [
            world_pos.x as f64 * self.scale_detail,
            world_pos.y as f64 * self.scale_detail,
        ];
        let lava_sample_pos = [
            world_pos.x as f64 * self.scale_detail * 0.13,
            world_pos.y as f64 * self.scale_detail * 0.21,
        ];

        //let cave_sample_pos = [world_pos.x as f64 * self.scale_cave_density, world_pos.y as f64 * self.scale_cave_density];
        let macro_noise_val = self.noise.get(macro_sample_pos) / 2. + 1.;
        let detail_noise_val = self.noise.get(detail_sample_pos) / 2. + 1.;
        //let cave_sample_noise_val = self.noise.get(cave_sample_pos);

        let lava_noise_val = self.noise.get(lava_sample_pos).powi(2);
        
        Particle::new(
            if detail_noise_val < macro_noise_val * self.cave_noisiness {
                ParticleType::Air
            } else {
                let surf_dist = 0.90 - (macro_noise_val * self.cave_noisiness / detail_noise_val);
                if lava_noise_val < 0.005 * surf_dist {
                    ParticleType::Lava
                }
                else {
                    ParticleType::Stone
                }
            },
        )
    }
}
