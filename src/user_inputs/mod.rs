//! Helpful abstractions over user inputs of all sorts.
//!
//! This module provides abstractions and utilities for defining and handling user inputs
//! across various input devices such as gamepads, keyboards, and mice.
//! It offers a unified interface for querying input values and states,
//! making it easier to manage and process user interactions within a Bevy application.
//!
//! # Traits
//!
//! - [`UserInput`]: A trait for defining a specific kind of user input.
//!   It provides methods for checking if the input is active,
//!   retrieving its current value, and detecting when it started or finished.
//!
//! # Modules
//!
//! ## General Input Settings
//!
//! - [`axislike_settings`]: Utilities for configuring axis-like input.
//!
//! ## General Inputs
//!
//! - [`gamepad_inputs`]: Utilities for handling gamepad inputs.
//! - [`keyboard_inputs`]: Utilities for handling keyboard inputs.
//! - [`mouse_inputs`]: Utilities for handling mouse inputs.
//!
//! ## Specific Inputs:
//!
//! - [`chord_inputs`]: A combination of buttons, pressed simultaneously.

use bevy::prelude::{Reflect, Vec2};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

use crate::input_streams::InputStreams;

pub mod axislike_settings;

pub mod gamepad_inputs;
pub mod keyboard_inputs;
pub mod mouse_inputs;

pub mod chord_inputs;

/// Allows defining a specific kind of user input.
pub trait UserInput<'a>:
    Send + Sync + Debug + Clone + PartialEq + Eq + Hash + Reflect + Serialize + Deserialize<'a>
{
    /// Checks if this input is currently active.
    fn is_active(&self, input_query: InputStreams<'a>) -> bool {
        self.value(input_query).is_some_and(|value| value != 0.0)
    }

    /// Retrieves the current value from input if available.
    fn value(&self, _input_query: InputStreams<'a>) -> Option<f32> {
        None
    }

    /// Retrieves the current two-dimensional values from input if available.
    fn pair_values(&self, _input_query: InputStreams<'a>) -> Option<Vec2> {
        None
    }

    /// Checks if this input is being active during the current tick.
    fn started(&self, _input_query: InputStreams<'a>) -> bool {
        false
    }

    /// Checks if this input is being inactive during the current tick.
    fn finished(&self, _input_query: InputStreams<'a>) -> bool {
        false
    }
}
