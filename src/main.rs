#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{prelude::*, render::texture::ImageSettings, window::PresentMode };

mod sandsim;
mod camera;
mod ui;

fn main(){
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(crate::sandsim::SandSimulationPlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_plugin(crate::ui::UiPlugin)
        .insert_resource(WindowDescriptor {
            title: "Project Sandbox - Bevy".to_string(),
            width: 500.,
            height: 300.,
            present_mode: PresentMode::AutoVsync,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(ImageSettings::default_nearest())
        .run();
}
