use crate::sandsim::{BrushMode, BrushOptions};
use bevy::prelude::*;
use sandworld::ParticleType;

pub struct UiPlugin;

#[derive(Resource)]
pub struct PointerCaptureState {
    pub click_consumed: bool,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_buttons)
            .add_startup_system(spawn_performance_info_text)
            .insert_resource(PointerCaptureState {
                click_consumed: false,
            })
            .add_system(
                button_system
                    .in_set(crate::UpdateStages::UI)
                    .before(crate::UpdateStages::Input),
            )
            .add_system(
                update_performance_text
                    .in_set(crate::UpdateStages::UI)
                    .after(crate::UpdateStages::WorldUpdate),
            );
    }
}

#[derive(Component)]
struct PerformanceReadout;

fn spawn_performance_info_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(
            TextBundle::from_sections([
                TextSection {
                    value: "FPS: 69".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 30.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                },
                TextSection {
                    value: "\nLoaded Regions: 000".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                },
                TextSection {
                    value: "\nUpdated Regions: 000".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                },
                TextSection {
                    value: "\nChunk Updates: 000".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                },
                TextSection {
                    value: "\nAvg time per chunk".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.6),
                    },
                },
                TextSection {
                    value: "\nAvg render time per chunk".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.9, 0.9, 0.6),
                    },
                },
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .insert(PerformanceReadout {});
}

fn update_performance_text(
    mut text_query: Query<(&PerformanceReadout, &mut Text, &mut Visibility)>,
    stats: Res<crate::sandsim::WorldStats>,
    draw_options: Res<crate::sandsim::DrawOptions>,
    frame_times: Res<crate::perf::FrameTimes>,
) {
    let (_, mut text, mut vis) = text_query.single_mut();
    if draw_options.world_stats {
        *vis = Visibility::Inherited;

        text.sections[0].value = format!(
            "FPS: {} ({:.1}ms)",
            (1. / frame_times.current_avg).round() as u32,
            frame_times.current_avg * 1000.
        );

        if let Some(world_stats) = &stats.update_stats {
            text.sections[1].value = format!("\nLoaded Regions: {}", world_stats.loaded_regions);
            text.sections[2].value = format!("\nRegion Updates: {}", world_stats.region_updates);
            text.sections[3].value = format!(
                "\nChunk Updates [Target]: {} [{}]",
                world_stats.chunk_updates, stats.target_chunk_updates
            );

            let mut texture_update_time_avg = 0.;
            let mut texture_update_per_chunk_avg = 0.;
            for (time, count) in &stats.chunk_texture_update_time {
                texture_update_time_avg += time;
                texture_update_per_chunk_avg += time / (*count as f64);
            }
            texture_update_time_avg =
                texture_update_time_avg / (stats.chunk_texture_update_time.len() as f64);
            texture_update_per_chunk_avg =
                texture_update_per_chunk_avg / (stats.chunk_texture_update_time.len() as f64);

            text.sections[5].value = format!(
                "\nTex Update time:  {:.2}ms - Avg time per chunk: {:.3}ms",
                texture_update_time_avg * 1000.,
                texture_update_per_chunk_avg * 1000.
            );
        }

        let mut chunk_updates_per_second_avg = 0.;
        let mut total_sand_update_second_avg = 0.;
        for (time, count) in &stats.sand_update_time {
            chunk_updates_per_second_avg += *count as f64 / time;
            total_sand_update_second_avg += time;
        }
        chunk_updates_per_second_avg =
            chunk_updates_per_second_avg / (stats.sand_update_time.len() as f64);
        total_sand_update_second_avg =
            total_sand_update_second_avg / (stats.sand_update_time.len() as f64);

        text.sections[4].value = format!(
            "\nSand update time: {:.2}ms - Avg time per chunk: {:.3}ms",
            total_sand_update_second_avg * 1000.,
            1000. / chunk_updates_per_second_avg
        );
    } else {
        *vis = Visibility::Hidden;
    }
}

#[derive(Component)]
struct ToolSelector {
    brush_mode: BrushMode,
    radius: i32,
}

fn spawn_tool_selector_button(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    label: &str,
    brush_mode: BrushMode,
    radius: i32,
) {
    commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(40.0)),
                margin: UiRect {
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    ..default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                align_self: AlignSelf::FlexEnd,
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(ToolSelector { brush_mode, radius })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
            ));
        });
}

fn setup_buttons(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_tool_selector_button(&mut commands, &asset_server, "MELT", BrushMode::Melt, 10);
    spawn_tool_selector_button(&mut commands, &asset_server, "BREAK", BrushMode::Break, 10);
    spawn_tool_selector_button(&mut commands, &asset_server, "CHILL", BrushMode::Chill, 20);
    spawn_tool_selector_button(&mut commands, &asset_server, "PIPE", BrushMode::PipeInlet, 0);
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Stone",
        BrushMode::Place(ParticleType::Stone, 0),
        20,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Gravel",
        BrushMode::Place(ParticleType::Gravel, 0),
        10,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Sand",
        BrushMode::Place(ParticleType::Sand, 0),
        10,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Ice",
        BrushMode::Place(ParticleType::Ice, 0),
        10,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Water",
        BrushMode::Place(ParticleType::Water, 0),
        10,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Steam",
        BrushMode::Place(ParticleType::Steam, 0),
        10,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Lava",
        BrushMode::Place(ParticleType::Lava, 0),
        10,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "Emit",
        BrushMode::Place(ParticleType::Source, 0),
        1,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "LaserR",
        BrushMode::Place(ParticleType::LaserEmitter, 1),
        1,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "LaserL",
        BrushMode::Place(ParticleType::LaserEmitter, 3),
        1,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "LaserU",
        BrushMode::Place(ParticleType::LaserEmitter, 0),
        1,
    );
    spawn_tool_selector_button(
        &mut commands,
        &asset_server,
        "LaserD",
        BrushMode::Place(ParticleType::LaserEmitter, 2),
        1,
    );
}

fn button_system(
    mut capture_state: ResMut<PointerCaptureState>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &ToolSelector), With<Button>>,
    mut brush_options: ResMut<BrushOptions>,
) {
    capture_state.click_consumed = false;

    for (interaction, mut color, selector) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                brush_options.brush_mode = selector.brush_mode.clone();
                brush_options.radius = selector.radius;
                capture_state.click_consumed = true;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }

        if selector.brush_mode == brush_options.brush_mode {
            *color = PRESSED_BUTTON.into();
        }
    }
}