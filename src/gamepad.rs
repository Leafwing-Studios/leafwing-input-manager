use crate::{ActionState, InputActionEnum, InputMap};
use bevy::prelude::*;
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut, Component, Default)]
pub struct AssociatedGamepad {
    pub gamepad: Option<Gamepad>,
}

/// Special-cased version of [update_action_state](crate::systems::update_action_state) for Gamepads
///
/// Each input-mapped entity should have its own [AssociatedGamepad] component,
/// which controls which [Gamepad] it is controlled by.
///
/// You should work with the [Gamepads] resource to ensure that this is configured properly according to your game logic.
pub fn update_action_state_gamepads<InputAction: InputActionEnum>(
    gamepad_map: Res<InputMap<InputAction, GamepadButton, GamepadButtonType>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut query: Query<(&mut ActionState<InputAction>, &AssociatedGamepad)>,
) {
    for (mut action_state, associated_gamepad) in query.iter_mut() {
        if let Some(gamepad) = associated_gamepad.gamepad {
            for action in InputAction::iter() {
                if gamepad_map.pressed(action, &*gamepad_input, gamepad) {
                    action_state.press(action);
                }
            }
        }
    }
}

/// A simple alternative to [update_action_state_gamepads] that accepts input from any [Gamepad]
///
/// Rather than reading the value of [AssociatedGamepad], input from all gamepads is mapped into the [ActionState]
/// This system is well-suited to single player games, where input from any acceptable [Gamepad]
/// should be used to control the main player.
pub fn update_action_state_any_gamepad<InputAction: InputActionEnum>(
    gamepad_map: Res<InputMap<InputAction, GamepadButton, GamepadButtonType>>,
    gamepads: Res<Gamepads>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut query: Query<&mut ActionState<InputAction>>,
) {
    for mut action_state in query.iter_mut() {
        for &gamepad in gamepads.iter() {
            for action in InputAction::iter() {
                if gamepad_map.pressed(action, &*gamepad_input, gamepad) {
                    action_state.press(action);
                }
            }
        }
    }
}
