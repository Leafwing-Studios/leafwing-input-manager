//! This is a fairly complete example that implements a twin stick controller.
//!
//! The controller supports both gamepad/MKB input and switches between them depending on
//! the most recent input.
//!
//! This example builds on top of several of the concepts introduced in other examples. In particular,
//! the `default_controls`. `mouse_position`, and `action_state_resource` examples.

use bevy::{
    input::gamepad::GamepadEvent, input::keyboard::KeyboardInput, prelude::*, window::PrimaryWindow,
};
use leafwing_input_manager::{axislike::DualAxisData, prelude::*, user_input::InputKind};

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
pub enum PlayerAction {
    Move,
    Look,
    Shoot,
}

// Exhaustively match `PlayerAction` and define the default binding to the input
impl PlayerAction {
    fn default_gamepad_binding(&self) -> UserInput {
        // Match against the provided action to get the correct default gamepad input
        match self {
            Self::Move => UserInput::Single(InputKind::DualAxis(DualAxis::left_stick())),
            Self::Look => UserInput::Single(InputKind::DualAxis(DualAxis::right_stick())),
            Self::Shoot => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::RightTrigger))
            }
        }
    }

    fn default_mkb_binding(&self) -> UserInput {
        // Match against the provided action to get the correct default gamepad input
        match self {
            Self::Move => UserInput::VirtualDPad(VirtualDPad::wasd()),
            Self::Look => UserInput::VirtualDPad(VirtualDPad::arrow_keys()),
            Self::Shoot => UserInput::Single(InputKind::Mouse(MouseButton::Left)),
        }
    }

    fn default_input_map() -> InputMap<PlayerAction> {
        let mut input_map = InputMap::default();

        for variant in PlayerAction::variants() {
            input_map.insert(variant.default_mkb_binding(), variant.clone());
            input_map.insert(variant.default_gamepad_binding(), variant);
        }
        input_map
    }
}

// ----------------------------- Input mode handling -----------------------------
pub struct InputModeManagerPlugin;

impl Plugin for InputModeManagerPlugin {
    fn build(&self, app: &mut App) {
        // Add a state to record the current active input
        app.add_state::<ActiveInput>()
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

/// Note that we handle the action_state mutation differently here than in the `mouse_position`
/// example. Here we don't use an `ActionDriver`, but change the action data directly.
fn player_mouse_look(
    camera_query: Query<(&GlobalTransform, &Camera)>,
    player_query: Query<&Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut action_state: ResMut<ActionState<PlayerAction>>,
) {
    // Update each actionstate with the mouse position from the window
    // by using the referenced entities in ActionStateDriver and the stored action as
    // a key into the action data
    let (camera_transform, camera) = camera_query.get_single().expect("Need a single camera");
    let player_transform = player_query.get_single().expect("Need a single player");
    let window = window_query
        .get_single()
        .expect("Need a single primary window");

    // Many steps can fail here, so we'll wrap in an option pipeline
    // First check if cursor is in window
    // Then check if the ray intersects the plane defined by the player
    // Then finally compute the point along the ray to look at
    if let Some(p) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .and_then(|ray| Some(ray).zip(ray.intersect_plane(player_transform.translation, Vec3::Y)))
        .map(|(ray, p)| ray.get_point(p))
    {
        let diff = (p - player_transform.translation).xz();
        if diff.length_squared() > 1e-3f32 {
            // Press the look action, so we can check that it is active
            action_state.press(PlayerAction::Look);
            // Modify the action data to set the axis
            let action_data = action_state.action_data_mut(PlayerAction::Look);
            // Flipping y sign here to be consistent with gamepad input. We could also invert the gamepad y axis
            action_data.axis_pair = Some(DualAxisData::from_xy(Vec2::new(diff.x, -diff.y)));
        }
    }
}

// ----------------------------- Movement -----------------------------
fn control_player(
    time: Res<Time>,
    action_state: Res<ActionState<PlayerAction>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    if action_state.pressed(PlayerAction::Move) {
        // Note: In a real game we'd feed this into an actual player controller
        // and respects the camera extrinsics to ensure the direction is correct
        let move_delta = time.delta_seconds()
            * action_state
                .clamped_axis_pair(PlayerAction::Move)
                .unwrap()
                .xy();
        player_transform.translation += Vec3::new(move_delta.x, 0.0, move_delta.y);
        println!("Player moved to: {}", player_transform.translation.xz());
    }

    if action_state.pressed(PlayerAction::Look) {
        let look = action_state
            .axis_pair(PlayerAction::Look)
            .unwrap()
            .xy()
            .normalize();
        println!("Player looking in direction: {}", look);
    }

    if action_state.pressed(PlayerAction::Shoot) {
        println!("Shoot!")
    }
}

// ----------------------------- Scene setup -----------------------------
// A player marker
#[derive(Component)]
struct Player;

fn setup_scene(mut commands: Commands) {
    // We need a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 15.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    // And a player
    commands.spawn(Player).insert(Transform::default());

    // But note that there is no visibility in this example
}
