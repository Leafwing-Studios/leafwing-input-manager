//! Demonstrates how to connect `bevy::ui` buttons to [`ActionState`] components using the [`ActionStateDriver`] component on your button

use bevy::{color::palettes, prelude::*};
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .add_systems(Startup, spawn_cameras)
        .add_systems(PostStartup, spawn_ui)
        .add_systems(Update, move_player)
        .run();
}

#[derive(Actionlike, Component, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Left,
    Right,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    let input_map = InputMap::new([
        (Action::Left, KeyCode::ArrowLeft),
        (Action::Right, KeyCode::ArrowRight),
    ]);

    commands
        .spawn(SpriteBundle {
            transform: Transform {
                scale: Vec3::new(100., 100., 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: palettes::css::PINK.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(InputManagerBundle::with_map(input_map))
        .insert(Player);
}

fn spawn_cameras(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_ui(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    let player_entity = player_query.single();

    // Left
    let left_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(150.0),
                    ..Default::default()
                },

                ..Default::default()
            },
            BackgroundColor(palettes::css::RED.into()),
        ))
        // This component links the button to the entity with the `ActionState` component
        .insert(ActionStateDriver {
            action: Action::Left,
            targets: player_entity.into(),
        })
        .id();

    // Right
    let right_button = commands
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(150.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            BackgroundColor(palettes::css::BLUE.into()),
        ))
        .insert(ActionStateDriver {
            action: Action::Right,
            targets: player_entity.into(),
        })
        .id();

    // Container for layout
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            background_color: Color::NONE.into(),
            ..Default::default()
        })
        .push_children(&[left_button, right_button]);
}

fn move_player(mut query: Query<(&ActionState<Action>, &mut Transform), With<Player>>) {
    let (action_state, mut transform) = query.single_mut();

    // To only perform the action once when the button is first clicked,
    // use `.just_pressed` instead.
    // To trigger when the click is released, use `.just_released`
    if action_state.pressed(&Action::Left) {
        transform.translation.x -= 10.;
    }

    if action_state.pressed(&Action::Right) {
        transform.translation.x += 10.;
    }
}
