use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<Action>::default())
        // Spawn an entity with Player, InputMap, and ActionState components
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(move_player)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Move,
    Throttle,
    Rudder,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            // Stores "which actions are currently activated"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new([
                // Configure the left stick as a dual-axis
                (
                    DualGamepadAxis {
                        x_axis: GamepadAxisType::LeftStickX,
                        y_axis: GamepadAxisType::LeftStickY,
                        // We want to trigger our move action when the left stick is moved more than 10%
                        // in any direction.
                        //
                        // Note: The Bevy `GamepadSettings` such as the axis deadzone with still apply.
                        // If bevy filters out a gamepad event,
                        // it will be filtered out of leafwing, too.
                        deadzone: 0.1,
                    },
                    Action::Move,
                ),
            ])
            // Let's also add a gamepad button binding to the right trigger
            .insert(GamepadButtonType::RightTrigger2, Action::Throttle)
            // And we'll use the x axis on the right stick as a rudder control
            .insert(
                SingleGamepadAxis {
                    axis: GamepadAxisType::RightStickX,
                    // This will trigger if the axis is moved 10% or more in either direction.
                    negative_low: -0.1,
                    positive_low: 0.1,
                },
                Action::Rudder,
            )
            // Listen for events on the first gamepad
            .set_gamepad(Gamepad(0))
            .build(),
        });
}

// Query for the `ActionState` component in your game logic systems!
fn move_player(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action has a button-like state of its own that you can check
    if action_state.pressed(Action::Move) {
        let axis_pair = action_state.action_axis_pair(Action::Move).unwrap();
        println!("Move:");
        println!("   distance: {}", axis_pair.magnitude());
        println!("          x: {}", axis_pair.x());
        println!("          y: {}", axis_pair.y());
    }

    if action_state.pressed(Action::Throttle) {
        // Note that some gamepad buttons are also tied to axes,
        // so even though we used a GamepadbuttonType::RightTrigger2 binding to trigger the
        // throttle action,
        // we can get a variable value here if you have a variable right trigger on your gamepad.
        //
        // If you don't have a variable trigger,
        // this will just return 0.0 when not pressed and 1.0 when pressed.
        let value = action_state.action_value(Action::Throttle);
        println!("Throttle: {}", value);
    }

    if action_state.pressed(Action::Rudder) {
        let value = action_state.action_value(Action::Rudder);
        println!("Rudder: {}", value);
    }
}
