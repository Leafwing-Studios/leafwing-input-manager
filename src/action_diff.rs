//! Serialization-friendly representation of changes to [`ActionState`].
//!
//! These are predominantly intended for use in networked games,
//! where the server needs to know what the players are doing.
//! They would like a compact, semantically meaningful representation of the changes to the game state without needing to know
//! about things like keybindings or input devices.

use bevy::{
    ecs::{
        entity::{Entity, MapEntities},
        event::Event,
        query::QueryFilter,
    },
    math::{Vec2, Vec3},
    platform::collections::{HashMap, HashSet},
    prelude::{EntityMapper, EventWriter, Query, Res},
};
use serde::{Deserialize, Serialize};

use crate::buttonlike::ButtonValue;
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
        /// The new value of the action
        value: f32,
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
        /// The new value of the axes
        axis_pair: Vec2,
    },
    /// The axis pair of the action changed
    TripleAxisChanged {
        /// The value of the action
        action: A,
        /// The new value of the axes
        axis_triple: Vec3,
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

/// Implements entity mapping for `ActionDiffEvent`.
///
/// This allows the owner entity to be remapped when transferring event diffs
/// between different ECS worlds (e.g. client and server).
impl<A: Actionlike> MapEntities for ActionDiffEvent<A> {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.owner = self.owner.map(|entity| entity_mapper.get_mapped(entity));
    }
}

/// Stores the state of all actions in the current frame.
///
/// Inside of the hashmap, [`Entity::PLACEHOLDER`] represents the global / resource state of the action.
#[derive(Debug, PartialEq, Clone)]
pub struct SummarizedActionState<A: Actionlike> {
    button_state_map: HashMap<Entity, HashMap<A, ButtonValue>>,
    axis_state_map: HashMap<Entity, HashMap<A, f32>>,
    dual_axis_state_map: HashMap<Entity, HashMap<A, Vec2>>,
    triple_axis_state_map: HashMap<Entity, HashMap<A, Vec3>>,
}

impl<A: Actionlike> SummarizedActionState<A> {
    /// Returns a list of all entities that are contained within this data structure.
    ///
    /// This includes the global / resource state, using [`Entity::PLACEHOLDER`].
    pub fn all_entities(&self) -> HashSet<Entity> {
        let mut entities = HashSet::default();
        let button_entities = self.button_state_map.keys();
        let axis_entities = self.axis_state_map.keys();
        let dual_axis_entities = self.dual_axis_state_map.keys();
        let triple_axis_entities = self.triple_axis_state_map.keys();

        entities.extend(button_entities);
        entities.extend(axis_entities);
        entities.extend(dual_axis_entities);
        entities.extend(triple_axis_entities);

        entities
    }

    /// Captures the raw values for each action in the current frame, for all entities with `ActionState<A>`.
    pub fn summarize(
        global_action_state: Option<Res<ActionState<A>>>,
        action_state_query: Query<(Entity, &ActionState<A>)>,
    ) -> Self {
        Self::summarize_filtered(global_action_state, action_state_query)
    }

    /// Captures the raw values for each action in the current frame, for entities with `ActionState<A>`
    /// matching the query filter.
    pub fn summarize_filtered<F: QueryFilter>(
        global_action_state: Option<Res<ActionState<A>>>,
        action_state_query: Query<(Entity, &ActionState<A>), F>,
    ) -> Self {
        let mut button_state_map = HashMap::default();
        let mut axis_state_map = HashMap::default();
        let mut dual_axis_state_map = HashMap::default();
        let mut triple_axis_state_map = HashMap::default();

        if let Some(global_action_state) = global_action_state {
            let mut per_entity_button_state = HashMap::default();
            let mut per_entity_axis_state = HashMap::default();
            let mut per_entity_dual_axis_state = HashMap::default();
            let mut per_entity_triple_axis_state = HashMap::default();

            for (action, action_data) in global_action_state.all_action_data() {
                match &action_data.kind_data {
                    ActionKindData::Button(button_data) => {
                        per_entity_button_state
                            .insert(action.clone(), button_data.to_button_value());
                    }
                    ActionKindData::Axis(axis_data) => {
                        per_entity_axis_state.insert(action.clone(), axis_data.value);
                    }
                    ActionKindData::DualAxis(dual_axis_data) => {
                        per_entity_dual_axis_state.insert(action.clone(), dual_axis_data.pair);
                    }
                    ActionKindData::TripleAxis(triple_axis_data) => {
                        per_entity_triple_axis_state
                            .insert(action.clone(), triple_axis_data.triple);
                    }
                }
            }

            button_state_map.insert(Entity::PLACEHOLDER, per_entity_button_state);
            axis_state_map.insert(Entity::PLACEHOLDER, per_entity_axis_state);
            dual_axis_state_map.insert(Entity::PLACEHOLDER, per_entity_dual_axis_state);
            triple_axis_state_map.insert(Entity::PLACEHOLDER, per_entity_triple_axis_state);
        }

        for (entity, action_state) in action_state_query.iter() {
            let mut per_entity_button_state = HashMap::default();
            let mut per_entity_axis_state = HashMap::default();
            let mut per_entity_dual_axis_state = HashMap::default();
            let mut per_entity_triple_axis_state = HashMap::default();

            for (action, action_data) in action_state.all_action_data() {
                match &action_data.kind_data {
                    ActionKindData::Button(button_data) => {
                        per_entity_button_state
                            .insert(action.clone(), button_data.to_button_value());
                    }
                    ActionKindData::Axis(axis_data) => {
                        per_entity_axis_state.insert(action.clone(), axis_data.value);
                    }
                    ActionKindData::DualAxis(dual_axis_data) => {
                        per_entity_dual_axis_state.insert(action.clone(), dual_axis_data.pair);
                    }
                    ActionKindData::TripleAxis(triple_axis_data) => {
                        per_entity_triple_axis_state
                            .insert(action.clone(), triple_axis_data.triple);
                    }
                }
            }

            button_state_map.insert(entity, per_entity_button_state);
            axis_state_map.insert(entity, per_entity_axis_state);
            dual_axis_state_map.insert(entity, per_entity_dual_axis_state);
            triple_axis_state_map.insert(entity, per_entity_triple_axis_state);
        }

        Self {
            button_state_map,
            axis_state_map,
            dual_axis_state_map,
            triple_axis_state_map,
        }
    }

