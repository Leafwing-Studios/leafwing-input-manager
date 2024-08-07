//! Serialization-friendly representation of changes to [`ActionState`](crate::action_state::ActionState).
//!
//! These are predominantly intended for use in networked games,
//! where the server needs to know what the players are doing.
//! They would like a compact, semantically meaningful representation of the changes to the game state without needing to know
//! about things like keybindings or input devices.

use bevy::{
    ecs::{entity::Entity, event::Event},
    math::Vec2,
    prelude::{EventWriter, Query, Res},
    utils::{HashMap, HashSet},
};
use serde::{Deserialize, Serialize};

use crate::{action_state::ActionKindData, prelude::ActionState, Actionlike};

/// Stores presses and releases of buttons without timing information
///
/// These are typically accessed using the `Events<ActionDiffEvent>` resource.
/// Uses a minimal storage format to facilitate transport over the network.
///
/// An `ActionState` can be fully reconstructed from a stream of `ActionDiff`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionDiff<A: Actionlike> {
    /// The action was pressed
    Pressed {
        /// The value of the action
        action: A,
    },
    /// The action was released
    Released {
        /// The value of the action
        action: A,
    },
    /// The value of the action changed
    AxisChanged {
        /// The value of the action
        action: A,
        /// The new value of the action
        value: f32,
    },
    /// The axis pair of the action changed
    DualAxisChanged {
        /// The value of the action
        action: A,
        /// The new value of the axis
        axis_pair: Vec2,
    },
}

/// Will store an `ActionDiff` as well as what generated it (either an Entity, or nothing if the
/// input actions are represented by a `Resource`)
///
/// These are typically accessed using the `Events<ActionDiffEvent>` resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Event)]
pub struct ActionDiffEvent<A: Actionlike> {
    /// If some: the entity that has the `ActionState<A>` component
    /// If none: `ActionState<A>` is a Resource, not a component
    pub owner: Option<Entity>,
    /// The `ActionDiff` that was generated
    pub action_diffs: Vec<ActionDiff<A>>,
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

            for (action, action_data) in global_action_state.all_action_data() {
                match &action_data.kind_data {
                    ActionKindData::Button(button_data) => {
                        per_entity_button_state.insert(action.clone(), button_data.pressed());
                    }
                    ActionKindData::Axis(axis_data) => {
                        per_entity_axis_state.insert(action.clone(), axis_data.value);
                    }
                    ActionKindData::DualAxis(dual_axis_data) => {
                        per_entity_dual_axis_state.insert(action.clone(), dual_axis_data.pair);
                    }
                }
            }

            button_state_map.insert(Entity::PLACEHOLDER, per_entity_button_state);
            axis_state_map.insert(Entity::PLACEHOLDER, per_entity_axis_state);
            dual_axis_state_map.insert(Entity::PLACEHOLDER, per_entity_dual_axis_state);
        }

        for (entity, action_state) in action_state_query.iter() {
            let mut per_entity_button_state = HashMap::default();
            let mut per_entity_axis_state = HashMap::default();
            let mut per_entity_dual_axis_state = HashMap::default();

            for (action, action_data) in action_state.all_action_data() {
                match &action_data.kind_data {
                    ActionKindData::Button(button_data) => {
                        per_entity_button_state.insert(action.clone(), button_data.pressed());
                    }
                    ActionKindData::Axis(axis_data) => {
                        per_entity_axis_state.insert(action.clone(), axis_data.value);
                    }
                    ActionKindData::DualAxis(dual_axis_data) => {
                        per_entity_dual_axis_state.insert(action.clone(), dual_axis_data.pair);
                    }
                }
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
