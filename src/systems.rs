//! The systems that power each [InputManagerPlugin](crate::InputManagerPlugin).

use crate::{
    action_state::{ActionState, ActionStateDriver},
    input_map::InputMap,
    Actionlike,
};
use bevy::prelude::*;

/// Clears the just-pressed and just-released values of all [ActionState]s
///
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action.
pub fn tick_action_state<A: Actionlike>(mut query: Query<&mut ActionState<A>>) {
    for mut action_state in query.iter_mut() {
        action_state.tick();
    }
}

/// Fetches all of the releveant [Input] resources to update [ActionState] according to the [InputMap]
pub fn update_action_state<A: Actionlike>(
    gamepad_input_stream: Res<Input<GamepadButton>>,
    keyboard_input_stream: Res<Input<KeyCode>>,
    mouse_input_stream: Res<Input<MouseButton>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
) {
    for (mut action_state, input_map) in query.iter_mut() {
        action_state.update(
            input_map,
            &*gamepad_input_stream,
            &*keyboard_input_stream,
            &*mouse_input_stream,
        );
    }
}

/// When a button with a component `A` is clicked, press the corresponding virtual button in the [ActionState]
///
/// The action triggered is determined by the variant stored in your UI-defined button.
pub fn update_action_state_from_interaction<A: Actionlike>(
    ui_query: Query<(&Interaction, &ActionStateDriver<A>)>,
    mut action_state_query: Query<&mut ActionState<A>>,
) {
    for (&interaction, action_state_driver) in ui_query.iter() {
        if interaction == Interaction::Clicked {
            let mut action_state = action_state_query
                .get_mut(action_state_driver.entity)
                .expect("Entity does not exist, or does not have an `ActionState` component.");
            action_state.press(action_state_driver.action);
        }
    }
}
