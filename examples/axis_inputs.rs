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
    #[actionlike(DualAxis)]
    Move,
    Throttle,
    #[actionlike(Axis)]
    Rudder,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Describes how to convert from player inputs into those actions
    let input_map = InputMap::default()
        // Let's bind the left stick for the move action
        .with_dual_axis(Action::Move, GamepadStick::LEFT)
        // And then bind the right gamepad trigger to the throttle action
        .with(Action::Throttle, GamepadButton::RightTrigger2)
        // And we'll use the right stick's x-axis as a rudder control
        .with_axis(
            // Add an AxisDeadzone to process horizontal values of the right stick.
            // This will trigger if the axis is moved 10% or more in either direction.
            Action::Rudder,
            GamepadControlAxis::RIGHT_X.with_deadzone_symmetric(0.1),
        );
    commands
        .spawn(InputManagerBundle::with_map(input_map))
        .insert(Player);
}

// Query for the `ActionState` component in your game logic systems!
fn move_player(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single().expect("Player actions not found");

    let axis_pair = action_state.clamped_axis_pair(&Action::Move);
    println!("Move:");
    println!("   distance: {}", axis_pair.length());
    println!("          x: {}", axis_pair.x);
    println!("          y: {}", axis_pair.y);

    if action_state.pressed(&Action::Throttle) {
        // Note that some gamepad buttons are also tied to axes, so even though we used a
        // GamepadButton::RightTrigger2 binding to trigger the throttle action,
        // we can get a variable value here if you have a variable right trigger on your gamepad.
        //
        // If you don't have a variable trigger, this will just return 0.0 when not pressed and 1.0
        // when pressed.
        let value = action_state.clamped_button_value(&Action::Throttle);
        println!("Throttle: {value}");
    }

    let value = action_state.clamped_value(&Action::Rudder);
    println!("Rudder: {value}");
}
