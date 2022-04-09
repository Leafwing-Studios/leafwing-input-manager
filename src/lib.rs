#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use crate::action_state::ActionState;
use crate::input_map::InputMap;
use bevy_ecs::prelude::*;
use std::marker::PhantomData;

pub mod action_state;
pub mod clashing_inputs;
mod display_impl;
pub mod errors;
pub mod input_map;
mod input_mocking;
// Re-export this at the root level
pub use input_mocking::MockInput;
pub mod axislike_user_input;
pub mod buttonlike_user_input;
pub mod orientation;
pub mod plugin;
pub mod systems;
pub mod input_resource;

// Importing the derive macro
pub use leafwing_input_manager_macros::Actionlike;
use crate::prelude::UserInput;

/// Everything you need to get started
pub mod prelude {
    pub use crate::action_state::{ActionState, ActionStateDriver};
    pub use crate::buttonlike_user_input::UserInput;
    pub use crate::clashing_inputs::ClashStrategy;
    pub use crate::input_map::InputMap;
    pub use crate::input_resource::InputResource;

    pub use crate::plugin::DisableInput;
    pub use crate::plugin::InputManagerPlugin;
    pub use crate::{Actionlike, InputManagerBundle};
}

/// Allows a type to be used as a gameplay action in an input-agnostic fashion
///
/// Actions serve as "virtual buttons", cleanly abstracting over messy, customizable inputs
/// in a way that can be easily consumed by your game logic.
///
/// This trait should be implemented on the `A` type that you want to pass into [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).
///
/// Generally, these types will be very small (often data-less) enums.
/// As a result, the APIs in this crate accept actions by value, rather than reference.
/// While `Copy` is not a required trait bound,
/// users are strongly encouraged to derive `Copy` on these enums whenever possible to improve ergonomics.
///
/// # Example
/// ```rust
/// use leafwing_input_manager::Actionlike;
///
/// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash)]
/// enum PlayerAction {
///    // Movement
///    Up,
///    Down,
///    Left,
///    Right,
///    // Abilities
///    Ability1,
///    Ability2,
///    Ability3,
///    Ability4,
///    Ultimate,
/// }
/// ```
pub trait Actionlike: Send + Sync + Clone + 'static {
    /// The number of variants of this action type
    const N_VARIANTS: usize;

    /// Iterates over the possible actions in the order they were defined
    fn variants() -> ActionIter<Self> {
        ActionIter::default()
    }

    /// Returns the default value for the action stored at the provided index if it exists
    ///
    /// This is mostly used internally, to enable space-efficient iteration.
    fn get_at(index: usize) -> Option<Self>;

    /// Returns the position in the defining enum of the given action
    fn index(&self) -> usize;
}

/// An iterator of [`Actionlike`] actions
///
/// Created by calling [`Actionlike::iter`].
#[derive(Debug, Clone)]
pub struct ActionIter<A: Actionlike> {
    index: usize,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Iterator for ActionIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<A> {
        let item = A::get_at(self.index);
        if item.is_some() {
            self.index += 1;
        }

        item
    }
}

impl<A: Actionlike> ExactSizeIterator for ActionIter<A> {
    fn len(&self) -> usize {
        A::N_VARIANTS
    }
}

// We can't derive this, because otherwise it won't work when A is not default
impl<A: Actionlike> Default for ActionIter<A> {
    fn default() -> Self {
        ActionIter {
            index: 0,
            _phantom: PhantomData::default(),
        }
    }
}

/// This [`Bundle`] allows entities to collect and interpret inputs from across input sources
///
/// Use with [`InputManagerPlugin`](crate::plugin::InputManagerPlugin), providing the same enum type to both.
#[derive(Bundle)]
pub struct InputManagerBundle<A: Actionlike> {
    /// An [ActionState] component
    pub action_state: ActionState<A>,
    /// An [InputMap] component
    pub input_map: InputMap<A>,
}

// Cannot use derive(Default), as it forces an undesirable bound on our generics
impl<A: Actionlike> Default for InputManagerBundle<A> {
    fn default() -> Self {
        Self {
            action_state: ActionState::default(),
            input_map: InputMap::default(),
        }
    }
}