    /// Generates an [`ActionDiff`] for button data,
    /// if the button has changed state.
    ///
    ///
    /// Previous values will be treated as default if they were not present.
    pub fn button_diff(
        action: A,
        previous_button: Option<ButtonValue>,
        current_button: Option<ButtonValue>,
    ) -> Option<ActionDiff<A>> {
        let previous_button = previous_button.unwrap_or_default();
        let current_button = current_button?;

        (previous_button != current_button).then(|| {
            if current_button.pressed {
                ActionDiff::Pressed {
                    action,
                    value: current_button.value,
                }
            } else {
                ActionDiff::Released { action }
            }
        })
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

        (previous_axis != current_axis).then(|| ActionDiff::AxisChanged {
            action,
            value: current_axis,
        })
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

        (previous_dual_axis != current_dual_axis).then(|| ActionDiff::DualAxisChanged {
            action,
            axis_pair: current_dual_axis,
        })
    }

    /// Generates an [`ActionDiff`] for triple axis data,
    /// if the triple axis has changed state.
    pub fn triple_axis_diff(
        action: A,
        previous_triple_axis: Option<Vec3>,
        current_triple_axis: Option<Vec3>,
    ) -> Option<ActionDiff<A>> {
        let previous_triple_axis = previous_triple_axis.unwrap_or_default();
        let current_triple_axis = current_triple_axis?;

        (previous_triple_axis != current_triple_axis).then(|| ActionDiff::TripleAxisChanged {
            action,
            axis_triple: current_triple_axis,
        })
    }

