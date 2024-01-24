//! Tools to control the [`ActionState`] of other entities automatically from other entities.

use bevy::utils::hashbrown::hash_set::Iter;
use std::iter::Once;

use bevy::{
    ecs::{component::Component, entity::Entity},
    utils::HashSet,
};

use crate::Actionlike;

/// A component that allows the attached entity to drive the [`ActionState`] of the associated entity
///
/// # Examples
///
/// By default, [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction) uses this component
/// in order to connect `bevy::ui` buttons to the corresponding `ActionState`.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
/// enum DanceDance {
///     Left,
///     Right,
///     Up,
///     Down,
/// }
///
/// // Spawn entity to track dance inputs
/// let mut world = World::new();
/// let dance_tracker = world
///     .spawn(ActionState::<DanceDance>::default())
///     .id();
///
/// // Spawn a button, which is wired up to the dance tracker
/// // When used with InputManagerPlugin<DanceDance>, this button will press the DanceDance::Left action when it is pressed.
/// world
///     .spawn(ButtonBundle::default())
///     // This component links the button to the entity with the `ActionState` component
///     .insert(ActionStateDriver {
///         action: DanceDance::Left,
///         targets: dance_tracker.into(),
///     });
///```
///
/// Writing your own systems that use the [`ActionStateDriver`] component is easy,
/// although this should be reserved for cases where the entity whose value you want to check
/// is distinct from the entity whose [`ActionState`] you want to set.
/// Check the source code of [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction) for an example of how this is done.
#[derive(Debug, Component, Clone, PartialEq, Eq)]
pub struct ActionStateDriver<A: Actionlike> {
    /// The action triggered by this entity
    pub action: A,
    /// The entity whose action state should be updated
    pub targets: ActionStateDriverTarget,
}

/// Represents the entities that an ``ActionStateDriver`` targets.
#[derive(Debug, Component, Clone, PartialEq, Eq)]
pub enum ActionStateDriverTarget {
    /// No targets
    None,
    /// Single target
    Single(Entity),
    /// Multiple targets
    Multi(HashSet<Entity>),
}

impl ActionStateDriverTarget {
    /// Get an iterator for the entities targeted.
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        match self {
            Self::None => ActionStateDriverTargetIterator::None,
            Self::Single(entity) => {
                ActionStateDriverTargetIterator::Single(std::iter::once(entity))
            }
            Self::Multi(entities) => ActionStateDriverTargetIterator::Multi(entities.iter()),
        }
    }

    /// Insert an entity as a target.
    #[inline(always)]
    pub fn insert(&mut self, entity: Entity) {
        // Don't want to copy a bunch of logic, switch out the ref, then replace it
        // rust doesn't like in place replacement
        *self = std::mem::replace(self, Self::None).with(entity);
    }

    /// Remove an entity as a target if it's in the target set.
    #[inline(always)]
    pub fn remove(&mut self, entity: Entity) {
        // see insert
        *self = std::mem::replace(self, Self::None).without(entity);
    }

    /// Add an entity as a target.
    #[inline(always)]
    pub fn add(&mut self, entities: impl Iterator<Item = Entity>) {
        for entity in entities {
            self.insert(entity)
        }
    }

    /// Get the number of targets.
    #[inline(always)]
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Single(_) => 1,
            Self::Multi(targets) => targets.len(),
        }
    }

    /// Returns true if there are no targets.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add an entity as a target using a builder style pattern.
    #[inline(always)]
    pub fn with(mut self, entity: Entity) -> Self {
        match self {
            Self::None => Self::Single(entity),
            Self::Single(og) => Self::Multi(HashSet::from([og, entity])),
            Self::Multi(ref mut targets) => {
                targets.insert(entity);
                self
            }
        }
    }

    /// Remove an entity as a target if it's in the set using a builder style pattern.
    pub fn without(self, entity: Entity) -> Self {
        match self {
            Self::None => Self::None,
            Self::Single(_) => Self::None,
            Self::Multi(mut targets) => {
                targets.remove(&entity);
                Self::from_iter(targets)
            }
        }
    }
}

impl From<Entity> for ActionStateDriverTarget {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}

impl From<()> for ActionStateDriverTarget {
    fn from(_value: ()) -> Self {
        Self::None
    }
}

impl FromIterator<Entity> for ActionStateDriverTarget {
    fn from_iter<T: IntoIterator<Item = Entity>>(iter: T) -> Self {
        let entities = HashSet::from_iter(iter);

        match entities.len() {
            0 => Self::None,
            1 => Self::Single(entities.into_iter().next().unwrap()),
            _ => Self::Multi(entities),
        }
    }
}

impl<'a> FromIterator<&'a Entity> for ActionStateDriverTarget {
    fn from_iter<T: IntoIterator<Item = &'a Entity>>(iter: T) -> Self {
        let entities = HashSet::from_iter(iter.into_iter().cloned());

        match entities.len() {
            0 => Self::None,
            1 => Self::Single(entities.into_iter().next().unwrap()),
            _ => Self::Multi(entities),
        }
    }
}

enum ActionStateDriverTargetIterator<'a> {
    None,
    Single(Once<&'a Entity>),
    Multi(Iter<'a, Entity>),
}

impl<'a> Iterator for ActionStateDriverTargetIterator<'a> {
    type Item = &'a Entity;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::None => None,
            Self::Single(iter) => iter.next(),
            Self::Multi(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ActionStateDriverTarget;
    use bevy::prelude::Entity;

    #[test]
    fn action_state_driver_targets() {
        let mut target = ActionStateDriverTarget::from(());

        assert_eq!(0, target.len());

        target.insert(Entity::from_raw(0));
        assert_eq!(1, target.len());

        target.insert(Entity::from_raw(1));
        assert_eq!(2, target.len());

        target.remove(Entity::from_raw(0));
        assert_eq!(1, target.len());

        target.remove(Entity::from_raw(1));
        assert_eq!(0, target.len());

        target = target.with(Entity::from_raw(0));
        assert_eq!(1, target.len());

        target = target.without(Entity::from_raw(0));
        assert_eq!(0, target.len());

        target.add(
            [
                Entity::from_raw(0),
                Entity::from_raw(1),
                Entity::from_raw(2),
            ]
            .iter()
            .cloned(),
        );
        assert_eq!(3, target.len());

        let mut sum = 0;
        for entity in target.iter() {
            sum += entity.index();
        }
        assert_eq!(3, sum);
    }
}
