//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

#[cfg(feature = "ui")]
use crate::action_state::ActionStateDriver;
use crate::{
    action_state::{ActionDiff, ActionState},
    clashing_inputs::ClashStrategy,
    input_map::InputMap,
    input_streams::InputStreams,
    plugin::ToggleActions,
    Actionlike,
};

use bevy::ecs::{prelude::*, schedule::ShouldRun};
use bevy::input::{
    gamepad::{GamepadAxis, GamepadButton, Gamepads},
    keyboard::KeyCode,
    mouse::{MouseButton, MouseMotion, MouseWheel},
    Axis, Input,
};
use bevy::time::Time;
use bevy::utils::Instant;

#[cfg(feature = "ui")]
use bevy::ui::Interaction;

/// Advances actions timer.
///
/// Clears the just-pressed and just-released values of all [`ActionState`]s.
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action.
pub fn tick_action_state<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    action_state: Option<ResMut<ActionState<A>>>,
    time: Res<Time>,
    mut stored_previous_instant: Local<Option<Instant>>,
) {
    // If this is the very first tick, measure from the start of the app
    let current_instant = time.last_update().unwrap_or_else(|| time.startup());
    let previous_instant = stored_previous_instant.unwrap_or_else(|| time.startup());

    // Only tick the ActionState resource if it exists
    if let Some(mut action_state) = action_state {
        action_state.tick(current_instant, previous_instant);
    }

    // Only tick the ActionState components if they exist
    for mut action_state in query.iter_mut() {
        // If `Time` has not ever been advanced, something has gone horribly wrong
        // and the user probably forgot to add the `core_plugin`.
        action_state.tick(current_instant, previous_instant);
    }

    // Store the previous time in the system
    *stored_previous_instant = time.last_update();
}

/// Fetches all of the releveant [`Input`] resources to update [`ActionState`] according to the [`InputMap`]
///
/// Missing resources will be ignored, and treated as if none of the corresponding inputs were pressed
#[allow(clippy::too_many_arguments)]
pub fn update_action_state<A: Actionlike>(
    maybe_gamepad_button_input_stream: Option<Res<Input<GamepadButton>>>,
    maybe_gamepad_button_axes_input_stream: Option<Res<Axis<GamepadButton>>>,
    maybe_gamepad_axes_input_stream: Option<Res<Axis<GamepadAxis>>>,
    maybe_gamepads: Option<Res<Gamepads>>,
    maybe_keyboard_input_stream: Option<Res<Input<KeyCode>>>,
    maybe_mouse_input_stream: Option<Res<Input<MouseButton>>>,
    maybe_mouse_wheel_events: Option<Res<Events<MouseWheel>>>,
    maybe_mouse_motion_events: Option<Res<Events<MouseMotion>>>,
    clash_strategy: Res<ClashStrategy>,
    mut action_state: Option<ResMut<ActionState<A>>>,
    mut input_map: Option<ResMut<InputMap<A>>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
) {
    let gamepad_buttons = maybe_gamepad_button_input_stream.as_deref();
    let gamepad_button_axes = maybe_gamepad_button_axes_input_stream.as_deref();
    let gamepad_axes = maybe_gamepad_axes_input_stream.as_deref();
    let gamepads = maybe_gamepads.as_deref();

    let keyboard = maybe_keyboard_input_stream.as_deref();

    let mouse = maybe_mouse_input_stream.as_deref();
    let mouse_wheel = maybe_mouse_wheel_events.as_deref();
    let mouse_motion = maybe_mouse_motion_events.as_deref();

    if let (Some(input_map), Some(action_state)) = (&mut input_map, &mut action_state) {
        let input_streams = InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keyboard,
            mouse,
            mouse_wheel,
            mouse_motion,
            associated_gamepad: input_map.gamepad(),
        };

        action_state.update(input_map.which_pressed(&input_streams, *clash_strategy));
    }

    for (mut action_state, input_map) in query.iter_mut() {
        let input_streams = InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keyboard,
            mouse,
            mouse_wheel,
            mouse_motion,
            associated_gamepad: input_map.gamepad(),
        };

        action_state.update(input_map.which_pressed(&input_streams, *clash_strategy));
    }
}

/// When a button with a component of type `A` is clicked, press the corresponding action in the [`ActionState`]
///
/// The action triggered is determined by the variant stored in your UI-defined button.
#[cfg(feature = "ui")]
pub fn update_action_state_from_interaction<A: Actionlike>(
    ui_query: Query<(&Interaction, &ActionStateDriver<A>)>,
    mut action_state_query: Query<&mut ActionState<A>>,
) {
    for (&interaction, action_state_driver) in ui_query.iter() {
        if interaction == Interaction::Clicked {
            let mut action_state = action_state_query
                .get_mut(action_state_driver.entity)
                .expect("Entity does not exist, or does not have an `ActionState` component.");
            action_state.press(action_state_driver.action.clone());
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

/// Release all inputs if the [`ToggleActions<A>`] resource exists and its `enabled` field is false.
pub fn release_on_disable<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    resource: Option<ResMut<ActionState<A>>>,
    toggle_actions: Res<ToggleActions<A>>,
) {
    if toggle_actions.is_changed() && !toggle_actions.enabled {
        for mut action_state in query.iter_mut() {
            action_state.release_all();
        }
        if let Some(mut action_state) = resource {
            action_state.release_all();
        }
    }
}

/// Returns [`ShouldRun::No`] if [`DisableInput`] exists and [`ShouldRun::Yes`] otherwise
pub(super) fn run_if_enabled<A: Actionlike>(toggle_actions: Res<ToggleActions<A>>) -> ShouldRun {
    if toggle_actions.enabled {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
