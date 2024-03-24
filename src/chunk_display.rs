use std::sync::atomic::AtomicU32;

use bevy::{prelude::*, render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureFormat}, view::visibility}};
use gridmath::{GridBounds, GridVec};
use sandworld::CHUNK_SIZE;
use crate::{camera::cam_bounds, sandsim::*};

#[derive(Resource)]
pub struct DrawOptions {
    pub update_bounds: bool,
    pub chunk_bounds: bool,
    pub world_stats: bool,
    pub force_redraw_all: bool,
    pub show_colliders: bool,
}

#[derive(Component, Default)]
pub struct ChunkDisplay {
    pub chunk_pos: Option<GridVec>,
    redraw: bool,
    dirty: bool,
}

#[derive(Bundle, Default)]
struct ChunkDisplayBundle {
    chunk_display: ChunkDisplay,
    sprite: SpriteBundle,
}

pub struct SandworldDisplayPlugin;

impl Plugin for SandworldDisplayPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(DrawOptions {
            update_bounds: false,
            chunk_bounds: false,
            world_stats: false,
            force_redraw_all: false,
            show_colliders: false,
        })
        .add_systems(Update,(assign_chunk_displays, update_chunk_textures))
        ;
    }
}

fn update_chunk_textures(
    world: Res<Sandworld>,
    mut chunk_display_query: Query<(&mut ChunkDisplay, &mut Handle<Image>)>,
    draw_options: Res<DrawOptions>,
    mut images: ResMut<Assets<Image>>,
    mut world_stats: ResMut<WorldStats>,
) {
    let update_start = std::time::Instant::now();

    let updated_chunks = world.world.get_updated_chunks();
    let mut updated_textures_count = 0;

    chunk_display_query.iter_mut().for_each(|(mut chunk_display, texture)| {
        // Is this display entity currently representing a chunk
        if let Some(chunk_pos) = chunk_display.chunk_pos {
            // If the chunk this entity is representing needs to show an update
            if draw_options.force_redraw_all || chunk_display.redraw || updated_chunks.contains(&chunk_pos) {
                // Get the chunk from the world, may fail if the world doesn't have the chunk yet
                if let Some(world_chunk) = world.world.get_chunk(&chunk_pos) {
                    let cur_tuxture = images.get_mut(texture.clone()).unwrap();

                    // TODO: do shader things to allow this to directly memcpy the chunks material data 
                    // so color nonsense happens in shader land and only need to send 8 bits per particle
                    cur_tuxture.data = render_chunk_data(world_chunk, &draw_options);

                    updated_textures_count += 1;

                    chunk_display.redraw = false;
                }
            }
        }
    });

    let update_end = std::time::Instant::now();
    let update_time = update_end - update_start;

    world_stats
        .chunk_texture_update_time
        .push_back((update_time.as_secs_f64(), updated_textures_count));
    if world_stats.chunk_texture_update_time.len() > 64 {
        world_stats.chunk_texture_update_time.pop_front();
    }
}

fn assign_chunk_displays(
    mut chunk_display_query: Query<(&mut ChunkDisplay, &mut Transform, &mut Visibility)>,
    cam_query: Query<(&OrthographicProjection, &Camera, &GlobalTransform), Or<(Changed<OrthographicProjection>, Changed<Camera>, Changed<GlobalTransform>)>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    println!("Currently have {} chunk display entities", chunk_display_query.iter().count());
    
    let (ortho, camera, cam_transform) = cam_query.single();
    let bounds = cam_bounds(ortho, camera, cam_transform);

    // Establish bounds that should be represented by Bevy entities
    let chunk_bounds: GridBounds = GridBounds::new_from_extents(
        sandworld::World::get_chunkpos(&bounds.bottom_left()), 
        sandworld::World::get_chunkpos(&bounds.top_right())
    ).inflated_by(1); // One chunk of padding to reduce flickering
    
    // Don't unload chunks until they're a few off the edge to limit thrashing
    let keep_chunk_bounds = chunk_bounds.inflated_by(2);

    // println!("Cam bounds: {}", bounds);
    // println!("chunk bounds: {}", chunk_bounds);
    // println!("discard bound: {}", keep_chunk_bounds);

    // Free up any display entities that are beyond those bounds (inflated a little to avoid thrashing for small movements)
    chunk_display_query.par_iter_mut().for_each(|(mut chunk_display, _, mut visibility)| {
        if let Some(chunk_pos) = chunk_display.chunk_pos {
            if !keep_chunk_bounds.contains(chunk_pos) {
                // Unassign and hide
                chunk_display.chunk_pos = None;
                *visibility = Visibility::Hidden;

                // println!("freeing display_entity for {}", chunk_pos);
            }
        }
    });

    // Go through loaded chunks
    // Keep track of entities ready for reassignment as well as which visible chunks are ready
    let mut free_displays = Vec::new();
    let mut filled_spots = 0;
    let mut filled_set = vec![false; chunk_bounds.area()];

    chunk_display_query.iter_mut().for_each(|result| {
        if let Some(chunk_pos) = result.0.chunk_pos {
            if let Some(index) = chunk_bounds.get_index(chunk_pos) {
                filled_set[index] = true;
                filled_spots += 1;
            }
        }
        else {
            free_displays.push(result);
        }
    });
    
    // println!("{0}/{1} visible chunks have entities, {2} display entities ready to be allocated", filled_spots, chunk_bounds.area(), free_displays.len());

    // For each point in the visible bounds that is missing representation, first attempt to claim one of the unclaimed chunks
    // Failing that increase the count of representing entities that need to spawn
    for i in 0..filled_set.len() {
        if !filled_set[i] {
            let chunk_pos = chunk_bounds.at_index(i);
            let chunk_size = CHUNK_SIZE as f32;
            let position = Vec3::new(
                (chunk_pos.x as f32 + 0.5) * chunk_size, 
                (chunk_pos.y as f32 + 0.5) * chunk_size, 
                0.
            );

            if let Some((mut chunk_display, mut transform, mut visibility)) = free_displays.pop() {
                chunk_display.chunk_pos = Some(chunk_pos);
                chunk_display.redraw = true;
                *visibility = Visibility::Inherited;
                transform.translation = position;
            }
            else {
                let image = create_chunk_image();
                commands.spawn(ChunkDisplayBundle {
                    sprite: SpriteBundle { 
                        texture: images.add(image),
                        transform: Transform::from_translation(position),
                        ..Default::default() 
                    },
                    chunk_display: ChunkDisplay {
                        chunk_pos: Some(chunk_pos),
                        redraw: true,
                        dirty: true
                    }
                });
            }
        }
    }
}

fn render_chunk_data(chunk: &sandworld::Chunk, draw_options: &DrawOptions) -> Vec<u8> {
    chunk.render_to_color_array(draw_options.update_bounds, draw_options.chunk_bounds)
}

fn create_chunk_image() -> Image {
    let side_size = sandworld::CHUNK_SIZE as u32;
    Image::new(
        Extent3d {
            width: side_size,
            height: side_size,
            ..default()
        },
        bevy::render::render_resource::TextureDimension::D2,
        vec![0; CHUNK_SIZE as usize * CHUNK_SIZE as usize * 4],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default()
    )
}