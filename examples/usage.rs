use bevy::prelude::*;
use leafwing_input_manager::{InputActionEnum, InputManagerPlugin, InputMap};

use derive_more::Display;
use strum_macros::EnumIter;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<ARPGAction>::default())
        .add_startup_system(initialize_controls)
        .run();
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Display, EnumIter)]
enum ARPGAction {
    // Movement
    Up,
    Down,
    Left,
    Right,
    // Abilities
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Ultimate,
    // Utilities
    AimLock,
    Emote,
    Interact,
}

impl InputActionEnum for ARPGAction {}

fn initialize_controls(
    mut keyboard_map: ResMut<InputMap<ARPGAction, KeyCode>>,
    mut gamepad_map: ResMut<InputMap<ARPGAction, GamepadButton, GamepadButtonType>>,
) {
    // Movement
    keyboard_map.insert(ARPGAction::Up, KeyCode::Up);
    gamepad_map.insert(ARPGAction::Up, GamepadButtonType::DPadUp);
    keyboard_map.insert(ARPGAction::Down, KeyCode::Down);
    gamepad_map.insert(ARPGAction::Down, GamepadButtonType::DPadDown);
    keyboard_map.insert(ARPGAction::Left, KeyCode::Left);
    gamepad_map.insert(ARPGAction::Left, GamepadButtonType::DPadLeft);
    keyboard_map.insert(ARPGAction::Right, KeyCode::Right);
    gamepad_map.insert(ARPGAction::Right, GamepadButtonType::DPadRight);

    // Abilities
    keyboard_map.insert(ARPGAction::Ability1, KeyCode::Q);
    gamepad_map.insert(ARPGAction::Ability1, GamepadButtonType::West);
    keyboard_map.insert(ARPGAction::Ability2, KeyCode::W);
    gamepad_map.insert(ARPGAction::Ability2, GamepadButtonType::North);
    keyboard_map.insert(ARPGAction::Ability3, KeyCode::E);
    gamepad_map.insert(ARPGAction::Ability3, GamepadButtonType::East);
    keyboard_map.insert(ARPGAction::Ability4, KeyCode::Space); // movement ability
    gamepad_map.insert(ARPGAction::Ability4, GamepadButtonType::South);
    keyboard_map.insert(ARPGAction::Ultimate, KeyCode::R);
    gamepad_map.insert(ARPGAction::Ultimate, GamepadButtonType::LeftTrigger2);

    // Utilities
    keyboard_map.insert(ARPGAction::AimLock, KeyCode::Grave);
    gamepad_map.insert(ARPGAction::AimLock, GamepadButtonType::RightTrigger);
    keyboard_map.insert(ARPGAction::Emote, KeyCode::LShift);
    gamepad_map.insert(ARPGAction::Emote, GamepadButtonType::LeftTrigger);
    keyboard_map.insert(ARPGAction::Interact, KeyCode::F);
    gamepad_map.insert(ARPGAction::Interact, GamepadButtonType::RightTrigger2);
}
