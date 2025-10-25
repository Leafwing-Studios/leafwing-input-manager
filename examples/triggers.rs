use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, input)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Throttle,
    Brake,
    #[actionlike(Axis)]
    PropulsionAxis,
}

fn startup(mut commands: Commands) {
    let input_map = InputMap::default()
        .with(Action::Throttle, GamepadButton::RightTrigger2)
        .with(Action::Brake, GamepadButton::LeftTrigger2)
        .with_axis(
            Action::PropulsionAxis,
            VirtualAxis::new(GamepadButton::LeftTrigger2, GamepadButton::RightTrigger2)
                .with_deadzone_symmetric(0.1),
        );

    commands.spawn(input_map);
}

fn input(action_state: Single<&ActionState<Action>>) {
    let brake_value = action_state.button_value(&Action::Brake);
    let brake_pressed = action_state.pressed(&Action::Brake);
    let brake_print = format!(
        "Brake (LeftTrigger2): {brake_value:.2}, Pressed: {}",
        brake_pressed
    );
    info!("{brake_print}");

    let throttle_value = action_state.button_value(&Action::Throttle);
    let throttle_pressed = action_state.pressed(&Action::Throttle);
    let throttle_print = format!(
        "Throttle (RightTrigger2): {throttle_value:.2}, Pressed: {}",
        throttle_pressed
    );
    info!("{throttle_print}");

    let propulsion_value = action_state.value(&Action::PropulsionAxis);
    let propulsion_print =
        format!("Propulsion Axis (Both): {propulsion_value:.2}, Pressed: Axis can't be pressed",);
    info!("{propulsion_print}");

    info!("---");
}
