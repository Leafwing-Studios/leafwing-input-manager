//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

use crate::{
    action_state::ActionState, clashing_inputs::ClashStrategy, input_map::InputMap,
    input_streams::InputStreams, Actionlike,
};

use bevy::ecs::prelude::*;
use bevy::{
    input::{
        gamepad::{GamepadAxis, GamepadButton, Gamepads},
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
        Axis, ButtonInput,
    },
    log::warn,
    math::Vec2,
    time::{Real, Time},
    utils::{HashMap, Instant},
};

use crate::action_diff::{ActionDiff, ActionDiffEvent};

#[cfg(feature = "ui")]
use bevy::ui::Interaction;
#[cfg(feature = "egui")]
use bevy_egui::EguiContext;

/// We are about to enter the `Main` schedule, so we:
/// - save all the changes applied to `state` into the `fixed_update_state`
/// - switch to loading the `update_state`
pub fn swap_to_update<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    action_state: Option<ResMut<ActionState<A>>>,
) {
    if let Some(mut action_state) = action_state {
        action_state.swap_to_update_state();
    }

    for mut action_state in query.iter_mut() {
        action_state.swap_to_update_state();
    }
}

/// We are about to enter the `FixedMain` schedule, so we:
/// - save all the changes applied to `state` into the `update_state`
/// - switch to loading the `fixed_update_state`
pub fn swap_to_fixed_update<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    action_state: Option<ResMut<ActionState<A>>>,
) {
    if let Some(mut action_state) = action_state {
        action_state.swap_to_fixed_update_state();
    }

    for mut action_state in query.iter_mut() {
        action_state.swap_to_fixed_update_state();
    }
}

