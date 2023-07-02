//! Demonstrates how to connect `bevy::ui` buttons to [`ActionState`] components using the [`ActionStateDriver`] component on your button

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_cameras)
        .add_startup_system(spawn_ui.in_base_set(StartupSet::PostStartup))
        .add_system(move_player)
        .run();
}

#[derive(Actionlike, Component, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Left,
    Right,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    let mut input_map = InputMap::default();
    input_map.insert(KeyCode::Left, Action::Left);
    input_map.insert(KeyCode::Right, Action::Right);

    commands
        .spawn(SpriteBundle {
            transform: Transform {
                scale: Vec3::new(100., 100., 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::PINK,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(InputManagerBundle::<Action> {
            input_map,
            ..Default::default()
        })
        .insert(Player);
}

fn spawn_cameras(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_ui(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    let player_entity = player_query.single();

    // Left
    let left_button = commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(150.0)),
                ..Default::default()
            },
            background_color: Color::RED.into(),
            ..Default::default()
        })
        // This component links the button to the entity with the `ActionState` component
        .insert(ActionStateDriver {
            action: Action::Left,
            targets: player_entity.into(),
        })
        .id();

    // Right
    let right_button = commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(150.0)),
                ..Default::default()
            },
            background_color: Color::BLUE.into(),
            ..Default::default()
        })
        .insert(ActionStateDriver {
            action: Action::Right,
            targets: player_entity.into(),
        })
        .id();

    // Container for layout
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
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
    if action_state.pressed(Action::Left) {
        transform.translation.x -= 10.;
    }

    if action_state.pressed(Action::Right) {
        transform.translation.x += 10.;
    }
}
