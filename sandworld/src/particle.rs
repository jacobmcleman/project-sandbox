use gridmath::GridVec;
use rand::Rng;

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
    LaserBeam,
    LaserEmitter,
    Boundary,
    RegionBoundary,
    Dirty,
}

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub particle_type: ParticleType,
    pub(crate) updated_this_frame: bool,
}

pub struct StateChange {
    melt: Option<(i32, ParticleType, f64)>,
    freeze: Option<(i32, ParticleType, f64)>,
}

pub(crate) struct CustomUpdateRules;

pub(crate) enum ChunkCommand {
    Add((GridVec, ParticleType)),
    Move(Vec<GridVec>),
    MoveOrDestroy(Vec<GridVec>),
    Remove,
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
                vec![GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0}, GridVec{x: 2, y: 0}, GridVec{x: -2, y: 0}, GridVec{x: 2, y: 1}, GridVec{x: -2, y: 1}],
                vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}],
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
            ParticleType::Sand => [ParticleType::Water, ParticleType::Lava].contains(&replace_type),
            ParticleType::Gravel => [ParticleType::Water, ParticleType::Steam, ParticleType::Lava].contains(&replace_type),
            ParticleType::Steam => [ParticleType::Water, ParticleType::Lava].contains(&replace_type),
            ParticleType::Lava => [ParticleType::Water, ParticleType::Steam].contains(&replace_type),
            ParticleType::LaserBeam => [ParticleType::Water, ParticleType::Steam].contains(&replace_type),
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
        ParticleType::LaserBeam => [0xff, 0x11, 0x11, 0xff],
        ParticleType::LaserEmitter => [0xff, 0xee, 0xee, 0xff],
        ParticleType::Dirty => [0xFF, 0x00, 0xFF, 0xff],
        ParticleType::RegionBoundary => [0xFF, 0xFF, 0x00, 0xFF],
        _ => [0x00, 0x00, 0x00, 0xff],
    }
}

pub fn get_heat_for_type(particle_type: ParticleType) -> i32 {
    match particle_type {
        ParticleType::Ice => -8,
        ParticleType::Water => -3,
        ParticleType::Stone => 2,
        ParticleType::Sand => 1,
        ParticleType::Gravel => 1,
        ParticleType::Lava => 64,
        ParticleType::Steam => 6,
        ParticleType::LaserBeam => 512,
        ParticleType::LaserEmitter => 1024,
        _ => 0,
    }
}

pub fn get_state_change_for_type(particle_type: ParticleType) -> StateChange {
    match particle_type {
        ParticleType::Ice => StateChange{    melt: Some((-28, ParticleType::Water, 0.5)), freeze: None },
        ParticleType::Water => StateChange{  melt: Some((64, ParticleType::Steam, 0.15)), freeze: Some((-40, ParticleType::Ice, 0.15)) },
        ParticleType::Steam => StateChange{  melt: None,                                  freeze: Some((50, ParticleType::Water, 0.05))},
        ParticleType::Stone => StateChange{  melt: Some((300, ParticleType::Lava, 0.05)),  freeze: None },
        ParticleType::Gravel => StateChange{ melt: Some((250, ParticleType::Lava, 0.25)),  freeze: None },
        ParticleType::Sand => StateChange{   melt: Some((180, ParticleType::Lava, 0.5)), freeze: None },
        ParticleType::Lava => StateChange{   melt: None,                                  freeze: Some((255, ParticleType::Stone, 0.1)) },
        _ => StateChange {                   melt: None,                                  freeze: None },
    }
}

pub fn try_state_change(particle_type: ParticleType, local_temperature: i32, rng: &mut rand::rngs::ThreadRng) -> Option<ParticleType> {
    let state_change = get_state_change_for_type(particle_type);
    
    if let Some((melt_temp, melt_type, melt_chance)) = state_change.melt {
        if local_temperature >= melt_temp && rng.gen_bool((melt_chance * (1. + ((local_temperature - melt_temp) as f64 / melt_temp.abs() as f64))).clamp(0., 1.)) {
            return Some(melt_type);
        }
    }
    if let Some((freeze_temp, freeze_type, freeze_chance)) = state_change.freeze {
        if local_temperature <= freeze_temp && rng.gen_bool((freeze_chance * (1. + ((freeze_temp - local_temperature) as f64 / freeze_temp.abs() as f64))).clamp(0., 1.)) {
            return Some(freeze_type);
        }
    }
    return None;
}

pub(crate) fn update_for_type(particle_type: ParticleType, x: u8, y: u8) -> Option<Vec<ChunkCommand>> {
    match particle_type {
        ParticleType::Source => Some(CustomUpdateRules::water_source_update(x, y)),
        ParticleType::LSource => Some(CustomUpdateRules::lava_source_update(x, y)),
        ParticleType::LaserBeam => Some(CustomUpdateRules::laser_beam_update(x, y)),
        ParticleType::LaserEmitter => Some(CustomUpdateRules::laser_emitter_update(x, y)),
        _ => None
    }
}


impl CustomUpdateRules {
    fn water_source_update(x: u8, y: u8) -> Vec<ChunkCommand> {
        vec![
            ChunkCommand::Add((GridVec{x: x as i32 - 1, y: y as i32}, ParticleType::Water)),
            ChunkCommand::Add((GridVec{x: x as i32 + 1, y: y as i32}, ParticleType::Water)),
            ChunkCommand::Add((GridVec{x: x as i32, y: y as i32 - 1}, ParticleType::Water)),
            ChunkCommand::Add((GridVec{x: x as i32, y: y as i32 + 1}, ParticleType::Water)),
        ]
    }
    
    fn lava_source_update(x: u8, y: u8) -> Vec<ChunkCommand> {
        vec![
            ChunkCommand::Add((GridVec{x: x as i32 - 1, y: y as i32}, ParticleType::Lava)),
            ChunkCommand::Add((GridVec{x: x as i32 + 1, y: y as i32}, ParticleType::Lava)),
            ChunkCommand::Add((GridVec{x: x as i32, y: y as i32 - 1}, ParticleType::Lava)),
            ChunkCommand::Add((GridVec{x: x as i32, y: y as i32 + 1}, ParticleType::Lava)),
        ]
    }
    
    fn laser_beam_update(_x: u8, _y: u8) -> Vec<ChunkCommand> {
        vec![
            ChunkCommand::MoveOrDestroy(vec![GridVec{x: 1, y: 0}]),
        ]
    }
    
    fn laser_emitter_update(x: u8, y: u8) -> Vec<ChunkCommand> {
        vec![
            ChunkCommand::Add((GridVec{x: x as i32 + 1, y: y as i32}, ParticleType::LaserBeam)),
        ]
    }
} 