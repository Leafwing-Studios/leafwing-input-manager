//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

use crate::user_input::{AccumulatedMouseMovement, AccumulatedMouseScroll};
use crate::{
    action_state::ActionState, clashing_inputs::ClashStrategy, input_map::InputMap,
    input_streams::InputStreams, Actionlike,
};

use bevy::ecs::prelude::*;
use bevy::utils::HashSet;
use bevy::{
    input::{
        gamepad::{GamepadAxis, GamepadButton, Gamepads},
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
        Axis, ButtonInput,
    },
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

/// Sums the[`MouseMotion`] events received since during this frame.
pub fn accumulate_mouse_movement(
    mut mouse_motion: ResMut<AccumulatedMouseMovement>,
    mut events: EventReader<MouseMotion>,
) {
    mouse_motion.reset();

    for event in events.read() {
        mouse_motion.accumulate(event);
    }
}

/// Sums the [`MouseWheel`] events received since during this frame.
pub fn accumulate_mouse_scroll(
    mut mouse_scroll: ResMut<AccumulatedMouseScroll>,
    mut events: EventReader<MouseWheel>,
) {
    mouse_scroll.reset();

    for event in events.read() {
        mouse_scroll.accumulate(event);
    }
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
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mouse_motion: Res<AccumulatedMouseMovement>,
    clash_strategy: Res<ClashStrategy>,
    #[cfg(feature = "ui")] interactions: Query<&Interaction>,
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
    let mouse_scroll = mouse_scroll.into_inner();
    let mouse_motion = mouse_motion.into_inner();

    // If the user clicks on a button, do not apply it to the game state
    #[cfg(feature = "ui")]
    let mouse_buttons = if interactions
        .iter()
        .any(|&interaction| interaction != Interaction::None)
    {
        None
    } else {
        mouse_buttons
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
    let mouse_buttons = if maybe_egui.iter_mut().any(|(_, mut ctx)| {
        ctx.get_mut().is_pointer_over_area() || ctx.get_mut().wants_pointer_input()
    }) {
        None
    } else {
        mouse_buttons
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
            mouse_scroll,
            mouse_motion,
            associated_gamepad: input_map.gamepad(),
        };

        action_state.update(input_map.process_actions(&input_streams, *clash_strategy));
    }
}

/// Generates an [`Events`] stream of [`ActionDiff`] from [`ActionState`]
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
pub fn generate_action_diffs<A: Actionlike>(
    global_action_state: Option<Res<ActionState<A>>>,
    action_state_query: Query<(Entity, &ActionState<A>)>,
    mut previous_action_state: Local<SummarizedActionState<A>>,
    mut action_diff_events: EventWriter<ActionDiffEvent<A>>,
) {
    let current_action_state =
        SummarizedActionState::summarize(global_action_state, action_state_query);
    current_action_state.send_diffs(&previous_action_state, &mut action_diff_events);
    *previous_action_state = current_action_state;
}

/// Stores the state of all actions in the current frame.
///
/// Inside of the hashmap, [`Entity::PLACEHOLDER`] represents the global / resource state of the action.
#[derive(Debug)]
pub struct SummarizedActionState<A: Actionlike> {
    button_state_map: HashMap<Entity, HashMap<A, bool>>,
    axis_state_map: HashMap<Entity, HashMap<A, f32>>,
    dual_axis_state_map: HashMap<Entity, HashMap<A, Vec2>>,
}

impl<A: Actionlike> SummarizedActionState<A> {
    /// Returns a list of all entities that are contained within this data structure.
    ///
    /// This includes the global / resource state, using [`Entity::PLACEHOLDER`].
    pub fn all_entities(&self) -> HashSet<Entity> {
        let mut entities = HashSet::new();
        let button_entities = self.button_state_map.keys();
        let axis_entities = self.axis_state_map.keys();
        let dual_axis_entities = self.dual_axis_state_map.keys();

        entities.extend(button_entities);
        entities.extend(axis_entities);
        entities.extend(dual_axis_entities);

        entities
    }

    /// Captures the raw values for each action in the current frame.
    pub fn summarize(
        global_action_state: Option<Res<ActionState<A>>>,
        action_state_query: Query<(Entity, &ActionState<A>)>,
    ) -> Self {
        let mut button_state_map = HashMap::default();
        let mut axis_state_map = HashMap::default();
        let mut dual_axis_state_map = HashMap::default();

        if let Some(global_action_state) = global_action_state {
            let mut per_entity_button_state = HashMap::default();
            let mut per_entity_axis_state = HashMap::default();
            let mut per_entity_dual_axis_state = HashMap::default();

            for (action, button_data) in global_action_state.all_button_data() {
                per_entity_button_state.insert(action.clone(), button_data.pressed());
            }

            for (action, axis_data) in global_action_state.all_axis_data() {
                per_entity_axis_state.insert(action.clone(), axis_data.value);
            }

            for (action, dual_axis_data) in global_action_state.all_dual_axis_data() {
                per_entity_dual_axis_state.insert(action.clone(), dual_axis_data.pair);
            }

            button_state_map.insert(Entity::PLACEHOLDER, per_entity_button_state);
            axis_state_map.insert(Entity::PLACEHOLDER, per_entity_axis_state);
            dual_axis_state_map.insert(Entity::PLACEHOLDER, per_entity_dual_axis_state);
        }

        for (entity, action_state) in action_state_query.iter() {
            let mut per_entity_button_state = HashMap::default();
            let mut per_entity_axis_state = HashMap::default();
            let mut per_entity_dual_axis_state = HashMap::default();

            for (action, button_data) in action_state.all_button_data() {
                per_entity_button_state.insert(action.clone(), button_data.pressed());
            }

            for (action, axis_data) in action_state.all_axis_data() {
                per_entity_axis_state.insert(action.clone(), axis_data.value);
            }

            for (action, dual_axis_data) in action_state.all_dual_axis_data() {
                per_entity_dual_axis_state.insert(action.clone(), dual_axis_data.pair);
            }

            button_state_map.insert(entity, per_entity_button_state);
            axis_state_map.insert(entity, per_entity_axis_state);
            dual_axis_state_map.insert(entity, per_entity_dual_axis_state);
        }

        Self {
            button_state_map,
            axis_state_map,
            dual_axis_state_map,
        }
    }

    /// Generates an [`ActionDiff`] for button data,
    /// if the button has changed state.
    ///
    ///
    /// Previous values will be treated as default if they were not present.
    pub fn button_diff(
        action: A,
        previous_button: Option<bool>,
        current_button: Option<bool>,
    ) -> Option<ActionDiff<A>> {
        let previous_button = previous_button.unwrap_or_default();
        let current_button = current_button?;

        if previous_button != current_button {
            if current_button {
                Some(ActionDiff::Pressed { action })
            } else {
                Some(ActionDiff::Released { action })
            }
        } else {
            None
        }
    }

    /// Generates an [`ActionDiff`] for axis data,
    /// if the axis has changed state.
    ///
    /// Previous values will be treated as default if they were not present.
    pub fn axis_diff(
        action: A,
        previous_axis: Option<f32>,
        current_axis: Option<f32>,
    ) -> Option<ActionDiff<A>> {
        let previous_axis = previous_axis.unwrap_or_default();
        let current_axis = current_axis?;

        if previous_axis != current_axis {
            Some(ActionDiff::AxisChanged {
                action,
                value: current_axis,
            })
        } else {
            None
        }
    }

    /// Generates an [`ActionDiff`] for dual axis data,
    /// if the dual axis has changed state.
    pub fn dual_axis_diff(
        action: A,
        previous_dual_axis: Option<Vec2>,
        current_dual_axis: Option<Vec2>,
    ) -> Option<ActionDiff<A>> {
        let previous_dual_axis = previous_dual_axis.unwrap_or_default();
        let current_dual_axis = current_dual_axis?;

        if previous_dual_axis != current_dual_axis {
            Some(ActionDiff::DualAxisChanged {
                action,
                axis_pair: current_dual_axis,
            })
        } else {
            None
        }
    }

    /// Generates all [`ActionDiff`]s for a single entity.
    pub fn entity_diffs(
        &self,
        previous_button_state: Option<&HashMap<A, bool>>,
        current_button_state: Option<&HashMap<A, bool>>,
        previous_axis_state: Option<&HashMap<A, f32>>,
        current_axis_state: Option<&HashMap<A, f32>>,
        previous_dual_axis_state: Option<&HashMap<A, Vec2>>,
        current_dual_axis_state: Option<&HashMap<A, Vec2>>,
    ) -> Vec<ActionDiff<A>> {
        let mut action_diffs = Vec::new();

        if let Some(current_button_state) = current_button_state {
            for (action, current_button) in current_button_state {
                let previous_button = previous_button_state
                    .and_then(|previous_button_state| previous_button_state.get(action))
                    .copied();

                if let Some(diff) =
                    Self::button_diff(action.clone(), previous_button, Some(*current_button))
                {
                    action_diffs.push(diff);
                }
            }
        }

        if let Some(current_axis_state) = current_axis_state {
            for (action, current_axis) in current_axis_state {
                let previous_axis = previous_axis_state
                    .and_then(|previous_axis_state| previous_axis_state.get(action))
                    .copied();

                if let Some(diff) =
                    Self::axis_diff(action.clone(), previous_axis, Some(*current_axis))
                {
                    action_diffs.push(diff);
                }
            }
        }

        if let Some(current_dual_axis_state) = current_dual_axis_state {
            for (action, current_dual_axis) in current_dual_axis_state {
                let previous_dual_axis = previous_dual_axis_state
                    .and_then(|previous_dual_axis_state| previous_dual_axis_state.get(action))
                    .copied();

                if let Some(diff) = Self::dual_axis_diff(
                    action.clone(),
                    previous_dual_axis,
                    Some(*current_dual_axis),
                ) {
                    action_diffs.push(diff);
                }
            }
        }

        action_diffs
    }

    /// Compares the current frame to the previous frame, generates [`ActionDiff`]s and then sends them as batched [`ActionDiffEvent`]s.
    pub fn send_diffs(&self, previous: &Self, writer: &mut EventWriter<ActionDiffEvent<A>>) {
        for entity in self.all_entities() {
            let owner = if entity == Entity::PLACEHOLDER {
                None
            } else {
                Some(entity)
            };

            let previous_button_state = previous.button_state_map.get(&entity);
            let current_button_state = self.button_state_map.get(&entity);
            let previous_axis_state = previous.axis_state_map.get(&entity);
            let current_axis_state = self.axis_state_map.get(&entity);
            let previous_dual_axis_state = previous.dual_axis_state_map.get(&entity);
            let current_dual_axis_state = self.dual_axis_state_map.get(&entity);

            let action_diffs = self.entity_diffs(
                previous_button_state,
                current_button_state,
                previous_axis_state,
                current_axis_state,
                previous_dual_axis_state,
                current_dual_axis_state,
            );

            writer.send(ActionDiffEvent {
                owner,
                action_diffs,
            });
        }
    }
}

// Manual impl due to A not being bounded by Default messing with the derive
impl<A: Actionlike> Default for SummarizedActionState<A> {
    fn default() -> Self {
        Self {
            button_state_map: Default::default(),
            axis_state_map: Default::default(),
            dual_axis_state_map: Default::default(),
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
