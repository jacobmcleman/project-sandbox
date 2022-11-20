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

pub fn read_particle_sets(
    asset_server: Res<AssetServer>,
    particle_sets: Res<Assets<ParticleSet>>,
) {
    let base_set_handle: Handle<ParticleSet> = asset_server.load("particle_types/base.partset");
    //let world_set_handle: Handle<ParticleSet> = asset_server.load("particle_types/world.partset");

    if let Some(base_set) = particle_sets.get(&base_set_handle) {
        println!("Loaded set {} from base set with {} members", base_set.set_name, base_set.types.len());
    }
    //if let Some(world_set) = particle_sets.get(&world_set_handle) {
    //    println!("Loaded set {} from base set", world_set.set_name);
    //}
}