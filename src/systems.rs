//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

#[cfg(feature = "ui")]
use crate::action_driver::ActionStateDriver;
use crate::{
    action_state::{ActionDiff, ActionState},
    clashing_inputs::ClashStrategy,
    input_map::InputMap,
    input_streams::InputStreams,
    plugin::ToggleActions,
    Actionlike,
};

use bevy::{ecs::prelude::*, prelude::ScanCode};
use bevy::{
    input::{
        gamepad::{GamepadAxis, GamepadButton, Gamepads},
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
        Axis, Input,
    },
    log::warn,
    math::Vec2,
    time::{Real, Time},
    utils::{HashMap, Instant},
};

use crate::action_state::ActionDiffEvent;
#[cfg(feature = "ui")]
use bevy::ui::Interaction;
#[cfg(feature = "egui")]
use bevy_egui::EguiContext;

/// Advances actions timer.
///
/// Clears the just-pressed and just-released values of all [`ActionState`]s.
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action.
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

/// Fetches all of the relevant [`Input`] resources to update [`ActionState`] according to the [`InputMap`].
///
/// Missing resources will be ignored, and treated as if none of the corresponding inputs were pressed.
#[allow(clippy::too_many_arguments)]
pub fn update_action_state<A: Actionlike>(
    gamepad_buttons: Res<Input<GamepadButton>>,
    gamepad_button_axes: Res<Axis<GamepadButton>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepads: Res<Gamepads>,
    keycodes: Option<Res<Input<KeyCode>>>,
    scan_codes: Option<Res<Input<ScanCode>>>,
    mouse_buttons: Option<Res<Input<MouseButton>>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    clash_strategy: Res<ClashStrategy>,
    #[cfg(all(feature = "ui", feature = "block_ui_interactions"))] interactions: Query<
        &Interaction,
    >,
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
    let scan_codes = scan_codes.map(|scan_codes| scan_codes.into_inner());
    let mouse_buttons = mouse_buttons.map(|mouse_buttons| mouse_buttons.into_inner());

    let mouse_wheel: Option<Vec<MouseWheel>> = Some(mouse_wheel.read().cloned().collect());
    let mouse_motion: Vec<MouseMotion> = mouse_motion.read().cloned().collect();

    // If use clicks on a button, do not apply them to the game state
    #[cfg(all(feature = "ui", feature = "block_ui_interactions"))]
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
    let (keycodes, scan_codes) = if maybe_egui
        .iter_mut()
        .any(|(_, mut ctx)| ctx.get_mut().wants_keyboard_input())
    {
        (None, None)
    } else {
        (keycodes, scan_codes)
    };

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
            scan_codes,
            mouse_buttons,
            mouse_wheel: mouse_wheel.clone(),
            mouse_motion: mouse_motion.clone(),
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
        if interaction == Interaction::Pressed {
            for entity in action_state_driver.targets.iter() {
                let mut action_state = action_state_query
                    .get_mut(*entity)
                    .expect("Entity does not exist, or does not have an `ActionState` component.");
                action_state.press(&action_state_driver.action.clone());
            }
        }
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

            match action_data.axis_pair {
                Some(axis_pair) => {
                    diffs.push(ActionDiff::AxisPairChanged {
                        action: action.clone(),
                        axis_pair: axis_pair.into(),
                    });
                    previous_axis_pairs
                        .raw_entry_mut()
                        .from_key(&action)
                        .or_insert_with(|| (action.clone(), HashMap::default()))
                        .1
                        .insert(maybe_entity, axis_pair.xy());
                }
                None => {
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
                        .raw_entry_mut()
                        .from_key(&action)
                        .or_insert_with(|| (action.clone(), HashMap::default()))
                        .1
                        .insert(maybe_entity, value);
                }
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

            match action_data.axis_pair {
                Some(axis_pair) => {
                    let previous_axis_pairs = previous_axis_pairs.get_mut(&action).unwrap();

                    if let Some(previous_axis_pair) = previous_axis_pairs.get(&maybe_entity) {
                        if *previous_axis_pair == axis_pair.xy() {
                            continue;
                        }
                    }
                    diffs.push(ActionDiff::AxisPairChanged {
                        action: action.clone(),
                        axis_pair: axis_pair.into(),
                    });
                    previous_axis_pairs.insert(maybe_entity, axis_pair.xy());
                }
                None => {
                    let value = action_data.value;
                    let previous_values = previous_values.get_mut(&action).unwrap();

                    if let Some(previous_value) = previous_values.get(&maybe_entity) {
                        if *previous_value == value {
                            continue;
                        }
                    }
                    diffs.push(ActionDiff::ValueChanged {
                        action: action.clone(),
                        value,
                    });
                    previous_values.insert(maybe_entity, value);
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
        // Store if the resource existed so we know if it was removed later.
        *input_map_resource_existed = true;
    } else if *input_map_resource_existed {
        // The input map does not exist and our local is true so we know the input map was removed.

        if let Some(mut action_state) = action_state_resource {
            action_state.release_all();
        }

        // Reset our local so our removal detection is only triggered once.
        *input_map_resource_existed = false;
    }
}

/// Uses the value of [`ToggleActions<A>`] to determine if input manager systems of type `A` should run.
pub fn run_if_enabled<A: Actionlike>(toggle_actions: Res<ToggleActions<A>>) -> bool {
    toggle_actions.enabled
}
