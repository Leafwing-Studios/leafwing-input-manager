//! Serialization-friendly representation of changes to [`ActionState`](crate::action_state::ActionState).
//!
//! These are predominantly intended for use in networked games,
//! where the server needs to know what the players are doing.
//! They would like a compact, semantically meaningful representation of the changes to the game state without needing to know
//! about things like keybindings or input devices.

use bevy::{
    ecs::{entity::Entity, event::Event},
    math::Vec2,
};
use serde::{Deserialize, Serialize};

use crate::Actionlike;

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
