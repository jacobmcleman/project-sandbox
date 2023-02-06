use gridmath::GridVec;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ParticleType {
    Air,
    Sand,
    Water,
    Stone,
    Gravel,
    Steam,
    Lava,
    Ice,
    Source,
    LSource,
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
                vec![GridVec{x: 0, y: -1}, GridVec{x: 0, y: -2}], 
                vec![GridVec{x: -1, y: -1}, GridVec{x: 1, y: -1}, GridVec{x: 2, y: -1}, GridVec{x: -2, y: -1}],
                ],
            ParticleType::Gravel => vec![
                vec![GridVec{x: 0, y: -4}, GridVec{x: 0, y: -2}, GridVec{x: 0, y: -3}],                 
                vec![GridVec{x: 0, y: -1}], 
                vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}],
                ],
            ParticleType::Water => vec![
                vec![GridVec{x: 1, y: -2}, GridVec{x: -1, y: -2}, GridVec{x: 0, y: -2}, GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -1}],
                vec![GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0}, GridVec{x: 2, y: -1}, GridVec{x: -2, y: -1}, GridVec{x: 2, y: 0}, GridVec{x: -2, y: 0}, GridVec{x: 3, y: -1}, GridVec{x: -3, y: -1}],
                vec![GridVec{x: 3, y: 0}, GridVec{x: -3, y: 0}, GridVec{x: 5, y: -1}, GridVec{x: -5, y: -1}, GridVec{x: 5, y: 0}, GridVec{x: -5, y: 0}, GridVec{x: 5, y: -1}, GridVec{x: -5, y: -1}],
                ],
            ParticleType::Steam => vec![
                vec![GridVec{x: 1, y: 2}, GridVec{x: -1, y: 2}, GridVec{x: 0, y: 2}, GridVec{x: 1, y: 1}, GridVec{x: -1, y: 1}, GridVec{x: 0, y: 1}],
                vec![GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0}, GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 2, y: 0}, GridVec{x: -2, y: 0}, GridVec{x: 2, y: 1}, GridVec{x: 2, y: 11}],
                ],
            ParticleType::Lava => vec![
                vec![GridVec{x: 1, y: -2}, GridVec{x: -1, y: -2}, GridVec{x: 0, y: -2}, GridVec{x: 0, y: -1}],
                vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0}, GridVec{x: 2, y: -1}, GridVec{x: -2, y: -1}, GridVec{x: 2, y: 0}, GridVec{x: -2, y: 0}, GridVec{x: 3, y: -1}, GridVec{x: -3, y: -1}],
                ],
            _ => Vec::<Vec::<GridVec>>::new(),
        }
    }

    pub fn get_can_replace(particle_type: ParticleType, replace_type: ParticleType) -> bool {
        match particle_type {
            ParticleType::Sand => [ParticleType::Water, ParticleType::Steam].contains(&replace_type),
            ParticleType::Gravel => [ParticleType::Water, ParticleType::Steam, ParticleType::Lava].contains(&replace_type),
            ParticleType::Steam => [ParticleType::Water, ParticleType::Lava].contains(&replace_type),
            ParticleType::Lava => [ParticleType::Water, ParticleType::Steam, ParticleType::Sand].contains(&replace_type),
            _ => false
        }
    }
}

impl Default for Particle {
    fn default() -> Self { Particle{particle_type: ParticleType::Air, updated_this_frame: false} }
}


pub fn get_color_for_type(particle_type: ParticleType) -> [u8; 4] {
    match particle_type {
        ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
        ParticleType::Water => [0x6d, 0x95, 0xc9, 0xff], // #6d95c9
        ParticleType::Gravel => [0xa9, 0xa3, 0xb5, 0xff], // #a9a3b5
        ParticleType::Stone => [0x6b, 0x6f, 0x75, 0xff], //#6b6f75
        ParticleType::Steam => [0xe6, 0xec, 0xf0, 0xff], //#e6ecf0
        ParticleType::Lava => [0xf0, 0x95, 0x16, 0xff], //#f09516
        ParticleType::Ice => [0xbf, 0xdb, 0xff, 0xff], //#bfdbff
        ParticleType::Air => [0x1e, 0x1e, 0x1e, 0xff],
        ParticleType::Source => [0xf7, 0xdf, 0x00, 0xff],
        ParticleType::LSource => [0xff, 0xdf, 0x00, 0xff],
        ParticleType::Dirty => [0xFF, 0x00, 0xFF, 0xff],
        ParticleType::RegionBoundary => [0xFF, 0xFF, 0x00, 0xFF],
        _ => [0x00, 0x00, 0x00, 0xff],
    }
}