#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{prelude::*, window::PresentMode };

mod sandsim;
mod camera;
mod ui;
mod perf;
mod particle_set;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum UpdateStages {
    UI,
    Input,
    WorldUpdate,
    WorldDraw,
}

fn main(){
    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Project Sandbox - Bevy".to_string(),
                    width: 500.,
                    height: 300.,
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                },
                ..default()
             })
        )
        .add_plugin(crate::sandsim::SandSimulationPlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::perf::PerfControlPlugin)
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .run();
}
