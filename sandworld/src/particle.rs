use gridmath::GridVec;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ParticleType {
    Air,
    Sand,
    Water,
    Stone,
    Source,
    Boundary,
    RegionBoundary,
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

    pub fn get_possible_moves(particle_type: ParticleType) -> Vec::<Vec::<GridVec>> {
        match particle_type {
            ParticleType::Sand => vec![
                vec![GridVec{x: 0, y: -1}, GridVec{x: 0, y: -2}, GridVec{x: 0, y: -3}], 
                vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}],
                ],
            ParticleType::Water => vec![
                vec![GridVec{x: 1, y: -2}, GridVec{x: -1, y: -2}, GridVec{x: 0, y: -2}, GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -1}],
                vec![GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0}, GridVec{x: 2, y: -1}, GridVec{x: -2, y: -1}, GridVec{x: 2, y: 0}, GridVec{x: -2, y: 0}, GridVec{x: 3, y: -1}, GridVec{x: -3, y: -1}],
                vec![GridVec{x: 3, y: 0}, GridVec{x: -3, y: 0}, GridVec{x: 5, y: -1}, GridVec{x: -5, y: -1}, GridVec{x: 5, y: 0}, GridVec{x: -5, y: 0}, GridVec{x: 5, y: -1}, GridVec{x: -5, y: -1}],
                ],
            _ => Vec::<Vec::<GridVec>>::new(),
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


pub fn get_color_for_type(particle_type: ParticleType) -> [u8; 4] {
    match particle_type {
        ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
        ParticleType::Water => [0x56, 0x9c, 0xd6, 0xff],
        ParticleType::Stone => [0xd4, 0xd4, 0xd4, 0xff],
        ParticleType::Air => [0x1e, 0x1e, 0x1e, 0xff],
        ParticleType::Source => [0xf7, 0xdf, 0x00, 0xff],
        ParticleType::Dirty => [0xFF, 0x00, 0xFF, 0xff],
        ParticleType::RegionBoundary => [0xFF, 0xFF, 0x00, 0xFF],
        _ => [0x00, 0x00, 0x00, 0xff],
    }
}