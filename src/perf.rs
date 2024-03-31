use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource)]
pub struct FrameTimes {
    recent: VecDeque<f64>,
    pub current_avg: f64,
    pub recent_worst: f64,
}

#[derive(Resource)]
pub struct PerfSettings {
    pub target_frame_rate: u32,
}

pub struct PerfControlPlugin;

impl Plugin for PerfControlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameTimes {
            recent: VecDeque::new(),
            current_avg: 0.,
            recent_worst: 0.,
        })
        .insert_resource(PerfSettings {
            target_frame_rate: 60,
        })
        .add_systems(PostUpdate, frame_timing)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        ;
    }
}

fn frame_timing(time: Res<Time>, mut timing_data: ResMut<FrameTimes>) {
    timing_data.recent.push_back(time.delta_seconds_f64());
    if timing_data.recent.len() > 64 {
        timing_data.recent.pop_front();
    }
    
    let mut worst = timing_data.current_avg;
    let mut total = 0.;
    for time in timing_data.recent.iter() {
        total += time;
        if *time > worst { worst = *time }
    }
    timing_data.current_avg = total / (timing_data.recent.len() as f64);
    timing_data.recent_worst = worst;
}
