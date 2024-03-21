use bevy::{prelude::*, core::Zeroable};
use gridmath::{GridBounds, GridVec};
use sandworld::CHUNK_SIZE;

pub struct CameraPlugin;

#[derive(Component)]
struct IdleMover {
    x_move: f32,
    y_move: f32,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, camera_movement.in_set(crate::UpdateStages::Input));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(
            sandworld::WORLD_WIDTH as f32 / 2.,
            sandworld::WORLD_HEIGHT as f32 / 2.,
            0.,
        )),
        ..default()
    }, IdleMover { x_move: 0., y_move: 0. })
    );
}

pub fn cam_bounds(ortho: &OrthographicProjection, camera: &Camera, camera_transform: &GlobalTransform) -> GridBounds {
    let bottom_left = camera.ndc_to_world(camera_transform, Vec3::new(-1., -1., 0.)).unwrap().truncate().floor();
    let top_right = camera.ndc_to_world(camera_transform, Vec3::new(1., 1., 0.)).unwrap().truncate().ceil();

    GridBounds::new_from_extents(
        GridVec::new(bottom_left.x as i32, bottom_left.y as i32),
        GridVec::new(top_right.x as i32, top_right.y as i32))
}

fn camera_movement(
    mut query: Query<(&Camera, &mut OrthographicProjection, &mut Transform, &mut IdleMover)>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (_camera, mut ortho, mut camera_transform, mut idle) = query.single_mut();

    let mut log_scale = ortho.scale.ln();
    let move_speed = 256.;
    let zoom_speed = 0.5;

    let max_zoom = 1.;
    let min_zoom = 0.1;

    if keys.pressed(KeyCode::ShiftLeft) {
        if keys.just_pressed(KeyCode::KeyD) {
            idle.x_move -= 10.;
        }
        if keys.just_pressed(KeyCode::KeyA) {
            idle.x_move += 10.;
        }
        if keys.just_pressed(KeyCode::KeyS) {
            idle.y_move -= 10.;
        }
        if keys.just_pressed(KeyCode::KeyW) {
            idle.y_move += 10.;
        }
    }
    else {
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            camera_transform.translation =
                (camera_transform.right() * move_speed * time.delta_seconds())
                    + camera_transform.translation;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            camera_transform.translation =
                (camera_transform.left() * move_speed * time.delta_seconds())
                    + camera_transform.translation;
        }
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            camera_transform.translation = (camera_transform.up() * move_speed * time.delta_seconds())
                + camera_transform.translation;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            camera_transform.translation =
                (camera_transform.down() * move_speed * time.delta_seconds())
                    + camera_transform.translation;
        }
    }
    

    if idle.x_move != 0. {
        camera_transform.translation =
            (idle.x_move * *camera_transform.left() * time.delta_seconds())
                + camera_transform.translation;
    }
    if idle.y_move != 0. {
        camera_transform.translation =
            (idle.y_move * *camera_transform.up() * time.delta_seconds())
                + camera_transform.translation;
    }

    if keys.any_pressed([KeyCode::PageUp, KeyCode::BracketRight]) {
        log_scale -= zoom_speed * time.delta_seconds();
    }
    if keys.any_pressed([KeyCode::PageDown, KeyCode::BracketRight]) {
        log_scale += zoom_speed * time.delta_seconds();
    }

    ortho.scale = log_scale.exp().clamp(min_zoom, max_zoom);
}
