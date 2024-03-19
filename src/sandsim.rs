use bevy::{
    input::keyboard::KeyboardInput, prelude::*, render::{render_asset::RenderAssetUsages, render_resource::{Extent3d, TextureFormat}}, window::PrimaryWindow
};
use bevy_rapier2d::{geometry::Collider, math::Vect, na::base};
use gridmath::{gridline::GridLine, GridBounds, GridVec};
use rand::Rng;
use sandworld::{ParticleType, ParticleSet, particle_set, CHUNK_SIZE};
use std::{collections::VecDeque, sync::{Arc, atomic::{AtomicU64, Ordering}}};

use crate::camera::cam_bounds;

pub struct SandSimulationPlugin;

impl Plugin for SandSimulationPlugin {
    fn build(&self, app: &mut App) {
        let mut rng = rand::thread_rng();
        let seed: u32 = rng.gen();

        println!("Seed: {}", seed);

        app.insert_resource(Sandworld {
            world: sandworld::World::new(Arc::new(crate::worldgen::WorldBuilder::new(
                seed, 5000., 1500., 500., 500., 400.,
            ))),
        })
        .insert_resource(DrawOptions {
            update_bounds: false,
            chunk_bounds: false,
            world_stats: false,
            force_redraw_all: false,
        })
        .insert_resource(BrushOptions {
            brush_mode: BrushMode::Place(ParticleType::Sand, 0),
            radius: 10,
            click_start: None,
        })
        .insert_resource(WorldStats {
            update_stats: None,
            sand_update_time: VecDeque::new(),
            chunk_texture_update_time: VecDeque::new(),
            chunk_cull_time: VecDeque::new(),
            target_chunk_updates: 0,
            mouse_grid_pos: GridVec::new(0, 0),
            mouse_chunk_pos: GridVec::new(0, 0),
            mouse_region: GridVec::new(0, 0),
        })
        .add_systems(Update, (create_spawned_chunks, clear_removed_chunks).in_set(crate::UpdateStages::WorldUpdate))
        .add_systems(Update, sand_update.in_set(crate::UpdateStages::WorldUpdate))
        .add_systems(Update, update_chunk_textures.in_set(crate::UpdateStages::WorldDraw))
        .add_systems(Update, world_interact.in_set(crate::UpdateStages::Input))
        .add_systems(Update, cull_hidden_chunks.in_set(crate::UpdateStages::WorldUpdate))
        .add_systems(Update, draw_mode_controls.in_set(crate::UpdateStages::Input));
    }
}

#[derive(Component)]
struct Chunk {
    chunk_pos: gridmath::GridVec,
    chunk_texture_handle: Handle<Image>,
    texture_dirty: bool,
}

#[derive(Resource)]
pub struct DrawOptions {
    pub update_bounds: bool,
    pub chunk_bounds: bool,
    pub world_stats: bool,
    pub force_redraw_all: bool,
}

#[derive(PartialEq, Eq, Clone)]
pub enum BrushMode {
    Place(sandworld::ParticleType, u8),
    Melt,
    Break,
    Chill,
    Beam,
}

#[derive(Resource)]
pub struct BrushOptions {
    pub brush_mode: BrushMode,
    pub radius: i32,
    pub click_start: Option<GridVec>,
}

#[derive(Resource)]
struct Sandworld {
    world: sandworld::World,
}

#[derive(Resource)]
pub struct WorldStats {
    pub update_stats: Option<sandworld::WorldUpdateStats>,
    pub sand_update_time: VecDeque<(f64, u64)>, // Pairs of update time and updated chunk counts
    pub chunk_texture_update_time: VecDeque<(f64, u64)>, // Pairs of update time and updated chunk counts
    pub chunk_cull_time: VecDeque<(f64, u64)>, // Pairs of culling time and culled chunk counts
    pub target_chunk_updates: u64,
    pub mouse_grid_pos: GridVec,
    pub mouse_chunk_pos: GridVec,
    pub mouse_region: GridVec,
}

