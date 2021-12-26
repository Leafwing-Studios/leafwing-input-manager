//! The systems that power each [InputManagerPlugin](crate::InputManagerPlugin).

use crate::{ActionState, Actionlike, InputMap};
use bevy::prelude::*;

/// Clears the just-pressed and just-released values of all [ActionState]s
///
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action.
/// Should run before [update_action_state].
pub fn tick_action_state<A: Actionlike>(mut query: Query<&mut ActionState<A>>) {
    for mut action_state in query.iter_mut() {
        action_state.tick();
    }
}

/// Fetches all of the releveant [Input] resources to update [ActionState] according to the [InputMap]
/// /// Should run after [tick_action_state] and before [release_action_state].
pub fn update_action_state<A: Actionlike>(
    gamepad_input: Res<Input<GamepadButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
) {
    for (mut action_state, input_map) in query.iter_mut() {
        for action in A::iter() {
            // A particular input type can add to the action state, but cannot revert it
            if input_map.pressed(action, &*gamepad_input, &*keyboard_input, &*mouse_input) {
                action_state.press(action);
            }
        }
    }
}

/// Releases all [ActionState] actions that were not pressed since the last time [tick_action_state] ran
///
/// Should run after [update_action_state].
pub fn release_action_state<A: Actionlike>(mut query: Query<&mut ActionState<A>>) {
    for mut action_state in query.iter_mut() {
        action_state.release_unpressed();
    }
}
