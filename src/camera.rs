use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera)
        .add_system(camera_movement.label(crate::UpdateStages::Input));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(
            sandworld::WORLD_WIDTH as f32 / 2.,
            sandworld::WORLD_HEIGHT as f32 / 2.,
            0.
        )),
        ..default()
    });
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