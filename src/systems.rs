//! The systems that power each [`InputManagerPlugin`](crate::InputManagerPlugin).

use crate::{action_state::{ActionDiff, ActionState, ActionStateDriver}, input_map::InputMap, user_input::InputStreams, Actionlike, InputResource};
use bevy::prelude::*;

/// Clears the just-pressed and just-released values of all [`ActionState`]s
///
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action.
pub fn tick_action_state<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    resource: Option<ResMut<InputResource<A>>>,
    time: Res<Time>,
) {
    if let Some(mut input_resource) = resource {
        input_resource.action_state.tick(
            time.last_update().expect("The `Time` resource has never been updated!")
        )
    }
    for mut action_state in query.iter_mut() {
        // If `Time` has not ever been advanced, something has gone horribly wrong
        // and the user probably forgot to add the `core_plugin`.
        action_state.tick(
            time.last_update()
                .expect("The `Time` resource has never been updated!"),
        );
    }
}

/// Fetches all of the releveant [`Input`] resources to update [`ActionState`] according to the [`InputMap`]
///
/// Missing resources will be ignored, and treated as if none of the corresponding inputs were pressed
pub fn update_action_state<A: Actionlike>(
    maybe_gamepad_input_stream: Option<Res<Input<GamepadButton>>>,
    maybe_keyboard_input_stream: Option<Res<Input<KeyCode>>>,
    maybe_mouse_input_stream: Option<Res<Input<MouseButton>>>,
    mut resource: Option<ResMut<InputResource<A>>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
) {
    let gamepad = maybe_gamepad_input_stream.as_deref();

    let keyboard = maybe_keyboard_input_stream.as_deref();

    let mouse = maybe_mouse_input_stream.as_deref();

    if let Some(res) = &mut resource {
        let input_streams = InputStreams {
            gamepad,
            keyboard,
            mouse,
            associated_gamepad: res.input_map.gamepad(),
        };

        let pressed_set = res.input_map.which_pressed(&input_streams);

        res.action_state.update(pressed_set);
    }

    for (mut action_state, input_map) in query.iter_mut() {
        let input_streams = InputStreams {
            gamepad,
            keyboard,
            mouse,
            associated_gamepad: input_map.gamepad(),
        };

        let pressed_set = input_map.which_pressed(&input_streams);

        action_state.update(pressed_set);
    }
}

/// When a button with a component `A` is clicked, press the corresponding virtual button in the [`ActionState`]
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
            action_state.press(&action_state_driver.action);
        }
    }
}

/// Generates an [`Events`](bevy::ecs::event::Events) stream of [`ActionDiff`] from [`ActionState`]
///
/// The `ID` generic type should be a stable entity identifer,
/// suitable to be sent across a network.
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
pub fn generate_action_diffs<A: Actionlike, ID: Eq + Clone + Component>(
    action_state_query: Query<(&ActionState<A>, &ID)>,
    mut action_diffs: EventWriter<ActionDiff<A, ID>>,
) {
    for (action_state, id) in action_state_query.iter() {
        for action in action_state.get_just_pressed() {
            action_diffs.send(ActionDiff::Pressed {
                action: action.clone(),
                id: id.clone(),
            });
        }

        for action in action_state.get_just_released() {
            action_diffs.send(ActionDiff::Released {
                action: action.clone(),
                id: id.clone(),
            });
        }
    }
}

/// Generates an [`Events`](bevy::ecs::event::Events) stream of [`ActionDiff`] from [`ActionState`]
///
/// The `ID` generic type should be a stable entity identifer,
/// suitable to be sent across a network.
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
pub fn process_action_diffs<A: Actionlike, ID: Eq + Component + Clone>(
    mut action_state_query: Query<(&mut ActionState<A>, &ID)>,
    mut action_diffs: EventReader<ActionDiff<A, ID>>,
) {
    // PERF: This would probably be faster with an index, but is much more fussy
    for action_diff in action_diffs.iter() {
        for (mut action_state, id) in action_state_query.iter_mut() {
            match action_diff {
                ActionDiff::Pressed {
                    action,
                    id: event_id,
                } => {
                    if event_id == id {
                        action_state.press(action);
                        continue;
                    }
                }
                ActionDiff::Released {
                    action,
                    id: event_id,
                } => {
                    if event_id == id {
                        action_state.release(action);
                        continue;
                    }
                }
            };
        }
    }
}