/// Advances actions timer.
///
/// Clears the just-pressed and just-released values of all [`ActionState`]s.
/// Also resets the internal `pressed_this_tick` field, used to track whether to release an action.
pub fn tick_action_state<A: Actionlike>(
    mut query: Query<&mut ActionState<A>>,
    action_state: Option<ResMut<ActionState<A>>>,
    time: Res<Time<Real>>,
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

/// Fetches all the relevant [`ButtonInput`] resources
/// to update [`ActionState`] according to the [`InputMap`].
///
/// Missing resources will be ignored and treated as if none of the corresponding inputs were pressed.
#[allow(clippy::too_many_arguments)]
pub fn update_action_state<A: Actionlike>(
    gamepad_buttons: Res<ButtonInput<GamepadButton>>,
    gamepad_button_axes: Res<Axis<GamepadButton>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepads: Res<Gamepads>,
    keycodes: Option<Res<ButtonInput<KeyCode>>>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    clash_strategy: Res<ClashStrategy>,
    #[cfg(all(feature = "ui", not(feature = "no_ui_priority")))] interactions: Query<&Interaction>,
    #[cfg(feature = "egui")] mut maybe_egui: Query<(Entity, &'static mut EguiContext)>,
    action_state: Option<ResMut<ActionState<A>>>,
    input_map: Option<Res<InputMap<A>>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
) {
    let gamepad_buttons = gamepad_buttons.into_inner();
    let gamepad_button_axes = gamepad_button_axes.into_inner();
    let gamepad_axes = gamepad_axes.into_inner();
    let gamepads = gamepads.into_inner();
    let keycodes = keycodes.map(|keycodes| keycodes.into_inner());
    let mouse_buttons = mouse_buttons.map(|mouse_buttons| mouse_buttons.into_inner());

    let mouse_wheel: Option<Vec<MouseWheel>> = Some(mouse_wheel.read().cloned().collect());
    let mouse_motion: Vec<MouseMotion> = mouse_motion.read().cloned().collect();

    // If the user clicks on a button, do not apply it to the game state
    #[cfg(all(feature = "ui", not(feature = "no_ui_priority")))]
    let (mouse_buttons, mouse_wheel) = if interactions
        .iter()
        .any(|&interaction| interaction != Interaction::None)
    {
        (None, None)
    } else {
        (mouse_buttons, mouse_wheel)
    };

    // If egui wants to own inputs, don't also apply them to the game state
    #[cfg(feature = "egui")]
    let keycodes = maybe_egui
        .iter_mut()
        .all(|(_, mut ctx)| !ctx.get_mut().wants_keyboard_input())
        .then_some(keycodes)
        .flatten();

    // `wants_pointer_input` sometimes returns `false` after clicking or holding a button over a widget,
    // so `is_pointer_over_area` is also needed.
    #[cfg(feature = "egui")]
    let (mouse_buttons, mouse_wheel) = if maybe_egui.iter_mut().any(|(_, mut ctx)| {
        ctx.get_mut().is_pointer_over_area() || ctx.get_mut().wants_pointer_input()
    }) {
        (None, None)
    } else {
        (mouse_buttons, mouse_wheel)
    };

    let resources = input_map
        .zip(action_state)
        .map(|(input_map, action_state)| (Mut::from(action_state), input_map.into_inner()));

    for (mut action_state, input_map) in query.iter_mut().chain(resources) {
        let input_streams = InputStreams {
            gamepad_buttons,
            gamepad_button_axes,
            gamepad_axes,
            gamepads,
            keycodes,
            mouse_buttons,
            mouse_wheel: mouse_wheel.clone(),
            mouse_motion: mouse_motion.clone(),
            associated_gamepad: input_map.gamepad(),
        };

        action_state.update(input_map.process_actions(&input_streams, *clash_strategy));
    }
}

/// Generates an [`Events`] stream of [`ActionDiff`] from [`ActionState`]
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
pub fn generate_action_diffs<A: Actionlike>(
    action_state: Option<ResMut<ActionState<A>>>,
    action_state_query: Query<(Entity, &ActionState<A>)>,
    mut action_diffs: EventWriter<ActionDiffEvent<A>>,
    mut previous_values: Local<HashMap<A, HashMap<Option<Entity>, f32>>>,
    mut previous_axis_pairs: Local<HashMap<A, HashMap<Option<Entity>, Vec2>>>,
) {
    // we use None to represent the global ActionState
    let action_state_iter = action_state_query
        .iter()
        .map(|(entity, action_state)| (Some(entity), action_state))
        .chain(
            action_state
                .as_ref()
                .map(|action_state| (None, action_state.as_ref())),
        );
    for (maybe_entity, action_state) in action_state_iter {
        let mut diffs = vec![];
        for action in action_state.get_just_pressed() {
            let Some(action_data) = action_state.action_data(&action) else {
                warn!("Action in ActionDiff has no data: was it generated correctly?");
                continue;
            };

            if let Some(axis_pair) = action_data.axis_pair {
                diffs.push(ActionDiff::AxisPairChanged {
                    action: action.clone(),
                    axis_pair: axis_pair.into(),
                });
                previous_axis_pairs
                    .entry(action)
                    .or_default()
                    .insert(maybe_entity, axis_pair.xy());
            } else {
                let value = action_data.value;

                diffs.push(if value == 1. {
                    ActionDiff::Pressed {
                        action: action.clone(),
                    }
                } else {
                    ActionDiff::ValueChanged {
                        action: action.clone(),
                        value,
                    }
                });
                previous_values
                    .entry(action)
                    .or_default()
                    .insert(maybe_entity, value);
            }
        }
        for action in action_state.get_pressed() {
            if action_state.just_pressed(&action) {
                continue;
            }

            let Some(action_data) = action_state.action_data(&action) else {
                warn!("Action in ActionState has no data: was it generated correctly?");
                continue;
            };

            if let Some(axis_pair) = action_data.axis_pair {
                let current_value = axis_pair.xy();
                let values = previous_axis_pairs.get_mut(&action).unwrap();

                let existing_value = values.get(&maybe_entity);
                if !matches!(existing_value, Some(value) if *value == current_value) {
                    diffs.push(ActionDiff::AxisPairChanged {
                        action: action.clone(),
                        axis_pair: axis_pair.into(),
                    });
                    values.insert(maybe_entity, current_value);
                }
            } else {
                let current_value = action_data.value;
                let values = previous_values.get_mut(&action).unwrap();

                if !matches!(values.get(&maybe_entity), Some(value) if *value == current_value) {
                    diffs.push(ActionDiff::ValueChanged {
                        action: action.clone(),
                        value: current_value,
                    });
                    values.insert(maybe_entity, current_value);
                }
            }
        }
        for action in action_state.get_just_released() {
            diffs.push(ActionDiff::Released {
                action: action.clone(),
            });
            if let Some(previous_axes) = previous_axis_pairs.get_mut(&action) {
                previous_axes.remove(&maybe_entity);
            }
            if let Some(previous_values) = previous_values.get_mut(&action) {
                previous_values.remove(&maybe_entity);
            }
        }
        if !diffs.is_empty() {
            action_diffs.send(ActionDiffEvent {
                owner: maybe_entity,
                action_diffs: diffs,
            });
        }
    }
}

/// Release all inputs when an [`InputMap<A>`] is removed to prevent them from being held forever.
///
/// By default, [`InputManagerPlugin<A>`](crate::plugin::InputManagerPlugin) will run this on [`PostUpdate`](bevy::prelude::PostUpdate).
/// For components you must remove the [`InputMap<A>`] before [`PostUpdate`](bevy::prelude::PostUpdate)
/// or this will not run.
pub fn release_on_input_map_removed<A: Actionlike>(
    mut removed_components: RemovedComponents<InputMap<A>>,
    input_map_resource: Option<ResMut<InputMap<A>>>,
    action_state_resource: Option<ResMut<ActionState<A>>>,
    mut input_map_resource_existed: Local<bool>,
    mut action_state_query: Query<&mut ActionState<A>>,
) {
    let mut iter = action_state_query.iter_many_mut(removed_components.read());
    while let Some(mut action_state) = iter.fetch_next() {
        action_state.release_all();
    }

    // Detect when an InputMap resource is removed.
    if input_map_resource.is_some() {
        // Store if the resource existed, so we know if it was removed later.
        *input_map_resource_existed = true;
    } else if *input_map_resource_existed {
        // The input map does not exist, and our local is true,
        // so we know the input map was removed.

        if let Some(mut action_state) = action_state_resource {
            action_state.release_all();
        }

        // Reset our local so our removal detection is only triggered once.
        *input_map_resource_existed = false;
    }
}