fn draw_mode_controls(mut draw_options: ResMut<DrawOptions>, keys: Res<ButtonInput<KeyCode>>) {
    draw_options.force_redraw_all = false;

    if keys.just_pressed(KeyCode::F2) {
        draw_options.chunk_bounds = !draw_options.chunk_bounds;
        draw_options.force_redraw_all = true;
    }
    if keys.just_pressed(KeyCode::F3) {
        draw_options.update_bounds = !draw_options.update_bounds;
        draw_options.force_redraw_all = true;
    }
    if keys.just_pressed(KeyCode::F4) {
        draw_options.world_stats = !draw_options.world_stats;
    }
}

fn create_chunk_image(chunk: &sandworld::Chunk, draw_options: &DrawOptions) -> Image {
    let side_size = sandworld::CHUNK_SIZE as u32;
    Image::new(
        Extent3d {
            width: side_size,
            height: side_size,
            ..default()
        },
        bevy::render::render_resource::TextureDimension::D2,
        render_chunk_data(&chunk, &draw_options),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default()
    )
}

fn marching_squares_polylines_from_chunk(chunk: &sandworld::Chunk) -> crate::polyline::PolylineSet {
    let mut polyline = crate::polyline::PolylineSet::new();
    
    let vals = chunk.get_marching_square_vals(particle_set!(ParticleType::Stone, ParticleType::Sand, ParticleType::Gravel));

    let chunk_bounds = GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(CHUNK_SIZE as i32, CHUNK_SIZE as i32));

    for point in chunk_bounds.iter() {
        let x = point.x;
        let y = point.y;
        let index = y as usize * CHUNK_SIZE as usize + x as usize;

        let offset = Vect::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32) * -0.5;

        let bottom = offset + Vect::new(x as f32 + 0.5, y as f32);
        let top = offset + Vect::new(x as f32 + 0.5, y as f32 + 1.0);
        let left = offset + Vect::new(x as f32, y as f32 + 0.5);
        let right = offset + Vect::new(x as f32 + 1.0, y as f32 + 0.5);

        match vals[index] {
            1 => { polyline.add(bottom, left); },
            2 => { polyline.add(right, bottom); },
            3 => { polyline.add(right, left); },
            4 => { polyline.add(top, right); },
            5 => { polyline.add(top, left); polyline.add(bottom,right);},
            6 => { polyline.add(top, bottom) ;},
            7 => { polyline.add(top, left); },
            8 => { polyline.add(left, top) ;},
            9 => { polyline.add(bottom, top); },
            10 => { polyline.add(left, bottom); polyline.add(right, top);},
            11 => { polyline.add(right, top); },
            12 => { polyline.add(left, right); },
            13 => { polyline.add(bottom, right); },
            14 => { polyline.add(left, bottom); },
            _ => ()
        }
    }

    polyline
}

fn collider_for_chunk(chunk: &sandworld::Chunk) -> Collider {
    let mut polyline = marching_squares_polylines_from_chunk(chunk);
    polyline.simplify(4.);
    let (vertices, indices) = polyline.to_verts_and_inds();
    Collider::polyline(vertices, Some(indices))
}

fn render_chunk_data(chunk: &sandworld::Chunk, draw_options: &DrawOptions) -> Vec<u8> {
    chunk.render_to_color_array(draw_options.update_bounds, draw_options.chunk_bounds)
}

fn create_spawned_chunks(
    mut world: ResMut<Sandworld>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    draw_options: Res<DrawOptions>,
) {
    let added_chunks = world.world.get_added_chunks();
    for chunkpos in added_chunks {
        if let Some(chunk) = world.world.get_chunk(&chunkpos) {
            let image = create_chunk_image(chunk.as_ref(), &draw_options);
            let image_handle = images.add(image);

            let chunk_size = sandworld::CHUNK_SIZE as f32;

            commands
                .spawn(SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        (chunkpos.x as f32 + 0.5) * chunk_size,
                        (chunkpos.y as f32 + 0.5) * chunk_size,
                        0.,
                    ))
                    .with_scale(Vec3::new(1., 1., 1.)),
                    texture: image_handle.clone(),
                    ..default()
                })
                .insert(Chunk {
                    chunk_pos: chunkpos,
                    chunk_texture_handle: image_handle.clone(),
                    texture_dirty: false,
                })
                .insert(collider_for_chunk(chunk.as_ref()));
        }
    }
}

