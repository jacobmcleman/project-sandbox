use bevy::prelude::*;
use crate::sandsim::BrushOptions;
use sandworld::ParticleType;

pub struct UiPlugin;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_buttons)
        .add_system(button_system);
    }
}

#[derive(Component)]
struct ToolSelector {
    material: sandworld::ParticleType,
    radius: i32,
}

fn spawn_tool_selector_button(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>, 
    label: &str, material: ParticleType, radius: i32) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(100.0), Val::Px(40.0)),
                // center button
                margin: UiRect {
                    left: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    ..default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            color: NORMAL_BUTTON.into(),
            ..default()
        })
        .insert(ToolSelector {
            material,
            radius,
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle::from_section(
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
    spawn_tool_selector_button(&mut commands, &asset_server, "Sand", ParticleType::Sand, 10);
    spawn_tool_selector_button(&mut commands, &asset_server, "Stone", ParticleType::Stone, 10);
    spawn_tool_selector_button(&mut commands, &asset_server, "Water", ParticleType::Water, 10);
    spawn_tool_selector_button(&mut commands, &asset_server, "Source", ParticleType::Source, 1);
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &ToolSelector),
        With<Button>,
    >,
    mut brush_options: ResMut<BrushOptions>,
) {
    for (interaction, mut color, selector) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                brush_options.material = selector.material;
                brush_options.radius = selector.radius;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }

        if selector.material == brush_options.material {
            *color = PRESSED_BUTTON.into();
        }
    }
}