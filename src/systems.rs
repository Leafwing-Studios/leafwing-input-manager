//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

#[cfg(feature = "ui")]
use crate::action_state::ActionStateDriver;
use crate::{
    action_state::{ActionDiff, ActionState},
    clashing_inputs::ClashStrategy,
    input_map::InputMap,
    plugin::ToggleActions,
    press_scheduler::PressScheduler,
    Actionlike,
};

use bevy::ecs::prelude::*;
use bevy::ecs::system::SystemState;
use bevy::time::Time;
use bevy::utils::Instant;

use crate::buttonlike::ButtonState;
use crate::input_streams::InputStreams;
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

/// Fetches all of the relevant [`Input`] resources to update [`ActionState`] according to the [`InputMap`].
///
/// Missing resources will be ignored, and treated as if none of the corresponding inputs were pressed.
pub fn update_action_state<A: Actionlike>(world: &mut World) {
    let input_maps: Vec<_> = {
        let mut state =
            SystemState::<(Option<Res<InputMap<A>>>, Query<(Entity, &InputMap<A>)>)>::new(world);
        let (input_map_resource, input_maps) = state.get(world);
        // Have to clone the input maps because we can't keep any references to world
        input_maps
            .iter()
            .map(|(e, m)| (e, m.clone()))
            .chain(input_map_resource.map(|r| (Entity::PLACEHOLDER, r.clone())))
            .collect()
    };

    let entity_action_data = world.resource_scope(|world, clash_strategy: Mut<ClashStrategy>| {
        let router = InputStreams::from_world(world);
        input_maps
            .iter()
            .map(|(e, m)| (*e, m.which_pressed(&router, *clash_strategy)))
            .collect::<Vec<_>>()
    });

    let mut state = SystemState::<(
        Option<ResMut<ActionState<A>>>,
        Option<ResMut<PressScheduler<A>>>,
        Query<(&mut ActionState<A>, Option<&mut PressScheduler<A>>)>,
    )>::new(world);

    for (entity, action_data) in entity_action_data {
        let (action_state, press_scheduler, mut query) = state.get_mut(world);

        // Get from either resource or component
        let (mut action_state, press_scheduler) = if entity == Entity::PLACEHOLDER {
            if let Some(action_state) = action_state {
                (Mut::from(action_state), press_scheduler.map(Mut::from))
            } else {
                continue;
            }
        } else if let Ok((action_state, press_scheduler)) = query.get_mut(entity) {
            (action_state, press_scheduler)
        } else {
            continue;
        };

        for action_data in &action_data {
            if action_data.state == ButtonState::Pressed {
                println!("{:?}", action_data);
            }
        }
        action_state.update(action_data);
        if let Some(mut press_scheduler) = press_scheduler {
            press_scheduler.apply(action_state.as_mut());
        }
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
/// The `ID` generic type should be a stable entity identifier,
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
/// The `ID` generic type should be a stable entity identifier,
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

/// Release all inputs when an [`InputMap<A>`] is removed to prevent them from being held forever.
///
/// By default, [`InputManagerPlugin<A>`](crate::plugin::InputManagerPlugin) will run this on [`CoreSet::PostUpdate`](bevy::prelude::CoreSet::PostUpdate).
/// For components you must remove the [`InputMap<A>`] before [`CoreSet::PostUpdate`](bevy::prelude::CoreSet::PostUpdate)
/// or this will not run.
pub fn release_on_input_map_removed<A: Actionlike>(
    mut removed_components: RemovedComponents<InputMap<A>>,
    input_map_resource: Option<ResMut<InputMap<A>>>,
    action_state_resource: Option<ResMut<ActionState<A>>>,
    mut input_map_resource_existed: Local<bool>,
    mut action_state_query: Query<&mut ActionState<A>>,
) {
    let mut iter = action_state_query.iter_many_mut(removed_components.iter());
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
pub(super) fn run_if_enabled<A: Actionlike>(toggle_actions: Res<ToggleActions<A>>) -> bool {
    toggle_actions.enabled
}
