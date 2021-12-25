use crate::{ActionState, InputActionEnum, InputMap};
use bevy::prelude::*;
use core::hash::Hash;

/// Clears the just-pressed and just-released values of all [ActionState]s
///
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action
pub fn tick_action_state<InputAction: InputActionEnum>(
    mut query: Query<&mut ActionState<InputAction>>,
) {
    for mut action_state in query.iter_mut() {
        action_state.tick();
    }
}

/// Fetches an [Input] resource to update [ActionState] according to the [InputMap]
pub fn update_action_state<
    InputAction: InputActionEnum,
    InputType: Send + Sync + Copy + Hash + Eq + 'static,
>(
    input: Res<Input<InputType>>,
    input_map: Res<InputMap<InputAction, InputType>>,
    mut query: Query<&mut ActionState<InputAction>>,
) {
    for mut action_state in query.iter_mut() {
        for action in InputAction::iter() {
            // A particular input type can add to the action state, but cannot revert it
            if input_map.pressed(action, &*input) {
                action_state.press(action);
            }
        }
    }
}

/// Special-cased version of [update_action_state] for Gamepads
///
/// This system is intended for single-player games;
/// all gamepads are mapped to a single [ActionState].
/// You will want to modify this system if you want to handle multiple players correctly
pub fn update_action_state_gamepads<InputAction: InputActionEnum>(
    gamepads: Res<Gamepads>,
    gamepad_map: Res<InputMap<InputAction, GamepadButton, GamepadButtonType>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut query: Query<&mut ActionState<InputAction>>,
) {
    for mut action_state in query.iter_mut() {
        for action in InputAction::iter() {
            for &gamepad in gamepads.iter() {
                if gamepad_map.pressed(action, &*gamepad_input, gamepad) {
                    action_state.press(action);
                }
            }
        }
    }
}

/// Releases all [ActionState] actions that were not pressed since the last time [tick_action_state] ran
pub fn release_action_state<InputAction: InputActionEnum>(
    mut query: Query<&mut ActionState<InputAction>>,
) {
    for mut action_state in query.iter_mut() {
        action_state.release_unpressed();
    }
}
