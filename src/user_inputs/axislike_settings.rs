//! Utilities for handling axis-like input settings.
//!
//! This module provides a set of tools for managing settings related to axis-like inputs,
//! commonly used in applications such as mouse motion, game controllers, and joysticks.
//!
//! # Deadzones
//!
//! Deadzones are regions around the center of the input range where no action is taken.
//! This helps to eliminate small fluctuations or inaccuracies in input devices,
//! providing smoother and more precise control.
//!
//! # Sensitivity
//!
//! Sensitivity adjustments allow users to fine-tune the responsiveness of input devices.
//! By scaling input values, sensitivity settings can affect the rate at which changes in input are reflected in the output.
//!
//! # Inversion
//!
//! Input inversion allows for changing the directionality of input devices.
//! For example, inverting the input axis of a joystick or mouse can reverse the direction of movement.

use bevy::prelude::Reflect;
use bevy::utils::FloatOrd;
use serde::{Deserialize, Serialize};

/// Settings for single-axis inputs.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SingleAxisSettings {
    /// Sensitivity of the input.
    sensitivity: f32,

    /// The [`SingleAxisDeadzone`] settings for the input.
    deadzone: SingleAxisDeadzone,

    /// The inversion factor of the input.
    ///
    /// Using an `f32` here instead of a `bool` reduces performance cost,
    /// directly indicating inversion status without conditionals.
    ///
    /// # Values
    ///
    /// - `1.0` means the input isn't inverted
    /// - `-1.0` means the input is inverted
    inversion_factor: f32,
}

impl SingleAxisSettings {
    /// The default [`SingleAxisSettings`].
    ///
    /// - Sensitivity: `1.0`
    /// - Deadzone: Default deadzone (excludes input values within the range [-0.1, 0.1])
    /// - Inversion: Not inverted
    pub const DEFAULT: Self = Self {
        sensitivity: 1.0,
        deadzone: SingleAxisDeadzone::DEFAULT,
        inversion_factor: 1.0,
    };

    /// The inverted [`SingleAxisSettings`] with default other settings.
    ///
    /// - Sensitivity: `1.0`
    /// - Deadzone: Default deadzone (excludes input values within the range [-0.1, 0.1])
    /// - Inversion: Inverted
    pub const DEFAULT_INVERTED: Self = Self {
        sensitivity: 1.0,
        deadzone: SingleAxisDeadzone::DEFAULT,
        inversion_factor: -1.0,
    };

    /// The default [`SingleAxisSettings`] with a zero deadzone.
    ///
    /// - Sensitivity: `1.0`
    /// - Deadzone: Zero deadzone (excludes only the zeroes)
    /// - Inversion: Not inverted
    pub const ZERO_DEADZONE: Self = Self {
        sensitivity: 1.0,
        deadzone: SingleAxisDeadzone::ZERO,
        inversion_factor: 1.0,
    };

    /// The inverted [`SingleAxisSettings`] with a zero deadzone.
    ///
    /// - Sensitivity: `1.0`
    /// - Deadzone: Zero deadzone (excludes only the zeroes)
    /// - Inversion: Inverted
    pub const ZERO_DEADZONE_INVERTED: Self = Self {
        sensitivity: 1.0,
        deadzone: SingleAxisDeadzone::ZERO,
        inversion_factor: -1.0,
    };

    /// Creates a new [`SingleAxisSettings`] with the given settings.
    pub const fn new(sensitivity: f32, deadzone: SingleAxisDeadzone) -> Self {
        Self {
            deadzone,
            sensitivity,
            inversion_factor: 1.0,
        }
    }

    /// Creates a new [`SingleAxisSettings`] with the given sensitivity.
    pub const fn with_sensitivity(sensitivity: f32) -> Self {
        Self::new(sensitivity, SingleAxisDeadzone::DEFAULT)
    }

    /// Creates a new [`SingleAxisSettings`] with the given deadzone.
    pub const fn with_deadzone(deadzone: SingleAxisDeadzone) -> Self {
        Self::new(1.0, deadzone)
    }

    /// Creates a new [`SingleAxisSettings`] with only negative values being filtered.
    ///
    /// # Arguments
    ///
    /// - `negative_max` - The maximum limit for negative values.
    pub fn with_negative_only(negative_max: f32) -> Self {
        let deadzone = SingleAxisDeadzone::negative_only(negative_max);
        Self::new(1.0, deadzone)
    }

    /// Creates a new [`SingleAxisSettings`] with only positive values being filtered.
    ///
    /// # Arguments
    ///
    /// - `positive_min` - The minimum limit for positive values.
    pub fn with_positive_only(positive_min: f32) -> Self {
        let deadzone = SingleAxisDeadzone::positive_only(positive_min);
        Self::new(1.0, deadzone)
    }

    /// Returns a new [`SingleAxisSettings`] with inversion applied.
    pub fn with_inverted(self) -> SingleAxisSettings {
        Self {
            sensitivity: self.sensitivity,
            deadzone: self.deadzone,
            inversion_factor: -self.inversion_factor,
        }
    }

    /// Returns the input value after applying these settings.
    pub fn apply_settings(&self, input_value: f32) -> f32 {
        let deadzone_value = self.deadzone.apply_deadzone(input_value);

        deadzone_value * self.sensitivity * self.inversion_factor
    }

