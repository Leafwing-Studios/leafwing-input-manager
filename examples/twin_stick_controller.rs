//! This is a fairly complete example that implements a twin stick controller.
//!
//! The controller supports both gamepad/MKB inputs and switches between them depending on
//! the most recent input.
//!
//! This example builds on top of several concepts introduced in other examples. In particular,
//! the `default_controls`. `mouse_position`, and `action_state_resource` examples.

use bevy::{
    input::gamepad::GamepadEvent, input::keyboard::KeyboardInput, prelude::*, window::PrimaryWindow,
};
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        // Defined below, detects whether MKB or gamepad are active
        .add_plugins(InputModeManagerPlugin)
        .init_resource::<ActionState<PlayerAction>>()
        .insert_resource(PlayerAction::default_input_map())
        // Set up the scene
        .add_systems(Startup, setup_scene)
        // Set up the input processing
        .add_systems(
            Update,
            player_mouse_look.run_if(in_state(ActiveInput::MouseKeyboard)),
        )
        .add_systems(Update, control_player.after(player_mouse_look))
        .run();
}

// ----------------------------- Player Action Input Handling -----------------------------
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
#[actionlike(DualAxis)]
pub enum PlayerAction {
    Move,
    Look,
    #[actionlike(Button)]
    Shoot,
}

impl PlayerAction {
    /// Define the default bindings to the input
    fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Default gamepad input bindings
        input_map.insert_dual_axis(Self::Move, GamepadStick::LEFT);
        input_map.insert_dual_axis(Self::Look, GamepadStick::RIGHT);
        input_map.insert(Self::Shoot, GamepadButton::RightTrigger);

        // Default kbm input bindings
        input_map.insert_dual_axis(Self::Move, VirtualDPad::wasd());
        input_map.insert_dual_axis(Self::Look, VirtualDPad::arrow_keys());
        input_map.insert(Self::Shoot, MouseButton::Left);

        input_map
    }
}

// ----------------------------- Input mode handling -----------------------------
pub struct InputModeManagerPlugin;

impl Plugin for InputModeManagerPlugin {
    fn build(&self, app: &mut App) {
        // Init a state to record the current active input
        app.init_state::<ActiveInput>()
            // System to switch to gamepad as active input
            .add_systems(
                Update,
                activate_gamepad.run_if(in_state(ActiveInput::MouseKeyboard)),
            )
            // System to switch to MKB as active input
            .add_systems(Update, activate_mkb.run_if(in_state(ActiveInput::Gamepad)));
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum ActiveInput {
    #[default]
    MouseKeyboard,
    Gamepad,
}

/// Switch the gamepad when any button is pressed or any axis input used
fn activate_gamepad(
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.read() {
        match ev {
            GamepadEvent::Button(_) | GamepadEvent::Axis(_) => {
                info!("Switching to gamepad input");
                next_state.set(ActiveInput::Gamepad);
                return;
            }
            _ => (),
        }
    }
}

/// Switch to mouse and keyboard input when any keyboard button is pressed
fn activate_mkb(
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut kb_evr: EventReader<KeyboardInput>,
) {
    for _ev in kb_evr.read() {
        info!("Switching to mouse and keyboard input");
        next_state.set(ActiveInput::MouseKeyboard);
    }
}

// ----------------------------- Mouse input handling-----------------------------

fn player_mouse_look(
    camera_query: Query<(&GlobalTransform, &Camera)>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut action_state: ResMut<ActionState<PlayerAction>>,
) {
    let (camera_transform, camera) = camera_query.single().expect("Need a single camera");
    let player_transform = player_query.single().expect("Need a single player");
    let window = window_query.single().expect("Need a single primary window");

    // Many steps can fail here, so we'll wrap in an option pipeline
    // First check if the cursor is in window
    // Then check if the ray intersects the plane defined by the player
    // Then finally compute the point along the ray to look at
    let player_position = player_transform.translation;
    if let Some(p) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .and_then(|ray| {
            Some(ray).zip(ray.intersect_plane(player_position, InfinitePlane3d::new(Vec3::Y)))
        })
        .map(|(ray, p)| ray.get_point(p))
    {
        let diff = (p - player_position).xz();
        if diff.length_squared() > 1e-3f32 {
            // Get the mutable action data to set the axis
            let action_data = action_state.dual_axis_data_mut_or_default(&PlayerAction::Look);

            // Flipping y sign here to be consistent with gamepad input.
            // We could also invert the gamepad y-axis
            action_data.pair = Vec2::new(diff.x, -diff.y);
        }
    }
}

// ----------------------------- Movement -----------------------------
fn control_player(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut player_transform: Single<&mut Transform, With<Player>>,
) {
    if action_state.axis_pair(&PlayerAction::Move) != Vec2::ZERO {
        // Note: In a real game we'd feed this into an actual player controller
        // and respects the camera extrinsics to ensure the direction is correct
        let move_delta = time.delta_secs() * action_state.clamped_axis_pair(&PlayerAction::Move);
        player_transform.translation += Vec3::new(move_delta.x, 0.0, move_delta.y);
        println!("Player moved to: {}", player_transform.translation.xz());
    }

    if action_state.axis_pair(&PlayerAction::Look) != Vec2::ZERO {
        let look = action_state.axis_pair(&PlayerAction::Look).normalize();
        println!("Player looking in direction: {look}");
    }

    if action_state.pressed(&PlayerAction::Shoot) {
        println!("Shoot!")
    }
}

// ----------------------------- Scene setup -----------------------------
// A player marker
#[derive(Component)]
struct Player;

fn setup_scene(mut commands: Commands) {
    // We need a camera
    commands
        .spawn(Camera3d::default())
        .insert(Transform::from_xyz(0.0, 10.0, 15.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y));

    // And a player
    commands.spawn(Player).insert(Transform::default());
}
