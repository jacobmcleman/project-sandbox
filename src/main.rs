#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{prelude::*, render::{render_resource::{Extent3d, TextureFormat}, camera::RenderTarget}, window::PresentMode };
use gridmath::GridVec;

fn main(){
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Project Sandbox - Bevy".to_string(),
            width: 500.,
            height: 300.,
            present_mode: PresentMode::AutoVsync,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(sandworld::World::new())
        .add_startup_system(setup)
        .add_system(create_spawned_chunks)
        .add_system(sand_update)
        .add_system(update_chunk_textures)
        .add_system(world_interact)
        .run();
}

#[derive(Component)]
struct Chunk {
    chunk_pos: gridmath::GridVec,
    chunk_texture_handle: Handle<Image>,
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

fn create_chunk_texture() -> Image {
    let side_size = sandworld::CHUNK_SIZE as u32;
    Image::new_fill(
        Extent3d { width: side_size, height: side_size, ..default() },
        bevy::render::render_resource::TextureDimension::D2,
        &[0xff, 0x00, 0xFF, 0xff],
        TextureFormat::Rgba8Unorm
    )
}

fn get_color_for_type(particle_type: sandworld::ParticleType) -> [u8; 4] {
    match particle_type {
        sandworld::ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
        sandworld::ParticleType::Water => [0x56, 0x9c, 0xd6, 0xff],
        sandworld::ParticleType::Stone => [0xd4, 0xd4, 0xd4, 0xff],
        sandworld::ParticleType::Air => [0x1e, 0x1e, 0x1e, 0xff],
        sandworld::ParticleType::Source => [0xf7, 0xdf, 0x00, 0xff],
        sandworld::ParticleType::Dirty => [0xFF, 0x00, 0xFF, 0xff],
        _ => [0x00, 0x00, 0x00, 0xff],
    }
}

fn render_chunk_texture(chunk: &sandworld::Chunk) -> Image {
    let side_size = sandworld::CHUNK_SIZE as u32;
    let mut bytes = Vec::with_capacity(side_size as usize * side_size as usize * 4);

    for y in 0..sandworld::CHUNK_SIZE {
        for x in 0..sandworld::CHUNK_SIZE {
            let part = chunk.get_particle(x, sandworld::CHUNK_SIZE - y - 1).particle_type;
            let color = get_color_for_type(part);
            bytes.push(color[0]);
            bytes.push(color[1]);
            bytes.push(color[2]);
            bytes.push(color[3]);
        }
    }

    Image::new(
        Extent3d { width: side_size, height: side_size, ..default() },
        bevy::render::render_resource::TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8Unorm
    )
}

fn create_spawned_chunks(
    mut world: ResMut<sandworld::World>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let added_chunks = world.get_added_chunks();
    for chunkpos in added_chunks {
        if let Some(_chunk) = world.get_chunk(&chunkpos) {
            //println!("New chunk at {} - created an entity to render it", chunkpos);
            let image = create_chunk_texture();
            let image_handle = images.add(image);

            let chunk_size = sandworld::CHUNK_SIZE as f32;

            commands.spawn_bundle(SpriteBundle {
                transform: Transform::from_translation(Vec3::new(chunkpos.x as f32 * chunk_size, chunkpos.y as f32 * chunk_size, 0.))
                    .with_scale(Vec3::new(1., 1., 1.)),
                texture: image_handle.clone(),
                ..default()
            }).insert(Chunk {
                chunk_pos: chunkpos,
                chunk_texture_handle: image_handle.clone(),
            });
        }
    }
}

fn update_chunk_textures(
    mut world: ResMut<sandworld::World>,
    mut images: ResMut<Assets<Image>>,
    chunk_query: Query<&Chunk>
) {
    let updated_chunks = world.get_updated_chunks();

    if updated_chunks.is_empty() {
        return;
    }

    for chunk_comp in chunk_query.iter() {
        if updated_chunks.contains(&chunk_comp.chunk_pos) {
            if let Some(chunk) = world.get_chunk(&chunk_comp.chunk_pos) {
                images.set_untracked(chunk_comp.chunk_texture_handle.clone(), render_chunk_texture(chunk.as_ref()));
            }
        }
    }
}

fn sand_update(mut world: ResMut<sandworld::World>) {
    world.update();    
}

fn world_interact(
    wnds: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform)>,
    mut sand: ResMut<sandworld::World>,
    buttons: Res<Input<MouseButton>>,
) {
    if buttons.pressed(MouseButton::Left) {
        // get the camera info and transform
        // assuming there is exactly one main camera entity, so query::single() is OK
        let (camera, camera_transform) = q_cam.single();

        // get the window that the camera is displaying to (or the primary window)
        let wnd = if let RenderTarget::Window(id) = camera.target {
            wnds.get(id).unwrap()
        } else {
            wnds.get_primary().unwrap()
        };

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = wnd.cursor_position() {
            // get the size of the window
            let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            let world_pos: Vec2 = world_pos.truncate();

            //println!("World coords: {}/{}", world_pos.x as i32, world_pos.y as i32);

            let gridpos = GridVec::new(world_pos.x as i32, world_pos.y as i32);
            if sand.contains(gridpos) {
                sand.place_circle(gridpos, 10, sandworld::Particle::new(sandworld::ParticleType::Sand), false);
            }
        }
    }
}