fn clear_removed_chunks(
    mut world: ResMut<Sandworld>,
    chunk_query: Query<(&mut Chunk, Entity)>,
    mut commands: Commands,
) {
    let removed_chunks = world.world.get_removed_chunks();
    
    for (chunk, entity) in chunk_query.iter() {
        if removed_chunks.contains(&chunk.chunk_pos) {
            commands.entity(entity).despawn();
        }
    }
}

fn cull_hidden_chunks(
    mut chunk_query: Query<(&Chunk, &mut Visibility)>,
    mut world_stats: ResMut<WorldStats>,
    cam_query: Query<(&OrthographicProjection, &Camera, &GlobalTransform)>,
) {
    let culled_chunks = AtomicU64::new(0);
    let culling_start = std::time::Instant::now();

    let (ortho, camera, cam_transform) = cam_query.single();
    let bounds = cam_bounds(ortho, camera, cam_transform); 

    chunk_query.par_iter_mut().for_each(|(chunk, mut vis)| {
        
        let chunk_bounds = GridBounds::new_from_corner(
            chunk.chunk_pos * (CHUNK_SIZE as i32),
            GridVec {
                x: CHUNK_SIZE as i32,
                y: CHUNK_SIZE as i32,
            },
        );

        if !chunk_bounds.overlaps(bounds) {
            culled_chunks.fetch_add(1, Ordering::Relaxed);
            *vis = Visibility::Hidden;
        }
        else {
            *vis = Visibility::Inherited;
        }
    });

    let culling_end = std::time::Instant::now();
    let culling_time = culling_end - culling_start;

    world_stats
        .chunk_cull_time
        .push_back((culling_time.as_secs_f64(), culled_chunks.load(Ordering::Relaxed)));
    if world_stats.chunk_cull_time.len() > 64 {
        world_stats.chunk_cull_time.pop_front();
    }
}

fn update_chunk_textures(
    mut world: ResMut<Sandworld>,
    mut images: ResMut<Assets<Image>>,
    mut world_stats: ResMut<WorldStats>,
    mut chunk_query: Query<(&mut Chunk, &Visibility)>,
    draw_options: Res<DrawOptions>,
) {
    let updated_chunks = world.world.get_updated_chunks();
    let mut updated_textures_count = 0;
    let update_start = std::time::Instant::now();

    if draw_options.force_redraw_all || !updated_chunks.is_empty() {
        chunk_query.par_iter_mut().for_each(|(mut chunk_comp, _visibility)| {
            if draw_options.force_redraw_all || updated_chunks.contains(&chunk_comp.chunk_pos) {
                chunk_comp.texture_dirty = true;
            }
        });
    }

    for (mut chunk_comp, visibility) in chunk_query.iter_mut() {
        if chunk_comp.texture_dirty && visibility == Visibility::Inherited {
            if let Some(chunk) = world.world.get_chunk(&chunk_comp.chunk_pos) {
                images.get_mut(chunk_comp.chunk_texture_handle.clone()).unwrap().data = render_chunk_data(chunk.as_ref(), &draw_options);
                updated_textures_count += 1;
                chunk_comp.texture_dirty = false;
            }
        }
    }

    let update_end = std::time::Instant::now();
    let update_time = update_end - update_start;

    world_stats
        .chunk_texture_update_time
        .push_back((update_time.as_secs_f64(), updated_textures_count));
    if world_stats.chunk_texture_update_time.len() > 64 {
        world_stats.chunk_texture_update_time.pop_front();
    }
}

