use std::{collections::VecDeque, sync::Arc};

use bevy::{
    prelude::*, window::PrimaryWindow
};
use bevy_xpbd_2d::prelude::*;
use gridmath::*;
use rand::Rng;
use sandworld::*;

use crate::{camera::cam_bounds, chunk_colliders::{self, ColliderLayer, SandworldColliderPlugin}, chunk_display::{DrawOptions, SandworldDisplayPlugin}};


pub struct SandSimulationPlugin;

#[derive(Component)]
struct BombComp {
    start_time: f32,
    timer_length: f32,
    blast_radius: i32,
    throw_power: f32,
}

#[derive(Component)]
struct SandParticle {
    particle: ParticleType,
}

#[derive(Bundle)]
struct SandParticleBundle {
    sprite_bundle: SpriteBundle,
    sand_particle: SandParticle,
    collider: Collider,
    rigidbody: RigidBody,
    velocity: LinearVelocity,
    layers: CollisionLayers,
    friction: Friction,
}

impl SandParticleBundle {
    fn new(particle_type: ParticleType, position: Vec3, velocity: Vec3) -> Self {
        let color = get_color_for_type(particle_type);
        SandParticleBundle {
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb_u8(color[0], color[1], color[2]),
                    ..Default::default()
                },
                transform: Transform::from_translation(position),
                ..Default::default()
            },
            sand_particle: SandParticle { particle: particle_type },
            collider: Collider::circle(0.5),
            rigidbody: RigidBody::Dynamic,
            velocity: LinearVelocity(velocity.truncate()),
            layers: CollisionLayers::new(ColliderLayer::Particle, [ColliderLayer::Terrain]),
            friction: Friction::new(0.5),
        }
    }
}

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
        .add_plugins(SandworldDisplayPlugin)
        .add_plugins(SandworldColliderPlugin)
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
        .add_systems(Update, sand_update.in_set(crate::UpdateStages::WorldUpdate))
        .add_systems(Update, (world_interact, bomb_timer, sand_particle_settle).in_set(crate::UpdateStages::Input))
        .add_systems(Update, draw_mode_controls.in_set(crate::UpdateStages::Input))
        ;
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum BrushMode {
    Place(sandworld::ParticleType, u8),
    Melt,
    Break,
    Chill,
    Beam,
    Ball,
}

#[derive(Resource)]
pub struct BrushOptions {
    pub brush_mode: BrushMode,
    pub radius: i32,
    pub click_start: Option<GridVec>,
}

#[derive(Resource)]
pub struct Sandworld {
    pub world: sandworld::World,
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

fn draw_mode_controls(
    mut draw_options: ResMut<DrawOptions>, 
    keys: Res<ButtonInput<KeyCode>>,
){
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

fn sand_update(
    mut world: ResMut<Sandworld>,
    mut world_stats: ResMut<WorldStats>,
    perf_settings: Res<crate::perf::PerfSettings>,
    cam_query: Query<(&Camera, &GlobalTransform)>,
    debug_buttons: Res<ButtonInput<KeyCode>>,
) {
    world.world.reset_updated_chunks();

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

    let (camera, cam_transform) = cam_query.single();
    let bounds = cam_bounds(camera, cam_transform);

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

fn bomb_timer(
    mut sand: ResMut<Sandworld>,
    bomb_query: Query<(&BombComp, &Transform, Entity)>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (bomb, transform, entity) in bomb_query.iter() {
        let timer = time.elapsed_seconds() - bomb.start_time;
        if timer > bomb.timer_length {
            let pos = transform.translation;
            let gridpos = GridVec::new(pos.x as i32, pos.y as i32);
            sand.world.break_circle(gridpos, bomb.blast_radius, 1.2);
            sand.world.temp_change_circle(gridpos, 8, 0.75, 1000);
            commands.entity(entity).despawn();

            let throwable_parts = particle_set![ParticleType::Gravel, ParticleType::Sand];

            let throw_radius = bomb.blast_radius / 2;
            let to_throw = sand.world.extract_circle(gridpos, throw_radius, throwable_parts);

            for (part_type, position) in to_throw {
                let world_pos = Vec3::new(position.x as f32, position.y as f32, 0.1);
                let power = (world_pos - pos).length_squared() / throw_radius.pow(2) as f32;
                let throw_velocity = (world_pos - pos).normalize_or_zero() * power * bomb.throw_power;
                commands.spawn(SandParticleBundle::new(part_type, world_pos, throw_velocity));
            }
        }
    }
}

fn sand_particle_settle(
    mut sand: ResMut<Sandworld>,
    particle_query: Query<(Entity, &Transform, &LinearVelocity, &SandParticle)>,
    mut commands: Commands,
) {
    let min_vel = 0.1;

    for (entity, transform, velocity, particle) in particle_query.iter() {
        let fpos = transform.translation;
        let gridpos = GridVec::new((fpos.x + 0.5) as i32, (fpos.y + 0.5) as i32);

        if particle_set![ParticleType::Stone].test(sand.world.get_particle(gridpos).particle_type) {
            commands.entity(entity).despawn_recursive();
        }
        else if velocity.length_squared() < min_vel {
            sand.world.replace_particle_filtered(gridpos, Particle::new(particle.particle), particle_set![ParticleType::Air, ParticleType::Water]);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn world_interact(
    wnds: Query<&Window, With<PrimaryWindow>>,
    capture_state: Res<crate::ui::PointerCaptureState>,
    q_cam: Query<(&Camera, &GlobalTransform)>,
    mut sand: ResMut<Sandworld>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut brush_options: ResMut<BrushOptions>,
    mut world_stats: ResMut<WorldStats>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
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

                match brush_options.brush_mode {
                    BrushMode::Ball => {
                        commands.spawn(SpriteBundle {
                                texture: asset_server.load("sprites/bomb1.png"),
                                transform: Transform::from_xyz(world_pos.x, world_pos.y, 0.1),
                                ..default()
                            })
                            .insert(BombComp {
                                start_time: time.elapsed_seconds(),
                                timer_length: 5.0,
                                blast_radius: 64,
                                throw_power: 1024.
                            })
                            .insert(Collider::circle(5.))
                            .insert(RigidBody::Dynamic)
                            .insert(CollisionLayers::new(
                                chunk_colliders::ColliderLayer::Projectile,
                                chunk_colliders::DEFAULT_COLLISION_LAYERS
                            ))
                            ;
                    },
                    _ => ()
                }
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
                            let clickposf = Vec2::new(click_pos.x as f32, click_pos.y as f32);
                            let direction = (world_pos - clickposf).normalize();

                            if let Some(hit) = spatial_query.cast_ray(
                                clickposf, 
                                Direction2d::new(direction).unwrap(), 
                                512., 
                                true, 
                                SpatialQueryFilter::default()
                            ) {
                                let hit_pos = clickposf + hit.time_of_impact * direction;
                                let grid_pos = GridVec::new(hit_pos.x as i32, hit_pos.y as i32);

                                sand.world.temp_change_circle(grid_pos, brush_options.radius, 0.01, 1800);
                            }
                        }     
                    },
                    BrushMode::Ball => () // only act on down
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
