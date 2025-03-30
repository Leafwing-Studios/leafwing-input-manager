//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

use crate::prelude::updating::CentralInputStore;
use bevy::ecs::query::QueryFilter;
use bevy::log::debug;

use crate::{
    action_state::ActionState, clashing_inputs::ClashStrategy, input_map::InputMap, Actionlike,
};

use bevy::ecs::prelude::*;
use bevy::prelude::Gamepad;
use bevy::{
    platform_support::time::Instant,
    time::{Real, Time},
};

use crate::action_diff::{ActionDiffEvent, SummarizedActionState};

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

/// Fetches the [`CentralInputStore`]
/// to update [`ActionState`] according to the [`InputMap`].
///
/// Clashes will be resolved according to the [`ClashStrategy`] resource.
pub fn update_action_state<A: Actionlike>(
    input_store: Res<CentralInputStore>,
    clash_strategy: Res<ClashStrategy>,
    mut gamepads: Query<Entity, With<Gamepad>>,
    action_state: Option<ResMut<ActionState<A>>>,
    input_map: Option<Res<InputMap<A>>>,
    mut query: Query<(&mut ActionState<A>, &InputMap<A>)>,
) {
    let resources = input_map
        .zip(action_state)
        .map(|(input_map, action_state)| (Mut::from(action_state), input_map.into_inner()));

    for (mut action_state, input_map) in query.iter_mut().chain(resources) {
        action_state.update(input_map.process_actions(
            Some(gamepads.reborrow()),
            &input_store,
            *clash_strategy,
        ));
    }
}

#[cfg(any(feature = "egui", feature = "ui"))]
/// Filters out all inputs that are captured by the UI.
pub fn filter_captured_input(
    mut mouse_buttons: ResMut<bevy::input::ButtonInput<bevy::input::mouse::MouseButton>>,
    #[cfg(feature = "ui")] interactions: Query<&bevy::ui::Interaction>,
    #[cfg(feature = "egui")] mut keycodes: ResMut<
        bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>,
    >,
    #[cfg(feature = "egui")] mut egui_query: Query<&'static mut bevy_egui::EguiContext>,
) {
    // If the user clicks on a button, do not apply it to the game state
    #[cfg(feature = "ui")]
    if interactions
        .iter()
        .any(|&interaction| interaction != bevy::ui::Interaction::None)
    {
        mouse_buttons.clear();
    }

    // If egui wants to own inputs, don't also apply them to the game state
    #[cfg(feature = "egui")]
    for mut egui_context in egui_query.iter_mut() {
        // Don't ask me why get doesn't exist :shrug:
        if egui_context.get_mut().wants_pointer_input() {
            mouse_buttons.reset_all();
        }

        if egui_context.get_mut().wants_keyboard_input() {
            keycodes.reset_all();
        }
    }
}

/// Generates an [`Events`] stream of [`ActionDiff`s](crate::action_diff::ActionDiff) from every [`ActionState`].
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
/// Generally speaking, this should be added as part of [`PostUpdate`](bevy::prelude::PostUpdate),
/// to ensure that all inputs have been processed and any manual actions have been sent.
pub fn generate_action_diffs<A: Actionlike>(
    global_action_state: Option<Res<ActionState<A>>>,
    action_state_query: Query<(Entity, &ActionState<A>)>,
    previous_action_state: Local<SummarizedActionState<A>>,
    action_diff_events: EventWriter<ActionDiffEvent<A>>,
) {
    generate_action_diffs_filtered(
        global_action_state,
        action_state_query,
        previous_action_state,
        action_diff_events,
    )
}

/// Generates an [`Events`] stream of [`ActionDiff`s](crate::action_diff::ActionDiff) from the [`ActionState`] of certain entities.
///
/// This system is not part of the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) and must be added manually.
/// Generally speaking, this should be added as part of [`PostUpdate`](bevy::prelude::PostUpdate),
/// to ensure that all inputs have been processed and any manual actions have been sent.
///
/// This system accepts a [`QueryFilter`] to limit which entities should have action diffs generated.
pub fn generate_action_diffs_filtered<A: Actionlike, F: QueryFilter>(
    global_action_state: Option<Res<ActionState<A>>>,
    action_state_query: Query<(Entity, &ActionState<A>), F>,
    mut previous_action_state: Local<SummarizedActionState<A>>,
    mut action_diff_events: EventWriter<ActionDiffEvent<A>>,
) {
    let current_action_state =
        SummarizedActionState::summarize_filtered(global_action_state, action_state_query);
    current_action_state.send_diffs(&previous_action_state, &mut action_diff_events);
    debug!("previous_action_state: {:?}", previous_action_state);
    debug!("current_action_state: {:?}", current_action_state);
    *previous_action_state = current_action_state;
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
        action_state.reset_all();
    }

    // Detect when an InputMap resource is removed.
    if input_map_resource.is_some() {
        // Store if the resource existed, so we know if it was removed later.
        *input_map_resource_existed = true;
    } else if *input_map_resource_existed {
        // The input map does not exist, and our local is true,
        // so we know the input map was removed.

        if let Some(mut action_state) = action_state_resource {
            action_state.reset_all();
        }

        // Reset our local so our removal detection is only triggered once.
        *input_map_resource_existed = false;
    }
}

/// Clears all values from the [`CentralInputStore`],
/// making sure that it can read fresh inputs for the frame.
pub fn clear_central_input_store(mut input_store: ResMut<CentralInputStore>) {
    input_store.clear();
}
