use crate::{input_map::Inputlike, ActionState, InputActionEnum, InputMap};
use bevy::prelude::*;

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
pub fn update_action_state<InputAction: InputActionEnum, InputType: Inputlike>(
    input: Res<Input<InputType>>,
    mut query: Query<(&mut ActionState<InputAction>, &InputMap<InputAction>)>,
) {
    for (mut action_state, input_map) in query.iter_mut() {
        for action in InputAction::iter() {
            // A particular input type can add to the action state, but cannot revert it
            if input_map.pressed_by(action, &*input) {
                action_state.press(action);
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
