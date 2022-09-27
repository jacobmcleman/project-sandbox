#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{prelude::*, render::{render_resource::{Extent3d, TextureFormat}, camera::{RenderTarget}}, window::PresentMode };
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
        .insert_resource(BrushOptions {
            material: sandworld::ParticleType::Sand,
            radius: 10,
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(sandworld::World::new())
        .add_startup_system(setup)
        .add_system(create_spawned_chunks)
        .add_system(sand_update)
        .add_system(update_chunk_textures)
        .add_system(world_interact)
        .add_system(camera_movement)
        .add_system(button_system)
        .run();
}

#[derive(Component)]
struct Chunk {
    chunk_pos: gridmath::GridVec,
    chunk_texture_handle: Handle<Image>,
}

#[derive(Component)]
struct ToolSelector {
    material: sandworld::ParticleType,
    radius: i32,
}

struct BrushOptions {
    material: sandworld::ParticleType,
    radius: i32,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: UiRect {
                    left: Val::Px(32.0),
                    bottom: Val::Px(32.0),
                    ..default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(ToolSelector {
            material: sandworld::ParticleType::Sand,
            radius: 10,
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Sand",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: UiRect {
                    left: Val::Px(32.0),
                    bottom: Val::Px(32.0),
                    ..default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(ToolSelector {
            material: sandworld::ParticleType::Water,
            radius: 10,
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Water",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: UiRect {
                    left: Val::Px(32.0),
                    bottom: Val::Px(32.0),
                    ..default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(ToolSelector {
            material: sandworld::ParticleType::Stone,
            radius: 10,
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Stone",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                margin: UiRect {
                    left: Val::Px(32.0),
                    bottom: Val::Px(32.0),
                    ..default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(ToolSelector {
            material: sandworld::ParticleType::Source,
            radius: 1,
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
                "Source",
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ToolSelector),
        With<Button>,
    >,
    mut brush_options: ResMut<BrushOptions>,
) {
    for (interaction, mut color, selector) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                brush_options.material = selector.material;
                brush_options.radius = selector.radius;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }

        if selector.material == brush_options.material {
            *color = PRESSED_BUTTON.into();
        }
    }
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
        if let Some(chunk) = world.get_chunk(&chunkpos) {
            //println!("New chunk at {} - created an entity to render it", chunkpos);
            let image = render_chunk_texture(chunk.as_ref());
            let image_handle = images.add(image);

            let chunk_size = sandworld::CHUNK_SIZE as f32;

            commands.spawn_bundle(SpriteBundle {
                transform: Transform::from_translation(Vec3::new((chunkpos.x as f32 + 0.5) * chunk_size, (chunkpos.y as f32 + 0.5) * chunk_size, 0.))
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

fn camera_movement(
    mut query: Query<(&Camera, &mut OrthographicProjection, &mut Transform)>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    let (_camera, mut ortho, mut camera_transform) = query.single_mut();
    
    let mut log_scale = ortho.scale.ln();
    let move_speed = 64.;
    let zoom_speed = 1.;

    if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
        camera_transform.translation = (camera_transform.right() * move_speed * time.delta_seconds()) + camera_transform.translation;
    }
    if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
        camera_transform.translation = (camera_transform.left() * move_speed * time.delta_seconds()) + camera_transform.translation;
    }
    if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
        camera_transform.translation = (camera_transform.up() * move_speed * time.delta_seconds()) + camera_transform.translation;
    }
    if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
        camera_transform.translation = (camera_transform.down() * move_speed * time.delta_seconds()) + camera_transform.translation;
    }

    if keys.pressed(KeyCode::PageUp) {
        log_scale -= zoom_speed * time.delta_seconds();
    }
    if keys.pressed(KeyCode::PageDown) {
        log_scale += zoom_speed * time.delta_seconds();
    }

    ortho.scale = log_scale.exp();
}

fn world_interact(
    wnds: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform)>,
    mut sand: ResMut<sandworld::World>,
    buttons: Res<Input<MouseButton>>,
    brush_options: Res<BrushOptions>
) {
    if buttons.any_pressed([MouseButton::Left, MouseButton::Right]) {
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
                if buttons.pressed(MouseButton::Left){
                    sand.place_circle(gridpos, brush_options.radius, sandworld::Particle::new(brush_options.material), false);
                }
                else if buttons.pressed(MouseButton::Right) {
                    sand.place_circle(gridpos, 10, sandworld::Particle::new(sandworld::ParticleType::Air), true);
                }
            }
        }
    }
}