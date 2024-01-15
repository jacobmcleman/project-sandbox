use gridmath::GridVec;
use rand::Rng;
use once_cell::sync::Lazy;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum ParticleType {
    Air,
    Sand,
    Water,
    Stone,
    Gravel,
    Steam,
    Lava,
    MoltenGlass,
    Glass,
    Ice,
    Source,
    LaserBeam,
    LaserEmitter,
    Boundary,
    RegionBoundary,
    Dirty,
}

#[derive(Debug, Copy, Clone)]
pub struct Particle {
    pub particle_type: ParticleType,
    /*
        Highest bit is reserved for particle update flag
        Other bits may be used for custom particle logic
    */
    data: u8, 
}

pub struct StateChange {
    melt: Option<(i32, ParticleType, f64)>,
    freeze: Option<(i32, ParticleType, f64)>,
}

pub(crate) struct CustomUpdateRules;

pub(crate) enum ChunkCommand {
    Add((GridVec, ParticleType, u8)),
    Move(Vec<GridVec>),
    MoveOrDestroy(Vec<GridVec>),
    Remove,
    Mutate(ParticleType, u8),
}

pub static SOLID_MATS: Lazy<Vec<ParticleType>> = Lazy::new(|| {vec![ParticleType::Stone, ParticleType::Glass] });


impl Particle {
    pub fn new(particle_type: ParticleType) -> Self {
        Particle{particle_type, data: 0}
    }
    
    pub(crate)  fn new_already_updated(particle_type: ParticleType) -> Self {
        Particle::new_with_data(particle_type, 1 << 7)
    }
    
    pub fn new_with_data(particle_type: ParticleType, particle_data: u8) -> Self {
        Particle{particle_type, data: particle_data}
    }
    
    pub(crate) fn updated_this_frame(&self) -> bool {
        return self.data & (1<<7) != 0
    }
    
    pub(crate) fn set_updated_this_frame(&mut self, val: bool) {
        if val {
            self.data |= 1<<7;
        }
        else {
            self.data &= !(1<<7);
        }
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
            ParticleType::MoltenGlass => vec![
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
            ParticleType::MoltenGlass => [ParticleType::Water, ParticleType::Steam, ParticleType::Lava].contains(&replace_type),
            ParticleType::LaserBeam => [ParticleType::Water, ParticleType::Steam].contains(&replace_type),
            _ => false
        }
    }
}

impl Default for Particle {
    fn default() -> Self { Particle::new(ParticleType::Air) }
}


pub fn get_color_for_type(particle_type: ParticleType) -> [u8; 4] {
    match particle_type {
        ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
        ParticleType::Water => [0x6d, 0x95, 0xc9, 0xff], // #6d95c9
        ParticleType::Gravel => [0xa9, 0xa3, 0xb5, 0xff], // #a9a3b5
        ParticleType::Stone => [0x6b, 0x6f, 0x75, 0xff], //#6b6f75
        ParticleType::Steam => [0xe6, 0xec, 0xf0, 0xff], //#e6ecf0
        ParticleType::Lava => [0xef, 0x70, 0x15, 0xff], //#ef7015
        ParticleType::MoltenGlass => [0xf0, 0x95, 0x16, 0xff], //#f09516
        ParticleType::Glass => [0x31, 0x60, 0x5e, 0xff], //#31605e
        ParticleType::Ice => [0xbf, 0xdb, 0xff, 0xff], //#bfdbff
        ParticleType::Air => [0x1e, 0x1e, 0x1e, 0xff],
        ParticleType::Source => [0xf7, 0xdf, 0x00, 0xff],
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
        ParticleType::Stone => 0,
        ParticleType::Sand => 0,
        ParticleType::Gravel => 0,
        ParticleType::Lava => 128,
        ParticleType::MoltenGlass => 128,
        ParticleType::Glass => 1,
        ParticleType::Steam => 16,
        ParticleType::LaserBeam => 1024,
        ParticleType::LaserEmitter => 2048,
        _ => 0,
    }
}

pub fn get_viscosity_for_type(particle_type: ParticleType, temp: i32) -> i32 {
    match particle_type {
        ParticleType::Water => 2,
        ParticleType::Lava =>  gridmath::int_util::remap_clamped(temp, 196, 320, 3, 1),
        ParticleType::MoltenGlass =>  gridmath::int_util::remap_clamped(temp, 196, 400, 4, 1),
        ParticleType::Steam => -1,
        _ => 0
    }
}

