//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

#[cfg(feature = "ui")]
use crate::action_state::ActionStateDriver;
use crate::{
    action_state::{ActionDiff, ActionState},
    buttonlike_user_input::InputStreams,
    clashing_inputs::ClashStrategy,
    input_map::InputMap,
    plugin::DisableInput,
    Actionlike,
};
use bevy_core::Time;
use bevy_ecs::prelude::*;
use bevy_input::{gamepad::GamepadButton, keyboard::KeyCode, mouse::MouseButton, Input};
#[cfg(feature = "ui")]
use bevy_ui::Interaction;

/// Advances actions timer.
///
/// Clears the just-pressed and just-released values of all [`ActionState`]s.
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action.
pub fn tick_action_state<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    time: Res<Time>,
    disable_input: Option<Res<DisableInput<A>>>,
    resource: Option<ResMut<ActionState<A>>>,
) {
    if disable_input.is_some() {
        return;
    }

    if let Some(mut action_state) = resource {
        action_state.tick(
            time.last_update()
                .expect("The `Time` resource has never been updated!"),
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
    clash_strategy: Res<ClashStrategy>,
    mut action_state_resource: Option<ResMut<ActionState<A>>>,
    mut input_map_resource: Option<ResMut<InputMap<A>>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
    disable_input: Option<Res<DisableInput<A>>>,
) {
    if disable_input.is_some() {
        return;
    }

    let gamepad = maybe_gamepad_input_stream.as_deref();

    let keyboard = maybe_keyboard_input_stream.as_deref();

    let mouse = maybe_mouse_input_stream.as_deref();

    if let (Some(input_map), Some(action_state)) =
        (&mut input_map_resource, &mut action_state_resource)
    {
        let input_streams = InputStreams {
            gamepad,
            keyboard,
            mouse,
            associated_gamepad: input_map.gamepad(),
        };

        let pressed_set = input_map.which_pressed(&input_streams, *clash_strategy);
        action_state.update(pressed_set);
    }

    for (mut action_state, input_map) in query.iter_mut() {
        let input_streams = InputStreams {
            gamepad,
            keyboard,
            mouse,
            associated_gamepad: input_map.gamepad(),
        };

        let pressed_set = input_map.which_pressed(&input_streams, *clash_strategy);

        action_state.update(pressed_set);
    }
}

/// When a button with a component `A` is clicked, press the corresponding virtual button in the [`ActionState`]
///
/// The action triggered is determined by the variant stored in your UI-defined button.
#[cfg(feature = "ui")]
pub fn update_action_state_from_interaction<A: Actionlike>(
    ui_query: Query<(&Interaction, &ActionStateDriver<A>)>,
    mut action_state_query: Query<&mut ActionState<A>>,
    disable_input: Option<Res<DisableInput<A>>>,
) {
    if disable_input.is_some() {
        return;
    }

    for (&interaction, action_state_driver) in ui_query.iter() {
        if interaction == Interaction::Clicked {
            let mut action_state = action_state_query
                .get_mut(action_state_driver.entity)
                .expect("Entity does not exist, or does not have an `ActionState` component.");
            action_state.press(action_state_driver.action.clone());
        }
    }
}

/// Generates an [`Events`](bevy_ecs::event::Events) stream of [`ActionDiff`] from [`ActionState`]
///
/// The `ID` generic type should be a stable entity identifer,
/// suitable to be sent across a network.
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
pub fn generate_action_diffs<A: Actionlike, ID: Eq + Clone + Component>(
    action_state_query: Query<(&ActionState<A>, &ID)>,
    mut action_diffs: EventWriter<ActionDiff<A, ID>>,
    disable_input: Option<Res<DisableInput<A>>>,
) {
    if disable_input.is_some() {
        return;
    }

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

/// Generates an [`Events`](bevy_ecs::event::Events) stream of [`ActionDiff`] from [`ActionState`]
///
/// The `ID` generic type should be a stable entity identifer,
/// suitable to be sent across a network.
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
pub fn process_action_diffs<A: Actionlike, ID: Eq + Component + Clone>(
    mut action_state_query: Query<(&mut ActionState<A>, &ID)>,
    mut action_diffs: EventReader<ActionDiff<A, ID>>,
    disable_input: Option<Res<DisableInput<A>>>,
) {
    if disable_input.is_some() {
        return;
    }

    // PERF: This would probably be faster with an index, but is much more fussy
    for action_diff in action_diffs.iter() {
        for (mut action_state, id) in action_state_query.iter_mut() {
            match action_diff {
                ActionDiff::Pressed {
                    action,
                    id: event_id,
                } => {
                    if event_id == id {
                        action_state.press(action.clone());
                        continue;
                    }
                }
                ActionDiff::Released {
                    action,
                    id: event_id,
                } => {
                    if event_id == id {
                        action_state.release(action.clone());
                        continue;
                    }
                }
            };
        }
    }
}

/// Release all inputs if [`DisableInput`] was added
pub fn release_on_disable<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    disable_input: Option<Res<DisableInput<A>>>,
) {
    if let Some(disable_input) = disable_input {
        if disable_input.is_added() {
            for mut action_state in query.iter_mut() {
                action_state.release_all();
            }
        }
    }
}
