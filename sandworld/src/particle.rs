use gridmath::GridVec;

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
    pub particle_type: ParticleType,
    pub(crate) updated_this_frame: bool,
}

impl Particle {
    pub fn new(particle_type: ParticleType) -> Self {
        Particle{particle_type, updated_this_frame: false}
    }

    pub fn get_possible_moves(particle_type: ParticleType) -> Vec::<GridVec> {
        match particle_type {
            ParticleType::Sand => vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -1}],
            ParticleType::Water => vec![GridVec{x: 1, y: -1}, GridVec{x: 0, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0} ],
            _ => Vec::<GridVec>::new(),
        }
    }

    pub fn can_replace_water(particle_type: ParticleType) -> bool {
        match particle_type {
            ParticleType::Sand => true,
            _ => false,
        }
    }
}

impl Default for Particle {
    fn default() -> Self { Particle{particle_type: ParticleType::Air, updated_this_frame: false} }
}