pub fn get_state_change_for_type(particle_type: ParticleType) -> StateChange {
    match particle_type {
        ParticleType::Ice => StateChange{           melt: Some((-28, ParticleType::Water, 0.5)),        freeze: None },
        ParticleType::Water => StateChange{         melt: Some((100, ParticleType::Steam, 0.15)),       freeze: Some((-40, ParticleType::Ice, 0.15)) },
        ParticleType::Steam => StateChange{         melt: None,                                         freeze: Some((150, ParticleType::Water, 0.25))},
        ParticleType::Stone => StateChange{         melt: Some((700, ParticleType::Lava, 0.15)),        freeze: None },
        ParticleType::Gravel => StateChange{        melt: Some((680, ParticleType::Lava, 0.2)),         freeze: None },
        ParticleType::Sand => StateChange{          melt: Some((650, ParticleType::MoltenGlass, 0.2)),  freeze: None },
        ParticleType::Lava => StateChange{          melt: None,                                         freeze: Some((516, ParticleType::Stone, 0.25)) },
        ParticleType::MoltenGlass => StateChange{   melt: None,                                         freeze: Some((500, ParticleType::Glass, 0.25)) },
        ParticleType::Glass => StateChange{         melt: Some((480, ParticleType::MoltenGlass, 0.1)),  freeze: None },
        _ => StateChange {                          melt: None,                                         freeze: None },
    }
}

pub fn get_is_lonely_type(particle_type: ParticleType) -> bool {
    match particle_type {
        ParticleType::Stone => true,
        ParticleType::Glass => true,
        _ => false
    }
}

pub fn get_lonely_break_type(particle_type: ParticleType) -> ParticleType {
    match particle_type {
        ParticleType::Stone => ParticleType::Gravel,
        ParticleType::Glass => ParticleType::Sand,
        _ => ParticleType::Sand
    }
}

pub fn try_state_change(particle_type: ParticleType, local_temperature: i32, rng: &mut rand::rngs::ThreadRng) -> Option<ParticleType> {
    let state_change = get_state_change_for_type(particle_type);
    
    if let Some((melt_temp, melt_type, melt_chance)) = state_change.melt {
        if local_temperature >= melt_temp && rng.gen_bool((melt_chance * (((local_temperature - melt_temp) as f64 / melt_temp.abs() as f64))).clamp(0., 1.)) {
            return Some(melt_type);
        }
    }
    if let Some((freeze_temp, freeze_type, freeze_chance)) = state_change.freeze {
        if local_temperature <= freeze_temp && rng.gen_bool((freeze_chance * (((freeze_temp - local_temperature) as f64 / freeze_temp.abs() as f64))).clamp(0., 1.)) {
            return Some(freeze_type);
        }
    }
    return None;
}

pub(crate) fn get_update_fn_for_type(particle_type: ParticleType) -> Option<fn(GridVec, Particle, &[ParticleType; 8])->Vec<ChunkCommand>> {
    match particle_type {
        ParticleType::Source => Some(CustomUpdateRules::water_source_update),
        ParticleType::LaserBeam => Some(CustomUpdateRules::laser_beam_update),
        ParticleType::LaserEmitter => Some(CustomUpdateRules::laser_emitter_update),
        _ => None
    }
}


impl CustomUpdateRules {
    fn water_source_update(position: GridVec, particle: Particle, neighbors: &[ParticleType; 8] ) -> Vec<ChunkCommand> {
        let data_val = particle.data & !(1<<7);
        if data_val == 0 {
            let mut new_val = 0;
            for part in neighbors {
                new_val = match  part {
                    ParticleType::Water => 1,
                    ParticleType::Lava => 2,
                    ParticleType::Sand => 3,
                    ParticleType::Gravel => 4,
                    ParticleType::Steam => 5,
                    _ => 0,
                };
                if new_val != 0 {
                    break;
                }
            }
            vec![ChunkCommand::Mutate(particle.particle_type, new_val)]
        }
        else {
            let emit_type = match data_val {
                1 => ParticleType::Water,
                2 => ParticleType::Lava,  
                3 => ParticleType::Sand,
                4 => ParticleType::Gravel,
                5 => ParticleType::Steam,
                _ => ParticleType::Air,
            };
            
            vec![
                ChunkCommand::Add((GridVec{x: position.x - 1, y: position.y}, emit_type, 0)),
                ChunkCommand::Add((GridVec{x: position.x + 1, y: position.y}, emit_type, 0)),
                ChunkCommand::Add((GridVec{x: position.x, y: position.y - 1}, emit_type, 0)),
                ChunkCommand::Add((GridVec{x: position.x, y: position.y + 1}, emit_type, 0)),
            ]
        }
    }
    
    fn laser_beam_update(_position: GridVec, particle: Particle, _neighbors: &[ParticleType; 8]) -> Vec<ChunkCommand> {
        let dir_val = particle.data & !(1<<7);
        let movement = match dir_val {
            1 => GridVec::new(1, 0),
            2 => GridVec::new(0, -1),
            3 => GridVec::new(-1, 0),
            _ => GridVec::new(0, 1),
        };
        
        vec! [
            ChunkCommand::MoveOrDestroy(vec![movement])
        ]
    }
    
    fn laser_emitter_update(position: GridVec, particle: Particle, _neighbors: &[ParticleType; 8]) -> Vec<ChunkCommand> {
        let dir_val = particle.data & !(1<<7);
        let movement = match dir_val {
            1 => GridVec::new(1, 0),
            2 => GridVec::new(0, -1),
            3 => GridVec::new(-1, 0),
            _ => GridVec::new(0, 1),
        };
        
        vec![
            ChunkCommand::Add((GridVec{x: position.x, y: position.y} + movement, ParticleType::LaserBeam, dir_val)),
        ]
    }
} 