fn sand_update(
    mut world: ResMut<Sandworld>,
    mut world_stats: ResMut<WorldStats>,
    perf_settings: Res<crate::perf::PerfSettings>,
    cam_query: Query<(&OrthographicProjection, &Camera, &GlobalTransform)>,
    debug_buttons: Res<ButtonInput<KeyCode>>,
) {
    let mut target_chunk_updates = 128;

    if let Some(_stats) = &world_stats.update_stats {
        // Aim to use half of the available frame time for sand updates
        let target_update_seconds: f64 = (1. / (perf_settings.target_frame_rate as f64)) * 0.5;
        let mut chunk_updates_per_second_avg = 0.;
        for (time, count) in &world_stats.sand_update_time {
            chunk_updates_per_second_avg += (*count + 1) as f64 / time;
        }
        chunk_updates_per_second_avg =
            chunk_updates_per_second_avg / (world_stats.sand_update_time.len() as f64);
        target_chunk_updates = (target_update_seconds * chunk_updates_per_second_avg) as u64;
        world_stats.target_chunk_updates = target_chunk_updates;
    }

    let (ortho, camera, cam_transform) = cam_query.single();
    let bounds = cam_bounds(ortho, camera, cam_transform);

    let update_options = sandworld::WorldUpdateOptions {
        force_compress_decompress_all: debug_buttons.just_pressed(KeyCode::F10),
    };

    let update_start = std::time::Instant::now();
    let stats = world.world.update(bounds, target_chunk_updates, update_options);
    let update_end = std::time::Instant::now();
    let update_time = update_end - update_start;
    world_stats
        .sand_update_time
        .push_back((update_time.as_secs_f64(), stats.chunk_updates));
    if world_stats.sand_update_time.len() > 64 {
        world_stats.sand_update_time.pop_front();
    }
    world_stats.update_stats = Some(stats);
}

fn world_interact(
    wnds: Query<&Window, With<PrimaryWindow>>,
    capture_state: Res<crate::ui::PointerCaptureState>,
    q_cam: Query<(&Camera, &GlobalTransform)>,
    mut sand: ResMut<Sandworld>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut brush_options: ResMut<BrushOptions>,
    mut world_stats: ResMut<WorldStats>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_cam.single();

    // get the window that the camera is displaying to (or the primary window)
    let Ok(wnd) = wnds.get_single() else {
        eprintln!("no window!!!");
        return;
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        let world_pos = camera.viewport_to_world_2d(camera_transform, screen_pos).unwrap();
        let gridpos = GridVec::new(world_pos.x as i32, world_pos.y as i32);

        world_stats.mouse_grid_pos = gridpos;
        world_stats.mouse_chunk_pos = gridpos / CHUNK_SIZE as i32;
        world_stats.mouse_region = sandworld::World::get_regionpos_for_chunkpos(&(world_stats.mouse_chunk_pos));

        if !capture_state.click_consumed && buttons.any_pressed([MouseButton::Left, MouseButton::Right])
        {
            if buttons.just_pressed(MouseButton::Left) {
                brush_options.click_start = Some(gridpos);

                println!("click start at {}", gridpos);
            }

            if buttons.pressed(MouseButton::Left) {
                match brush_options.brush_mode {
                    BrushMode::Place(part_type, data) => sand.world.place_circle(
                        gridpos,
                        brush_options.radius,
                        sandworld::Particle::new_with_data(part_type, data),
                        false,
                    ),
                    BrushMode::Melt => {
                        sand.world
                            .temp_change_circle(gridpos, brush_options.radius, 0.01, 1800)
                    }
                    BrushMode::Break => sand.world.break_circle(gridpos, brush_options.radius, 0.1),
                    BrushMode::Chill => {
                        sand.world
                            .temp_change_circle(gridpos, brush_options.radius, 0.01, -100)
                    },
                    BrushMode::Beam => {
                        if let Some(click_pos) = brush_options.click_start {
                            let hitmask = particle_set![ParticleType::Stone, ParticleType::Sand, ParticleType::Gravel];
                            if let Some(hit) = sand.world.cast_ray(&hitmask, GridLine::new(click_pos, gridpos)) {
                                sand.world.temp_change_circle(hit.point, brush_options.radius, 0.01, 1800);
                            }
                        }     
                    }
                }
            } else if buttons.pressed(MouseButton::Right) {
                sand.world.place_circle(
                    gridpos,
                    10,
                    sandworld::Particle::new(sandworld::ParticleType::Air),
                    true,
                );
            }
        }
    }
}
