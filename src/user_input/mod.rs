//! Helpful abstractions over user inputs of all sorts.
//!
//! This module simplifies user input handling in Bevy applications
//! by providing abstractions and utilities for various input devices
//! like gamepads, keyboards, and mice. It offers a unified interface
//! for querying input values and states, reducing boilerplate code
//! and making user interactions easier to manage.
//!
//! The foundation of this module lies in the [`UserInput`] trait,
//! used to define the behavior expected from a specific user input source.
//!
//! Need something specific? You can also create your own inputs by implementing the trait for specific needs.
//!
//! Feel free to suggest additions to the built-in inputs if you have a common use case!
//!
//! ## Control Types
//!
//! [`UserInput`]s use the method [`UserInput::kind`] returning an [`InputControlKind`]
//! to classify the behavior of the input (buttons, analog axes, etc.).
//!
//! - [`InputControlKind::Button`]: Represents a digital input with an on/off state (e.g., button press).
//!   These inputs typically provide two values, typically `0.0` (inactive) and `1.0` (fully active).
//!
//! - [`InputControlKind::Axis`]: Represents an analog input (e.g., mouse wheel)
//!   with a continuous value typically ranging from `-1.0` (fully left/down) to `1.0` (fully right/up).
//!   Non-zero values are considered active.
//!
//! - [`InputControlKind::DualAxis`]: Represents a combination of two analog axes (e.g., thumb stick).
//!   These inputs provide separate X and Y values typically ranging from `-1.0` to `1.0`.
//!   Non-zero values are considered active.
//!
//! ## Basic Inputs
//!
//! [`UserInput`]s use the method [`UserInput::decompose`] returning a [`BasicInputs`]
//! used for clashing detection, see [clashing input check](crate::clashing_inputs) for details.
//!
//! ## Raw Input Events
//!
//! [`UserInput`]s use the method [`UserInput::raw_inputs`] returning a [`RawInputs`]
//! used for sending fake input events, see [input mocking](crate::input_mocking::MockInput) for details.
//!
//! ## Built-in Inputs
//!
//! ### Gamepad Inputs
//!
//! - Check gamepad button presses using Bevy's [`GamepadButtonType`] directly.
//! - Access physical sticks using [`GamepadStick`], [`GamepadControlAxis`], and [`GamepadControlDirection`].
//!
//! ### Keyboard Inputs
//!
//! - Check physical keys presses using Bevy's [`KeyCode`] directly.
//! - Use [`ModifierKey`] to check for either left or right modifier keys is pressed.
//!
//! ### Mouse Inputs
//!
//! - Check mouse buttons presses using Bevy's [`MouseButton`] directly.
//! - Track mouse motion with [`MouseMove`], [`MouseMoveAxis`], and [`MouseMoveDirection`].
//! - Capture mouse wheel events with [`MouseScroll`], [`MouseScrollAxis`], and [`MouseScrollDirection`].
//!
//! ### Complex Composition
//!
//! - Combine multiple inputs into a virtual button using [`ButtonlikeChord`].
//!   - Only active if all its inner inputs are active simultaneously.
//!   - Combine values from all inner single-axis inputs if available.
//!   - Retrieve values from the first encountered dual-axis input within the chord.
//!
//! - Create a virtual axis control:
//!   - [`GamepadVirtualAxis`] from two [`GamepadButtonType`]s.
//!   - [`KeyboardVirtualAxis`] from two [`KeyCode`]s.
//!
//! - Create a virtual directional pad (D-pad) for dual-axis control:
//!   - [`GamepadVirtualDPad`] from four [`GamepadButtonType`]s.
//!   - [`KeyboardVirtualDPad`] from four [`KeyCode`]s.
//!
//! [`GamepadAxisType`]: bevy::prelude::GamepadAxisType
//! [`GamepadButtonType`]: bevy::prelude::GamepadButtonType
//! [`KeyCode`]: bevy::prelude::KeyCode
//! [`MouseButton`]: bevy::prelude::MouseButton

use std::fmt::Debug;

use bevy::math::Vec2;
use bevy::reflect::{erased_serde, Reflect};
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::Serialize;

use crate::clashing_inputs::BasicInputs;
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::InputControlKind;

pub use self::chord::*;
pub use self::gamepad::*;
pub use self::keyboard::*;
pub use self::mouse::*;
pub use self::trait_serde::RegisterUserInput;

pub mod chord;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;
mod trait_reflection;
mod trait_serde;

