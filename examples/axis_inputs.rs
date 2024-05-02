use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugins(InputManagerPlugin::<Action>::default())
        // Spawn an entity with Player, InputMap, and ActionState components
        .add_systems(Startup, spawn_player)
        // Read the ActionState in your systems using queries!
        .add_systems(Update, move_player)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Move,
    Throttle,
    Rudder,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Describes how to convert from player inputs into those actions
    let input_map = InputMap::default()
        // Configure the left stick as a dual-axis control
        .with(Action::Move, DualAxis::left_stick())
        // Let's bind the right gamepad trigger to the throttle action
        .with(Action::Throttle, GamepadButtonType::RightTrigger2)
        // And we'll use the right stick's x-axis as a rudder control
        .with(
            // Add an AxisDeadzone to process horizontal values of the right stick.
            // This will trigger if the axis is moved 10% or more in either direction.
            Action::Rudder,
            SingleAxis::new(GamepadAxisType::RightStickX).with_deadzone_magnitude(0.1),
        );
    commands
        .spawn(InputManagerBundle::with_map(input_map))
        .insert(Player);
}

// Query for the `ActionState` component in your game logic systems!
fn move_player(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();

    // Each action has a button-like state of its own that you can check
    if action_state.pressed(&Action::Move) {
        // We're working with gamepads, so we want to defensively ensure that we're using the clamped values
        let axis_pair = action_state.clamped_axis_pair(&Action::Move).unwrap();
        println!("Move:");
        println!("   distance: {}", axis_pair.length());
        println!("          x: {}", axis_pair.x());
        println!("          y: {}", axis_pair.y());
    }

    if action_state.pressed(&Action::Throttle) {
        // Note that some gamepad buttons are also tied to axes, so even though we used a
        // GamepadButtonType::RightTrigger2 binding to trigger the throttle action,
        // we can get a variable value here if you have a variable right trigger on your gamepad.
        //
        // If you don't have a variable trigger, this will just return 0.0 when not pressed and 1.0
        // when pressed.
        let value = action_state.clamped_value(&Action::Throttle);
        println!("Throttle: {value}");
    }

    if action_state.pressed(&Action::Rudder) {
        let value = action_state.clamped_value(&Action::Rudder);
        println!("Rudder: {value}");
    }
}
