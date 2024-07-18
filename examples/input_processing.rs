use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        .add_systems(Update, check_data)
        .run();
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Move,
    LookAround,
}

impl Actionlike for Action {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Action::Move => InputControlKind::DualAxis,
            Action::LookAround => InputControlKind::DualAxis,
        }
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    let input_map = InputMap::default()
        .with_dual_axis(
            Action::Move,
            KeyboardVirtualDPad::WASD
                // You can configure a processing pipeline to handle axis-like user inputs.
                //
                // This step adds a circular deadzone that normalizes input values
                // by clamping their magnitude to a maximum of 1.0,
                // excluding those with a magnitude less than 0.1,
                // and scaling other values linearly in between.
                .with_circle_deadzone(0.1)
                // Followed by appending Y-axis inversion for the next processing step.
                .inverted_y()
                // Or reset the pipeline, leaving no any processing applied.
                .reset_processing_pipeline(),
        )
        .with_dual_axis(
            Action::LookAround,
            // You can also use a sequence of processors as the processing pipeline.
            MouseMove::default().replace_processing_pipeline([
                // The first processor is a circular deadzone.
                CircleDeadZone::new(0.1).into(),
                // The next processor doubles inputs normalized by the deadzone.
                DualAxisSensitivity::all(2.0).into(),
            ]),
        );
    commands
        .spawn(InputManagerBundle::with_map(input_map))
        .insert(Player);
}

fn check_data(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    for action in action_state.get_pressed() {
        println!(
            "Pressed {action:?} with data: {:?}",
            action_state.axis_pair(&action)
        );
    }
}