/// A trait for defining the behavior expected from different user input sources.
///
/// Implementers of this trait should provide methods for accessing and processing user input data.
///
/// # Examples
///
/// ```rust
/// use std::hash::{Hash, Hasher};
/// use bevy::prelude::*;
/// use bevy::math::{Vec2, FloatOrd};
/// use serde::{Deserialize, Serialize};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::input_streams::InputStreams;
/// use leafwing_input_manager::axislike::{DualAxisType};
/// use leafwing_input_manager::raw_inputs::RawInputs;
/// use leafwing_input_manager::clashing_inputs::BasicInputs;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
/// pub struct MouseScrollAlwaysFiveOnYAxis;
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[serde_typetag]
/// impl UserInput for MouseScrollAlwaysFiveOnYAxis {
///     fn kind(&self) -> InputControlKind {
///         // Returns the kind of input this represents.
///         //
///         // In this case, it represents an axial input.
///         InputControlKind::Axis
///     }
///
///     fn decompose(&self) -> BasicInputs {
///         // Gets the most basic form of this input for clashing input detection.
///         //
///         // This input is not buttonlike, so it uses `None`.
///         BasicInputs::None
///     }
///
///     fn raw_inputs(&self) -> RawInputs {
///         // Defines the raw input events used for simulating this input.
///         //
///         // This input simulates a mouse scroll event on the Y-axis.
///         RawInputs::from_mouse_scroll_axes([DualAxisType::Y])
///     }
/// }
///
/// // Remember to register your input - it will ensure everything works smoothly!
/// let mut app = App::new();
/// app.register_user_input::<MouseScrollAlwaysFiveOnYAxis>();
/// ```
pub trait UserInput:
    Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Defines the kind of behavior that the input should be.
    fn kind(&self) -> InputControlKind;

    /// Returns the set of primitive inputs that make up this input.
    ///
    /// These inputs are used to detect clashes between different user inputs,
    /// and are stored in a [`BasicInputs`] for easy comparison.
    ///
    /// For inputs that represent a simple, atomic control,
    /// this method should always return a [`BasicInputs::Simple`] that only contains the input itself.
    fn decompose(&self) -> BasicInputs;

    /// Returns the raw input events that make up this input.
    ///
    /// Unlike [`UserInput::decompose`], which stores boxed user inputs,
    /// this method returns the raw input types.
    fn raw_inputs(&self) -> RawInputs;
}

/// A trait used for buttonlike user inputs, which can be pressed or released.
pub trait Buttonlike: UserInput {
    /// Checks if the input is currently active.
    fn pressed(&self, input_streams: &InputStreams) -> bool;

    /// Checks if the input is currently inactive.
    fn released(&self, input_streams: &InputStreams) -> bool {
        !self.pressed(input_streams)
    }
}

/// A trait used for axis-like user inputs, which provide a continuous value.
pub trait Axislike: UserInput {
    /// Gets the current value of the input as an `f32`.
    fn value(&self, input_streams: &InputStreams) -> f32;
}

/// A trait used for dual-axis-like user inputs, which provide separate X and Y values.
pub trait DualAxislike: UserInput {
    /// Gets the values of this input along the X and Y axes (if applicable).
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2;
}

/// A wrapper type to get around the lack of [trait upcasting coercion](https://github.com/rust-lang/rust/issues/65991).
///
/// To return a generic [`UserInput`] trait object from a function, you can use this wrapper type.

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum UserInputWrapper {
    /// Wraps a [`Buttonlike`] input.
    Button(Box<dyn Buttonlike>),
    /// Wraps an [`Axislike`] input.
    Axis(Box<dyn Axislike>),
    /// Wraps a [`DualAxislike`] input.
    DualAxis(Box<dyn DualAxislike>),
}

impl UserInput for UserInputWrapper {
    fn kind(&self) -> InputControlKind {
        match self {
            UserInputWrapper::Button(input) => {
                debug_assert!(input.kind() == InputControlKind::Button);
                input.kind()
            }
            UserInputWrapper::Axis(input) => {
                debug_assert!(input.kind() == InputControlKind::Axis);
                input.kind()
            }
            UserInputWrapper::DualAxis(input) => {
                debug_assert!(input.kind() == InputControlKind::DualAxis);
                input.kind()
            }
        }
    }

    fn decompose(&self) -> BasicInputs {
        match self {
            UserInputWrapper::Button(input) => input.decompose(),
            UserInputWrapper::Axis(input) => input.decompose(),
            UserInputWrapper::DualAxis(input) => input.decompose(),
        }
    }

    fn raw_inputs(&self) -> RawInputs {
        match self {
            UserInputWrapper::Button(input) => input.raw_inputs(),
            UserInputWrapper::Axis(input) => input.raw_inputs(),
            UserInputWrapper::DualAxis(input) => input.raw_inputs(),
        }
    }
}
