#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::mem::size_of;

use bevy::{prelude::*, window::PresentMode, window::WindowResolution};
use bevy_xpbd_2d::prelude::*;

mod camera;
mod perf;
mod sandsim;
mod ui;
mod worldgen;
mod polyline;
mod chunk_display;
mod chunk_colliders;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
enum UpdateStages {
    UI,
    Input,
    WorldUpdate,
    WorldDraw,
}

fn main() {
    println!("particle size: {}", size_of::<sandworld::Particle>());

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Project Sandbox - Bevy".to_string(),
                        resolution: WindowResolution::new(1920., 1080.),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(crate::sandsim::SandSimulationPlugin)
        .add_plugins(crate::camera::CameraPlugin)
        .add_plugins(crate::ui::UiPlugin)
        .add_plugins(crate::perf::PerfControlPlugin)
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(Gravity(Vec2::NEG_Y * 100.0))
        .run();
}
