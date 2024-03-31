use crate::sandsim::{BrushMode, BrushOptions};
use crate::chunk_display::DrawOptions;
use bevy::prelude::*;
use sandworld::ParticleType;
use iyes_perf_ui::prelude::*;
use bevy::ecs::system::lifetimeless::SRes;

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
        app
            .add_plugins(PerfUiPlugin)
            .add_perf_ui_entry_type::<PerfUiEntryChunkUpdates>()
            .add_perf_ui_entry_type::<PerfUiEntryLoadedRegions>()
            .add_perf_ui_entry_type::<PerfUiEntryUpdatedRegions>()
            .add_systems(Startup, setup_buttons)
            .add_systems(Startup, spawn_perf_ui)
            //.add_systems(Update, toggle_perf_ui.run_if(resource_changed::<DrawOptions>))
            .insert_resource(PointerCaptureState {
                click_consumed: false,
            })
            .add_systems(
                Update,
                button_system
                    .in_set(crate::UpdateStages::UI)
                    .before(crate::UpdateStages::Input),
            )
            
            ;
    }
}

fn spawn_perf_ui(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot {
            position: PerfUiPosition::TopRight,
            ..default()
        },
        PerfUiEntryFPS::default(),
        PerfUiEntryFrameTime::default(),
        PerfUiEntryFrameTimeWorst::default(),
        PerfUiEntryEntityCount::default(),
        PerfUiEntryChunkUpdates::default(),
        PerfUiEntryLoadedRegions::default(),
        PerfUiEntryUpdatedRegions::default(),
    ));
}

fn toggle_perf_ui(
    mut commands: Commands,
    draw_options: Res<DrawOptions>,
    perf_ui_query: Query<Entity, With<PerfUiRoot>>,
) {
    let ui_entity_result = perf_ui_query.get_single().ok();
    let has_ui = ui_entity_result.is_some();
    if draw_options.world_stats != has_ui {
        if draw_options.world_stats {
            println!("spawned perf UI");
            commands.spawn((
                PerfUiRoot {
                    position: PerfUiPosition::TopLeft,
                    ..default()
                },
                PerfUiEntryFPS::default(),
                PerfUiEntryFrameTime::default(),
                PerfUiEntryFrameTimeWorst::default(),
                PerfUiEntryEntityCount::default(),
                PerfUiEntryChunkUpdates::default(),
                PerfUiEntryLoadedRegions::default(),
                PerfUiEntryUpdatedRegions::default(),
            ));
        }
        else {
            let ui_entity = ui_entity_result.unwrap();
            commands.entity(ui_entity).despawn_recursive();
            println!("despawned perf UI");
        }
    }
}

#[derive(Component)]
pub struct PerfUiEntryLoadedRegions {
    pub sort_key: i32,
} 

#[derive(Component)]
pub struct PerfUiEntryUpdatedRegions {
    pub sort_key: i32,
    pub color_gradient: ColorGradient,
} 

#[derive(Component)]
pub struct PerfUiEntryChunkUpdates {
    pub sort_key: i32,
    pub color_gradient: ColorGradient,
} 

impl Default for PerfUiEntryLoadedRegions {
    fn default() -> Self {
        PerfUiEntryLoadedRegions {
            sort_key: iyes_perf_ui::utils::next_sort_key(),
        }
    }
}

impl Default for PerfUiEntryUpdatedRegions {
    fn default() -> Self {
        PerfUiEntryUpdatedRegions {
            sort_key: iyes_perf_ui::utils::next_sort_key(),
            color_gradient: ColorGradient::new_preset_ryg(3.0, 8.0, 12.0).unwrap(),
        }
    }
}

impl Default for PerfUiEntryChunkUpdates {
    fn default() -> Self {
        PerfUiEntryChunkUpdates {
            sort_key: iyes_perf_ui::utils::next_sort_key(),
            color_gradient: ColorGradient::new_preset_gyr(50.0, 500.0, 1000.0).unwrap(),
        }
    }
}

impl PerfUiEntry for PerfUiEntryLoadedRegions {
    type Value = usize;
    type SystemParam = SRes<crate::sandsim::WorldStats>;
    
    fn label(&self) -> &str {
        "Loaded Regions"
    }
    
