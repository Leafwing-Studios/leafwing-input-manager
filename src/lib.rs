#![forbid(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use crate::action_state::ActionState;
use crate::input_map::InputMap;
use bevy::ecs::prelude::*;
use bevy::reflect::{FromReflect, Reflect, TypePath};
use std::hash::Hash;
use std::marker::PhantomData;

pub mod action_state;
pub mod axislike;
pub mod buttonlike;
pub mod clashing_inputs;
pub mod common_conditions;
mod display_impl;
pub mod dynamic_action;
pub mod errors;
pub mod input_map;
pub mod input_mocking;
pub mod input_streams;
pub mod orientation;
pub mod plugin;
pub mod press_scheduler;
pub mod scan_codes;
pub mod systems;
pub mod user_input;

// Importing the derive macro
pub use leafwing_input_manager_macros::Actionlike;

/// Everything you need to get started
pub mod prelude {
    pub use crate::action_state::{ActionState, ActionStateDriver};
    pub use crate::axislike::{
        DeadZoneShape, DualAxis, MouseWheelAxisType, SingleAxis, VirtualDPad,
    };
    pub use crate::buttonlike::MouseWheelDirection;
    pub use crate::clashing_inputs::ClashStrategy;
    pub use crate::input_map::InputMap;
    #[cfg(feature = "ui")]
    pub use crate::input_mocking::MockUIInteraction;
    pub use crate::input_mocking::{MockInput, QueryInput};
    pub use crate::scan_codes::QwertyScanCode;
    pub use crate::user_input::{Modifier, UserInput};

    pub use crate::plugin::InputManagerPlugin;
    pub use crate::plugin::ToggleActions;
    pub use crate::{Actionlike, InputManagerBundle};
}

/// Allows a type to be used as a gameplay action in an input-agnostic fashion
///
/// Actions are modelled as "virtual buttons", cleanly abstracting over messy, customizable inputs
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
/// use bevy::prelude::Reflect;
/// use leafwing_input_manager::Actionlike;
///
/// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
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
pub trait Actionlike:
    Eq + Hash + Send + Sync + Clone + Hash + Reflect + TypePath + FromReflect + 'static
{
    /// The number of variants of this action type
    fn n_variants() -> usize;

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
/// Created by calling [`Actionlike::variants()`].
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
        A::n_variants()
    }
}

// We can't derive this, because otherwise it won't work when A is not default
impl<A: Actionlike> Default for ActionIter<A> {
    fn default() -> Self {
        ActionIter {
            index: 0,
            _phantom: PhantomData,
        }
    }
}

/// This [`Bundle`] allows entities to collect and interpret inputs from across input sources
///
/// Use with [`InputManagerPlugin`](crate::plugin::InputManagerPlugin), providing the same enum type to both.
#[derive(Bundle)]
pub struct InputManagerBundle<A: Actionlike> {
    /// An [`ActionState`] component
    pub action_state: ActionState<A>,
    /// An [`InputMap`] component
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
