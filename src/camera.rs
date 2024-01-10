use bevy::prelude::*;
use gridmath::{GridBounds, GridVec};
use sandworld::CHUNK_SIZE;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
            .add_system(camera_movement.in_set(crate::UpdateStages::Input));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(
            sandworld::WORLD_WIDTH as f32 / 2.,
            sandworld::WORLD_HEIGHT as f32 / 2.,
            0.,
        )),
        ..default()
    });
}

pub fn cam_bounds(ortho: &OrthographicProjection, transform: &GlobalTransform) -> GridBounds {
    let camera_pos = transform.translation();
    let scale =   ortho.scale;
    let left =    (ortho.area.min.x * 2.5 * scale + camera_pos.x - CHUNK_SIZE as f32).floor() as i32;
    let right =   (ortho.area.max.x * 2.5 * scale + camera_pos.x + CHUNK_SIZE as f32).ceil()  as i32;
    let bottom =  (ortho.area.min.y * 2.5 * scale + camera_pos.y - CHUNK_SIZE as f32).floor() as i32;
    let top =     (ortho.area.max.y * 2.5 * scale + camera_pos.y + CHUNK_SIZE as f32).ceil()  as i32;

    GridBounds::new_from_extents(
        GridVec::new(left, bottom),
        GridVec::new(right, top))
}

fn camera_movement(
    mut query: Query<(&Camera, &mut OrthographicProjection, &mut Transform)>,
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
) {
    let (_camera, mut ortho, mut camera_transform) = query.single_mut();

    let mut log_scale = ortho.scale.ln();
    let move_speed = 128.;
    let zoom_speed = 0.5;

    let max_zoom = 2.;
    let min_zoom = 0.1;

    if keys.pressed(KeyCode::D) || keys.pressed(KeyCode::Right) {
        camera_transform.translation =
            (camera_transform.right() * move_speed * time.delta_seconds())
                + camera_transform.translation;
    }
    if keys.pressed(KeyCode::A) || keys.pressed(KeyCode::Left) {
        camera_transform.translation =
            (camera_transform.left() * move_speed * time.delta_seconds())
                + camera_transform.translation;
    }
    if keys.pressed(KeyCode::W) || keys.pressed(KeyCode::Up) {
        camera_transform.translation = (camera_transform.up() * move_speed * time.delta_seconds())
            + camera_transform.translation;
    }
    if keys.pressed(KeyCode::S) || keys.pressed(KeyCode::Down) {
        camera_transform.translation =
            (camera_transform.down() * move_speed * time.delta_seconds())
                + camera_transform.translation;
    }

    if keys.any_pressed([KeyCode::PageUp, KeyCode::RBracket]) {
        log_scale -= zoom_speed * time.delta_seconds();
    }
    if keys.any_pressed([KeyCode::PageDown, KeyCode::LBracket]) {
        log_scale += zoom_speed * time.delta_seconds();
    }

    ortho.scale = log_scale.exp().clamp(min_zoom, max_zoom);
}
