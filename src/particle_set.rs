use bevy::prelude::*;
use bevy::reflect::TypeUuid;

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c46"]
pub struct ParticleSet {
    set_name: String,
    format_version: u32,
    types: Vec<ReadParticleType>,
}

#[derive(serde::Deserialize)]
struct ReadParticleType {
    pub name: String,
    pub moves: Vec<[i32; 2]>,
    pub replace_water: bool,
    pub color: [u8; 4],
}

#[derive(Resource)]
pub struct LoadedParticleSets {
    pub set_handles: Vec<Handle<ParticleSet>>
}

pub fn read_particle_sets(
    asset_server: Res<AssetServer>,
    mut loaded_sets: ResMut<LoadedParticleSets>
) {
    let base_set_handle: Handle<ParticleSet> = asset_server.load("particle_types/base.partset");
    let world_set_handle: Handle<ParticleSet> = asset_server.load("particle_types/world.partset");

    loaded_sets.set_handles.push(base_set_handle);
    loaded_sets.set_handles.push(world_set_handle);
}