    /// Generates all [`ActionDiff`]s for a single entity.
    pub fn entity_diffs(&self, entity: &Entity, previous: &Self) -> Vec<ActionDiff<A>> {
        let mut action_diffs = Vec::new();

        if let Some(current_button_state) = self.button_state_map.get(entity) {
            let previous_button_state = previous.button_state_map.get(entity);
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

        if let Some(current_axis_state) = self.axis_state_map.get(entity) {
            let previous_axis_state = previous.axis_state_map.get(entity);
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

        if let Some(current_dual_axis_state) = self.dual_axis_state_map.get(entity) {
            let previous_dual_axis_state = previous.dual_axis_state_map.get(entity);
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

        if let Some(current_triple_axis_state) = self.triple_axis_state_map.get(entity) {
            let previous_triple_axis_state = previous.triple_axis_state_map.get(entity);
            for (action, current_triple_axis) in current_triple_axis_state {
                let previous_triple_axis = previous_triple_axis_state
                    .and_then(|previous_triple_axis_state| previous_triple_axis_state.get(action))
                    .copied();

                if let Some(diff) = Self::triple_axis_diff(
                    action.clone(),
                    previous_triple_axis,
                    Some(*current_triple_axis),
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
            let owner = (entity != Entity::PLACEHOLDER).then_some(entity);

            let action_diffs = self.entity_diffs(&entity, previous);

            if !action_diffs.is_empty() {
                writer.write(ActionDiffEvent {
                    owner,
                    action_diffs,
                });
            }
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
            triple_axis_state_map: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as leafwing_input_manager;

    use super::*;
    use crate::buttonlike::ButtonValue;
    use bevy::{ecs::system::SystemState, prelude::*};

    #[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    enum TestAction {
        Button,
        #[actionlike(Axis)]
        Axis,
        #[actionlike(DualAxis)]
        DualAxis,
        #[actionlike(TripleAxis)]
        TripleAxis,
    }

    fn test_action_state() -> ActionState<TestAction> {
        let mut action_state = ActionState::default();
        action_state.press(&TestAction::Button);
        action_state.set_value(&TestAction::Axis, 0.3);
        action_state.set_axis_pair(&TestAction::DualAxis, Vec2::new(0.5, 0.7));
        action_state.set_axis_triple(&TestAction::TripleAxis, Vec3::new(0.5, 0.7, 0.9));
        action_state
    }

    #[derive(Component)]
    struct NotSummarized;

    fn expected_summary(entity: Entity) -> SummarizedActionState<TestAction> {
        let mut button_state_map = HashMap::default();
        let mut axis_state_map = HashMap::default();
        let mut dual_axis_state_map = HashMap::default();
        let mut triple_axis_state_map = HashMap::default();

        let mut global_button_state = HashMap::default();
        global_button_state.insert(TestAction::Button, ButtonValue::from_pressed(true));
        button_state_map.insert(entity, global_button_state);

        let mut global_axis_state = HashMap::default();
        global_axis_state.insert(TestAction::Axis, 0.3);
        axis_state_map.insert(entity, global_axis_state);

        let mut global_dual_axis_state = HashMap::default();
        global_dual_axis_state.insert(TestAction::DualAxis, Vec2::new(0.5, 0.7));
        dual_axis_state_map.insert(entity, global_dual_axis_state);

        let mut global_triple_axis_state = HashMap::default();
        global_triple_axis_state.insert(TestAction::TripleAxis, Vec3::new(0.5, 0.7, 0.9));
        triple_axis_state_map.insert(entity, global_triple_axis_state);

        SummarizedActionState {
            button_state_map,
            axis_state_map,
            dual_axis_state_map,
            triple_axis_state_map,
        }
    }

    #[test]
    fn summarize_from_resource() {
        let mut world = World::new();
        world.insert_resource(test_action_state());
        let mut system_state: SystemState<(
            Option<Res<ActionState<TestAction>>>,
            Query<(Entity, &ActionState<TestAction>)>,
        )> = SystemState::new(&mut world);
        let (global_action_state, action_state_query) = system_state.get(&world);
        let summarized = SummarizedActionState::summarize(global_action_state, action_state_query);

        // Resources use the placeholder entity
        assert_eq!(summarized, expected_summary(Entity::PLACEHOLDER));
    }

    #[test]
    fn summarize_from_component() {
        let mut world = World::new();
        let entity = world.spawn(test_action_state()).id();
        let mut system_state: SystemState<(
            Option<Res<ActionState<TestAction>>>,
            Query<(Entity, &ActionState<TestAction>)>,
        )> = SystemState::new(&mut world);
        let (global_action_state, action_state_query) = system_state.get(&world);
        let summarized = SummarizedActionState::summarize(global_action_state, action_state_query);

        // Components use the entity
        assert_eq!(summarized, expected_summary(entity));
    }

    #[test]
    fn summarize_filtered_entities_from_component() {
        // Spawn two entities, one to be summarized and one to be filtered out
        let mut world = World::new();
        let entity = world.spawn(test_action_state()).id();
        world.spawn((test_action_state(), NotSummarized));

        let mut system_state: SystemState<(
            Option<Res<ActionState<TestAction>>>,
            Query<(Entity, &ActionState<TestAction>), Without<NotSummarized>>,
        )> = SystemState::new(&mut world);
        let (global_action_state, action_state_query) = system_state.get(&world);
        let summarized =
            SummarizedActionState::summarize_filtered(global_action_state, action_state_query);

        // Check that only the entity without NotSummarized was summarized
        assert_eq!(summarized, expected_summary(entity));
    }

    #[test]
    fn diffs_are_sent() {
        let mut world = World::new();
        world.init_resource::<Events<ActionDiffEvent<TestAction>>>();

        let entity = world.spawn(test_action_state()).id();
        let mut system_state: SystemState<(
            Option<Res<ActionState<TestAction>>>,
            Query<(Entity, &ActionState<TestAction>)>,
            EventWriter<ActionDiffEvent<TestAction>>,
        )> = SystemState::new(&mut world);
        let (global_action_state, action_state_query, mut action_diff_writer) =
            system_state.get_mut(&mut world);
        let summarized = SummarizedActionState::summarize(global_action_state, action_state_query);

        let previous = SummarizedActionState::default();
        summarized.send_diffs(&previous, &mut action_diff_writer);

        let mut system_state: SystemState<EventReader<ActionDiffEvent<TestAction>>> =
            SystemState::new(&mut world);
        let mut event_reader = system_state.get_mut(&mut world);
        let action_diff_events = event_reader.read().collect::<Vec<_>>();

        dbg!(&action_diff_events);
        assert_eq!(action_diff_events.len(), 1);
        let action_diff_event = action_diff_events[0];
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 4);
    }
}
