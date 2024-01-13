use bevy::ecs::world;
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

pub struct WorldBuilder {
    noise: Perlin,
    terrain_scale: f64,
    terrain_height: f64,
    cave_scale: f64,
    lava_scale: f64,
    lava_depth: f64, 
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

impl WorldBuilder {
    pub fn new(seed: u32, terrain_scale: f64, terrain_height: f64, cave_scale: f64, lava_depth: f64, lava_scale: f64) -> Self {
        WorldBuilder {
            noise: Perlin::new(seed),
            terrain_scale,
            terrain_height,
            cave_scale,
            lava_depth: lava_depth * 100.,
            lava_scale,
        }
    }
}

impl WorldGenerator for WorldBuilder {
    fn get_particle(&self, world_pos: GridVec) -> Particle {
        let detail_scale_1 = 8.7;
        let detail_scale_2 = 7.8;
        let detail_scale_3 = 9.3;
        let terrain_sample_pos_broad = [world_pos.x as f64 / self.terrain_scale, 0.];
        let terrain_sample_pos_detail_1 = [world_pos.x as f64 / (self.terrain_scale / detail_scale_1), 0.1];
        let terrain_sample_pos_detail_2 = [world_pos.x as f64 / (self.terrain_scale / (detail_scale_1 * detail_scale_2)), 0.2];
        let terrain_sample_pos_detail_3 = [world_pos.x as f64 / (self.terrain_scale / (detail_scale_1 * detail_scale_2 * detail_scale_3)), 0.3];

        let terrain_height_broad = self.noise.get(terrain_sample_pos_broad) * self.terrain_height;
        let terrain_height_detail_1 = self.noise.get(terrain_sample_pos_detail_1) * self.terrain_height / detail_scale_1;
        let terrain_height_detail_2 = self.noise.get(terrain_sample_pos_detail_2) * self.terrain_height / (detail_scale_1 * detail_scale_2);
        let terrain_height_detail_3 = self.noise.get(terrain_sample_pos_detail_3) * self.terrain_height / (detail_scale_1 * detail_scale_2 * detail_scale_3);
        let terrain_height = terrain_height_broad + terrain_height_detail_1 + terrain_height_detail_2 + terrain_height_detail_3;

        // let detail_sample_pos  = [world_pos.x as f64 / self.detail_scale, world_pos.y as f64 / self.detail_scale];

        let from_terrain = world_pos.y as f64 - terrain_height;

        Particle::new(
            if from_terrain < 0. {
                let cave_sample_pos = [world_pos.x as f64 / self.cave_scale / 2., world_pos.y as f64 / self.cave_scale];
                let cave_detail_pos = [world_pos.x as f64 / (self.cave_scale / detail_scale_1), world_pos.y as f64 / (self.cave_scale / detail_scale_1)];
                let cave_base_value = self.noise.get(cave_sample_pos).powi(2);
                let cave_detail_value = self.noise.get(cave_detail_pos).abs();
                let cave_value = cave_base_value + 0.03 * (cave_detail_value / detail_scale_1);

                let surface_avoidance = if world_pos.y <= 0 && terrain_height < 0. {
                    (from_terrain / -256.).clamp(0., 1.)
                }
                else {
                    1.
                };

                if cave_value + 0.001 < 0.01 * surface_avoidance {
                    ParticleType::Air
                }
                else {
                    let from_cave = (0.8 - ((0.01 * surface_avoidance) / (cave_value + 0.001))).clamp(0., 1.);
                    let lava_mult = from_cave * (-from_terrain / self.lava_depth).abs();

                    let lava_sample_pos = [world_pos.x as f64 / self.lava_scale, world_pos.y as f64 / self.lava_scale * 2.];
                    let lava_noise_val = self.noise.get(lava_sample_pos).powi(2);

                    if lava_noise_val + 0.01 < 0.1 * lava_mult {
                        ParticleType::Lava
                    }
                    else {
                        ParticleType::Stone
                    }
                }
            }
            else {
                // Filled in, but ripple surface
                if world_pos.y < 0 || (world_pos.y == 0 && world_pos.x % 3 == 0) {
                    ParticleType::Water
                }
                else {
                    ParticleType::Air
                }
            }
        )
    }
}