    fn update_value(
        &self,
        world_stats: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        if let Some(update_stats) = &world_stats.update_stats {
            Some(update_stats.loaded_regions)
        }
        else {
            None
        }
    }
    
    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn width_hint(&self) -> usize {
        2
    }
}

impl PerfUiEntry for PerfUiEntryUpdatedRegions {
    type Value = u64;
    type SystemParam = SRes<crate::sandsim::WorldStats>;
    
    fn label(&self) -> &str {
        "Region Updates"
    }
    
    fn update_value(
        &self,
        world_stats: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        if let Some(update_stats) = &world_stats.update_stats {
            Some(update_stats.region_updates)
        }
        else {
            None
        }
    }
    
    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn width_hint(&self) -> usize {
        2
    }

    fn value_color(
        &self,
        value: &Self::Value,
    ) -> Option<Color> {
        self.color_gradient.get_color_for_value(*value as f32)
    }
}

impl PerfUiEntry for PerfUiEntryChunkUpdates {
    type Value = u64;
    type SystemParam = SRes<crate::sandsim::WorldStats>;
    
    fn label(&self) -> &str {
        "Chunk Updates"
    }
    
    fn update_value(
        &self,
        world_stats: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        if let Some(update_stats) = &world_stats.update_stats {
            Some(update_stats.chunk_updates)
        }
        else {
            None
        }
    }
    
    fn sort_key(&self) -> i32 {
        self.sort_key
    }

    fn width_hint(&self) -> usize {
        4
    }

    fn value_color(
        &self,
        value: &Self::Value,
    ) -> Option<Color> {
        self.color_gradient.get_color_for_value(*value as f32)
    }
}


#[derive(Component)]
struct ToolSelector {
    brush_mode: BrushMode,
    radius: i32,
}

fn spawn_tool_selector_button(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    label: &str,
    brush_mode: BrushMode,
    radius: i32,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(100.0),
                height: Val::Px(40.),
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
                flex_direction: FlexDirection::Row,
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
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::horizontal(Val::Px(25.)),
            align_self: AlignSelf::End,
            ..Default::default()
        },
        ..Default::default()
    }).with_children(|parent| {
        spawn_tool_selector_button(parent, &asset_server, "BOMB", BrushMode::Ball, 10);
        spawn_tool_selector_button(parent, &asset_server, "BEAM", BrushMode::Beam, 10);
        spawn_tool_selector_button(parent, &asset_server, "MELT", BrushMode::Melt, 10);
        spawn_tool_selector_button(parent, &asset_server, "BREAK", BrushMode::Break, 10);
        spawn_tool_selector_button(parent, &asset_server, "CHILL", BrushMode::Chill, 20);
        spawn_tool_selector_button(
             parent,
            &asset_server,
            "Stone",
            BrushMode::Place(ParticleType::Stone, 0),
            20,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Gravel",
            BrushMode::Place(ParticleType::Gravel, 0),
            10,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Sand",
            BrushMode::Place(ParticleType::Sand, 0),
            10,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Ice",
            BrushMode::Place(ParticleType::Ice, 0),
            10,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Water",
            BrushMode::Place(ParticleType::Water, 0),
            10,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Steam",
            BrushMode::Place(ParticleType::Steam, 0),
            10,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Lava",
            BrushMode::Place(ParticleType::Lava, 0),
            10,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "Emit",
            BrushMode::Place(ParticleType::Source, 0),
            1,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "LaserR",
            BrushMode::Place(ParticleType::LaserEmitter, 1),
            1,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "LaserL",
            BrushMode::Place(ParticleType::LaserEmitter, 3),
            1,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "LaserU",
            BrushMode::Place(ParticleType::LaserEmitter, 0),
            1,
        );
        spawn_tool_selector_button(
            parent,
            &asset_server,
            "LaserD",
            BrushMode::Place(ParticleType::LaserEmitter, 2),
            1,
        );
    });
    
}

fn button_system(
    mut capture_state: ResMut<PointerCaptureState>,
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &ToolSelector), With<Button>>,
    mut brush_options: ResMut<BrushOptions>,
) {
    capture_state.click_consumed = false;

    for (interaction, mut color, selector) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
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