    /// Returns the input value after applying these settings without the deadzone.
    pub fn apply_settings_without_deadzone(&self, input_value: f32) -> f32 {
        input_value * self.sensitivity * self.inversion_factor
    }
}

/// Deadzone settings for single-axis inputs.
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct SingleAxisDeadzone {
    /// The maximum limit for negative values.
    negative_max: f32,

    /// The minimum limit for positive values.
    positive_min: f32,

    /// The width of the deadzone around the negative axis.
    ///
    /// This value represents the absolute value of `negative_max`,
    /// reducing the performance cost of using `abs` during value computation.
    negative_deadzone_width: f32,
}

impl SingleAxisDeadzone {
    /// The default deadzone with a small offset to filter out near-zero input values.
    ///
    /// This deadzone excludes input values within the range `[-0.1, 0.1]`.
    pub const DEFAULT: Self = Self {
        negative_max: -0.1,
        positive_min: 0.1,
        negative_deadzone_width: 0.1,
    };

    /// The deadzone that only filters out the zeroes.
    ///
    /// This deadzone doesn't filter out near-zero negative or positive values.
    pub const ZERO: Self = Self {
        negative_max: 0.0,
        positive_min: 0.0,
        negative_deadzone_width: 0.0,
    };

    /// Creates a new [`SingleAxisDeadzone`] to filter out input values within the range `[negative_max, positive_min]`.
    ///
    /// # Arguments
    ///
    /// - `negative_max` - The maximum limit for negative values, clamped to `0.0` if greater than `0.0`.
    /// - `positive_min` - The minimum limit for positive values, clamped to `0.0` if less than `0.0`.
    pub fn new(negative_max: f32, positive_min: f32) -> Self {
        Self {
            negative_max: negative_max.min(0.0),
            positive_min: positive_min.max(0.0),
            negative_deadzone_width: negative_max.abs(),
        }
    }

    /// Creates a new [`SingleAxisDeadzone`] with only negative values being filtered.
    ///
    /// # Arguments
    ///
    /// - `negative_max` - The maximum limit for negative values, clamped to `0.0` if greater than `0.0`.
    pub fn negative_only(negative_max: f32) -> Self {
        Self {
            negative_max: negative_max.min(0.0),
            positive_min: f32::MAX,
            negative_deadzone_width: negative_max.abs(),
        }
    }

    /// Creates a new [`SingleAxisDeadzone`] with only positive values being filtered.
    ///
    /// # Arguments
    ///
    /// - `positive_min` - The minimum limit for negative values, clamped to `0.0` if less than `0.0`.
    pub fn positive_only(positive_min: f32) -> Self {
        Self {
            negative_max: f32::MIN,
            positive_min: positive_min.max(0.0),
            negative_deadzone_width: f32::MAX,
        }
    }

    /// Returns the input value after applying the deadzone.
    ///
    /// This function calculates the deadzone width based on the input value and the deadzone settings.
    /// If the input value falls within the deadzone range, it returns `0.0`.
    /// Otherwise, it normalizes the input value into the range `[-1.0, 1.0]` by subtracting the deadzone width.
    ///
    /// # Panics
    ///
    /// Panic if both the negative and positive deadzone ranges are active, must never be reached.
    ///
    /// If this happens, you might be exploring the quantum realm!
    /// Consider offering your computer a cup of coffee and politely asking for a less mysterious explanation.
    pub fn apply_deadzone(&self, input_value: f32) -> f32 {
        let is_negative_active = self.negative_max > input_value;
        let is_positive_active = self.positive_min < input_value;

        let deadzone_width = match (is_negative_active, is_positive_active) {
            // The input value is within the deadzone and falls back to `0.0`
            (false, false) => return 0.0,
            // The input value is outside the negative deadzone range
            (true, false) => self.negative_deadzone_width,
            // The input value is outside the positive deadzone range
            (false, true) => self.positive_min,
            // This case must never be reached.
            // Unless you've discovered the elusive quantum deadzone!
            // Please check your quantum computer and contact the Rust Team.
            (true, true) => unreachable!("Quantum deadzone detected!"),
        };

        // Normalize the input value into the range [-1.0, 1.0]
        input_value.signum() * (input_value.abs() - deadzone_width) / (1.0 - deadzone_width)
    }
}

// Unfortunately, Rust doesn't let us automatically derive `Eq` and `Hash` for `f32`.
// It's like teaching a fish to ride a bike – a bit nonsensical!
// But if that fish really wants to pedal, we'll make it work.
// So here we are, showing Rust who's boss!

impl Eq for SingleAxisSettings {}

impl std::hash::Hash for SingleAxisSettings {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        FloatOrd(self.sensitivity).hash(state);
        self.deadzone.hash(state);
        FloatOrd(self.inversion_factor).hash(state);
    }
}

impl Eq for SingleAxisDeadzone {}

impl std::hash::Hash for SingleAxisDeadzone {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        FloatOrd(self.negative_max).hash(state);
        FloatOrd(self.positive_min).hash(state);
        FloatOrd(self.negative_deadzone_width).hash(state);
    }
}
