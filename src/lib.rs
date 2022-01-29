#![forbid(missing_docs)]
#![warn(clippy::doc_markdown)]

//! A simple but robust input-action manager for Bevy: intended to be useful both as a plugin and a helpful library.
//!
//! Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
//! which can be conveniently used in your game logic.
//!
//! The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on your player entity.
//! A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
//! and a single input can result in multiple actions being triggered, which can be handled contextually.
//!
//! This library seamlessly supports both single-player and local multiplayer games!
//! Simply add the `InputManagerBundle` to each controllable entity, and customize the `InputMap` appropriately.
//!
//! Of particular note: this plugin allows users to configure which state (i.e. `GameState::Playing`) it runs in.
//! No more characters wandering around while your menu is open!
//!
//! ## Features
//!
//! - Full keyboard, mouse and joystick support for button-like inputs.
//! - Effortlessly wire UI buttons to game state with one simple component!
//!   - When clicked, your button will send a virtual button press to the corresponding entity.
//! - Store all your input mappings in a single `InputMap` component
//!   - No more bespoke `Keybindings<KeyCode>`, `Keybindings<Gamepad>` headaches
//! - Look up your current input state in a single `ActionState` component
//!   - Easily check player statistics while reading input
//!   - That pesky maximum of 16 system parameters got you down? Say goodbye to that input handling mega-system
//! - Ergonomic insertion API that seamlessly blends multiple input types for you
//!   - `input_map.insert(Action::Jump, KeyCode::Space)` XOR `input_map.insert(Action::Jump, C)`? Why not both?
//! - Full support for arbitrary button combinations: chord your heart out.
//!   - `input_map.insert_chord(Action::Console, [KeyCode::LCtrl, KeyCode::Shift, KeyCode::C])`
//! - Create an arbitrary number of strongly typed disjoint action sets: decouple your camera and player state.
//! - Local multiplayer support: freely bind keys to distinct entities, rather than worrying about singular global state
//! - Leafwing Studio's trademark `#![forbid(missing_docs)]`
//!
//! ## Limitations
//!
//! - The `Button` enum only includes `KeyCode`, `MouseButton` and `GamepadButtonType`.
//!   - This is due to object-safety limitations on the types stored in `bevy::input::Input`
//!   - Please file an issue if you would like something more exotic!
//! - No built-in support for non-button input types (e.g. gestures or analog sticks).
//!   - All methods on `ActionState` are `pub`: it's designed to be hooked into and extended.
//! - Gamepads must be associated with each player by the app using this plugin: read from the `Gamepads` resource and use `InputMap::set_gamepad`.

use crate::action_state::ActionState;
use crate::input_map::InputMap;
use bevy::ecs::prelude::*;
use core::hash::Hash;
use std::marker::PhantomData;

pub mod action_state;
pub mod clashing_inputs;
mod display_impl;
pub mod input_map;
mod input_mocking;
// Re-export this at the root level
pub use input_mocking::MockInput;
pub mod plugin;
pub mod systems;
pub mod user_input;

// Importing the derive macro
pub use leafwing_input_manager_macros::Actionlike;

/// Everything you need to get started
pub mod prelude {
    pub use crate::action_state::{ActionState, ActionStateDriver};
    pub use crate::clashing_inputs::ClashStrategy;
    pub use crate::input_map::InputMap;
    pub use crate::user_input::UserInput;

    pub use crate::plugin::InputManagerPlugin;
    pub use crate::{Actionlike, InputManagerBundle};
}

/// Allows a type to be used as a gameplay action in an input-agnostic fashion
///
/// Actions serve as "virtual buttons", cleanly abstracting over messy, customizable inputs
/// in a way that can be easily consumed by your game logic.
///
/// This trait should be implemented on the `A` type that you want to pass into [`InputManagerPlugin`]
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
pub trait Actionlike: Send + Sync + Copy + Eq + Hash + 'static {
    /// Iterates over the possible actions in the order they were defined
    fn iter() -> ActionIter<Self> {
        ActionIter::default()
    }

    /// Returns the default value for the action stored at the provided index if it exists
    ///
    /// This is mostly used internally, to enable space-efficient iteration.
    fn get_at(index: usize) -> Option<Self>;
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

impl<A: Actionlike> ExactSizeIterator for ActionIter<A> {}

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
/// Use with [`InputManagerPlugin`], providing the same enum type to both.
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
