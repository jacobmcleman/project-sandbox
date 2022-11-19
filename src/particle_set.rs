use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_common_assets::ron::RonAssetPlugin;


#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct ParticleSet {
    set_name: String,
    types: Vec<ReadParticleType>,
}

#[derive(serde::Deserialize)]
struct ReadParticleType {
    pub name: String,
    pub moves: Vec<GridVec>,
    pub replace_water: bool,
    pub color: [u8; 4],
}