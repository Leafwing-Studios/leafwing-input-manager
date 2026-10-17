//! Demonstrates input isolation across [`InputMap`] replacements.
//!
//! Run with:
//!
//! `cargo run --example input_map_isolation`
//!
//! Steps:
//!
//! 1. Press and hold `Space` while `InputMap A` is active.
//! 2. Press `Tab` to switch to `InputMap B` while still holding `Space`.
//! 3. Release `Space`.
//! 4. Press `Space` again.
//!
//! Expected behavior:
//!
//! - the first `Space` press triggers `Jump`;
//! - switching maps while holding `Space` does not trigger `Confirm`;
//! - releasing that held `Space` does not trigger `Confirm.just_released`;
//! - only the second press after the release triggers `Confirm`.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "InputMap Isolation Example".to_owned(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, (print_instructions, spawn_player))
        .add_systems(
            Update,
            (
                report_physical_space_events,
                swap_input_map_on_tab,
                report_action_events,
            ),
        )
        .run();
}

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    Jump,
    Confirm,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum ActiveMap {
    A,
    B,
}

#[derive(Component)]
struct Player;

fn print_instructions() {
    println!("InputMap isolation demo");
    println!("-----------------------");
    println!("Space is bound to Jump in InputMap A.");
    println!("Press Tab to replace the map so Space is bound to Confirm in InputMap B.");
    println!("Hold Space, press Tab, then release Space.");
    println!("You should NOT see `Confirm just released` from that release.");
    println!("Press Space again after releasing it to trigger Confirm normally.");
    println!();
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((Player, ActiveMap::A, input_map_a()));
    println!("Spawned player with InputMap A: Space -> Jump");
}

fn input_map_a() -> InputMap<Action> {
    InputMap::new([(Action::Jump, KeyCode::Space)])
}

fn input_map_b() -> InputMap<Action> {
    InputMap::new([(Action::Confirm, KeyCode::Space)])
}

fn report_physical_space_events(
    keys: Res<ButtonInput<KeyCode>>,
    map: Single<&ActiveMap, With<Player>>,
) {
    let active_map = *map;

    if keys.just_pressed(KeyCode::Space) {
        println!("Physical Space pressed while InputMap {active_map:?} is active");
    }

    if keys.just_released(KeyCode::Space) {
        println!("Physical Space released while InputMap {active_map:?} is active");
    }
}

fn swap_input_map_on_tab(
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut InputMap<Action>, &mut ActiveMap), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    let (mut input_map, mut active_map) = player.into_inner();

    match *active_map {
        ActiveMap::A => {
            *input_map = input_map_b();
            *active_map = ActiveMap::B;
            println!("Switched to InputMap B: Space -> Confirm");
        }
        ActiveMap::B => {
            *input_map = input_map_a();
            *active_map = ActiveMap::A;
            println!("Switched to InputMap A: Space -> Jump");
        }
    }
}

fn report_action_events(action_state: Single<&ActionState<Action>, With<Player>>) {
    if action_state.just_pressed(&Action::Jump) {
        println!("Action event: Jump just pressed");
    }

    if action_state.just_released(&Action::Jump) {
        println!("Action event: Jump just released");
    }

    if action_state.just_pressed(&Action::Confirm) {
        println!("Action event: Confirm just pressed");
    }

    if action_state.just_released(&Action::Confirm) {
        println!("Action event: Confirm just released");
    }
}
