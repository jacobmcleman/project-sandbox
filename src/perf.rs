use std::collections::VecDeque;

use bevy::prelude::*;

#[derive(Resource)]
pub struct FrameTimes {
    recent: VecDeque<f64>,
    pub current_avg: f64,
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
        })
        .insert_resource(PerfSettings {
            target_frame_rate: 60,
        })
        .add_system(frame_timing);
    }
}

fn frame_timing(time: Res<Time>, mut timing_data: ResMut<FrameTimes>) {
    timing_data.recent.push_back(time.delta_seconds_f64());
    if timing_data.recent.len() > 64 {
        timing_data.recent.pop_front();
    }

    timing_data.current_avg = timing_data.recent.iter().fold(0., |total, val| total + val)
        / (timing_data.recent.len() as f64);
}
