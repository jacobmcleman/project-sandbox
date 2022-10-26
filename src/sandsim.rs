use bevy::{prelude::*, render::{render_resource::{Extent3d, TextureFormat}, camera::{RenderTarget}} };
use gridmath::GridVec;

pub struct SandSimulationPlugin;

impl Plugin for SandSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(sandworld::World::new())
        .insert_resource(DrawOptions {
            update_bounds: false,
            chunk_bounds: false,
            world_stats: false,
            force_redraw_all: false,
        })
        .insert_resource(BrushOptions {
            material: sandworld::ParticleType::Sand,
            radius: 10,
        })
        .insert_resource(WorldStats {
            update_stats: None,
        })
        .add_system(create_spawned_chunks.label(crate::UpdateStages::WorldUpdate))
        .add_system(sand_update.label(crate::UpdateStages::WorldUpdate))
        .add_system(update_chunk_textures.label(crate::UpdateStages::WorldDraw))
        .add_system(world_interact.label(crate::UpdateStages::Input))
        .add_system(draw_mode_controls.label(crate::UpdateStages::Input));
    }
}

#[derive(Component)]
struct Chunk {
    chunk_pos: gridmath::GridVec,
    chunk_texture_handle: Handle<Image>,
}

pub struct DrawOptions {
    pub update_bounds: bool,
    pub chunk_bounds: bool,
    pub world_stats: bool,
    pub force_redraw_all: bool,
}

pub struct BrushOptions {
    pub material: sandworld::ParticleType,
    pub radius: i32,
}

pub struct WorldStats {
    pub update_stats: Option<sandworld::WorldUpdateStats>,
}

fn draw_mode_controls(
    mut draw_options: ResMut<DrawOptions>,
    keys: Res<Input<KeyCode>>,
) {
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

fn render_chunk_texture(chunk: &sandworld::Chunk, draw_options: &DrawOptions) -> Image {
    let side_size = sandworld::CHUNK_SIZE as u32;
    Image::new(
        Extent3d { width: side_size, height: side_size, ..default() },
        bevy::render::render_resource::TextureDimension::D2,
        chunk.render_to_color_array(draw_options.update_bounds, draw_options.chunk_bounds),
        TextureFormat::Rgba8Unorm
    )
}

fn create_spawned_chunks(
    mut world: ResMut<sandworld::World>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    draw_options: Res<DrawOptions>,
) {
    let added_chunks = world.get_added_chunks();
    for chunkpos in added_chunks {
        if let Some(chunk) = world.get_chunk(&chunkpos) {
            //println!("New chunk at {} - created an entity to render it", chunkpos);
            let image = render_chunk_texture(chunk.as_ref(), &draw_options);
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
    chunk_query: Query<&Chunk>,
    draw_options: Res<DrawOptions>,
) {
    let updated_chunks = world.get_updated_chunks();

    if !draw_options.force_redraw_all && updated_chunks.is_empty() {
        return;
    }

    for chunk_comp in chunk_query.iter() {
        if draw_options.force_redraw_all || updated_chunks.contains(&chunk_comp.chunk_pos) {
            if let Some(chunk) = world.get_chunk(&chunk_comp.chunk_pos) {
                images.set_untracked(chunk_comp.chunk_texture_handle.clone(), render_chunk_texture(chunk.as_ref(), &draw_options));
            }
        }
    }
}

fn sand_update(mut world: ResMut<sandworld::World>, mut world_stats: ResMut<WorldStats>) {
    let stats = world.update();  
    world_stats.update_stats = Some(stats);  
}

fn world_interact(
    wnds: Res<Windows>,
    capture_state: Res<crate::ui::PointerCaptureState>,
    q_cam: Query<(&Camera, &GlobalTransform)>,
    mut sand: ResMut<sandworld::World>,
    buttons: Res<Input<MouseButton>>,
    brush_options: Res<BrushOptions>
) {
    if !capture_state.click_consumed && buttons.any_pressed([MouseButton::Left, MouseButton::Right]) {
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
            //if sand.contains(gridpos) {
                if buttons.pressed(MouseButton::Left){
                    sand.place_circle(gridpos, brush_options.radius, sandworld::Particle::new(brush_options.material), false);
                }
                else if buttons.pressed(MouseButton::Right) {
                    sand.place_circle(gridpos, 10, sandworld::Particle::new(sandworld::ParticleType::Air), true);
                }
            //}
        }
    }
}