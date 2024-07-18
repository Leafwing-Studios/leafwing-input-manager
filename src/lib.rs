#![forbid(missing_docs)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use crate::action_state::ActionState;
use crate::input_map::InputMap;
use bevy::ecs::prelude::*;
use bevy::reflect::{FromReflect, Reflect, TypePath};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

pub mod action_diff;
pub mod action_state;
pub mod axislike;
pub mod buttonlike;
pub mod clashing_inputs;
pub mod common_conditions;
pub mod input_map;
pub mod input_mocking;
pub mod input_processing;
pub mod input_streams;
pub mod plugin;
pub mod raw_inputs;
pub mod systems;

#[cfg(feature = "timing")]
pub mod timing;
pub mod typetag;
pub mod user_input;

// Importing the derive macro
pub use leafwing_input_manager_macros::Actionlike;

/// Everything you need to get started
pub mod prelude {
    pub use crate::InputControlKind;

    pub use crate::action_state::ActionState;
    pub use crate::clashing_inputs::ClashStrategy;
    pub use crate::input_map::InputMap;
    #[cfg(feature = "ui")]
    pub use crate::input_mocking::MockUIInteraction;
    pub use crate::input_mocking::{MockInput, QueryInput};
    pub use crate::input_processing::*;
    pub use crate::user_input::*;

    pub use crate::plugin::InputManagerPlugin;
    pub use crate::{Actionlike, InputManagerBundle};

    pub use leafwing_input_manager_macros::serde_typetag;
}

/// Allows a type to be used as a gameplay action in an input-agnostic fashion
///
/// Actions are modelled as "virtual buttons" (or axes), cleanly abstracting over messy, customizable inputs
/// in a way that can be easily consumed by your game logic.
///
/// This trait should be implemented on the `A` type that you want to pass into [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).
///
/// Generally, these types will be very small (often data-less) enums.
/// As a result, the APIs in this crate accept actions by value, rather than reference.
/// While `Copy` is not a required trait bound,
/// users are strongly encouraged to derive `Copy` on these enums whenever possible to improve ergonomics.
///
/// # Warning
///
/// The derive macro for this trait assumes that all actions are buttonlike.
/// If you have axislike or dual-axislike actions, you will need to implement this trait manually.
///
/// # Examples
/// ```rust
/// use bevy::prelude::Reflect;
/// use leafwing_input_manager::Actionlike;
///
/// #[derive(Actionlike, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
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
///
/// ```rust
/// use bevy::prelude::Reflect;
/// use leafwing_input_manager::Actionlike;
/// use leafwing_input_manager::InputControlKind;
///
/// #[derive(PartialEq, Debug, Eq, Clone, Copy, Hash, Reflect)]
/// enum PlayerAction {
///    // Movement
///    Movement,
///    // Abilities
///    Ability1,
///    Ability2,
///    Ability3,
///    Ability4,
///    Ultimate,
/// }
///
/// impl Actionlike for PlayerAction {
///     fn input_control_kind(&self) -> InputControlKind {
///         match self {
///            PlayerAction::Movement => InputControlKind::DualAxis,
///            _ => InputControlKind::Button,
///         }
///     }
/// }
/// ```
pub trait Actionlike:
    Debug + Eq + Hash + Send + Sync + Clone + Reflect + TypePath + FromReflect + 'static
{
    /// Returns the kind of input control this action represents: buttonlike, axislike, or dual-axislike.
    fn input_control_kind(&self) -> InputControlKind;
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

impl<A: Actionlike> InputManagerBundle<A> {
    /// Creates a [`InputManagerBundle`] with the given [`InputMap`].
    pub fn with_map(input_map: InputMap<A>) -> Self {
        Self {
            input_map,
            action_state: ActionState::default(),
        }
    }
}

/// Classifies [`UserInput`](crate::user_input::UserInput)s and [`Actionlike`] actions based on their behavior (buttons, analog axes, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum InputControlKind {
    /// A single input with binary state (active or inactive), typically a button press (on or off).
    ///
    /// Corresponds to [`Buttonlike`](crate::user_input::Buttonlike)  inputs.
    Button,

    /// A single analog or digital input, often used for range controls like a thumb stick on a gamepad or mouse wheel,
    /// providing a value within a min-max range.
    ///
    /// Corresponds to [`Axislike`](crate::user_input::Axislike) inputs.
    Axis,

    /// A combination of two axis-like inputs, often used for directional controls like a D-pad on a gamepad,
    /// providing separate values for the X and Y axes.
    ///
    /// Corresponds to [`DualAxislike`](crate::user_input::DualAxislike) inputs.
    DualAxis,
}
