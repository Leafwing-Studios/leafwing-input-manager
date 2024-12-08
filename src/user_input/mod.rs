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
//! ### Virtual Axial Controls
//!
//! - [`VirtualAxis`]: Create a virtual axis control from two buttons.
//!
//! - [`VirtualDPad`]: Create a virtual dual-axis control from four buttons.
//!
//! - [`VirtualDPad3D`]: Create a virtual triple-axis control from six buttons.
//!
//! ### Chords
//!
//! - [`ButtonlikeChord`]: A combined input that groups multiple [`Buttonlike`]s together,
//!   allowing you to define complex input combinations like hotkeys, shortcuts, and macros.
//!
//! - [`AxislikeChord`]: A combined input that groups a [`Buttonlike`] and an [`Axislike`] together,
//!   allowing you to only read the dual axis data when the button is pressed.
//!
//! - [`DualAxislikeChord`]: A combined input that groups a [`Buttonlike`] and a [`DualAxislike`] together,
//!   allowing you to only read the dual axis data when the button is pressed.
//!
//! - [`TripleAxislikeChord`]: A combined input that groups a [`Buttonlike`] and a [`TripleAxislike`] together,
//!   allowing you to only read the dual axis data when the button is pressed.
//!
//! [`GamepadAxisType`]: bevy::prelude::GamepadAxisType
//! [`GamepadButtonType`]: bevy::prelude::GamepadButtonType
//! [`KeyCode`]: bevy::prelude::KeyCode
//! [`MouseButton`]: bevy::prelude::MouseButton

use std::fmt::Debug;

use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, World};
use bevy::reflect::{erased_serde, Reflect};
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::Serialize;
use updating::CentralInputStore;

use crate::clashing_inputs::BasicInputs;
use crate::InputControlKind;

pub use self::chord::*;
#[cfg(feature = "gamepad")]
pub use self::gamepad::*;
#[cfg(feature = "keyboard")]
pub use self::keyboard::*;
#[cfg(feature = "mouse")]
pub use self::mouse::*;
pub use self::trait_serde::RegisterUserInput;
pub use self::virtual_axial::*;

pub mod chord;
#[cfg(feature = "gamepad")]
pub mod gamepad;
#[cfg(feature = "keyboard")]
pub mod keyboard;
#[cfg(feature = "mouse")]
pub mod mouse;
pub mod testing_utils;
mod trait_reflection;
mod trait_serde;
pub mod updating;
pub mod virtual_axial;

/// A trait for defining the behavior expected from different user input sources.
pub trait UserInput: Send + Sync + Debug {
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
}

/// A trait used for buttonlike user inputs, which can be pressed or released
/// with a value for how much they are pressed.
pub trait Buttonlike:
    UserInput + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Checks if the input is currently active.
    fn pressed(&self, input_store: &CentralInputStore, gamepad: Entity) -> bool;

    /// Checks if the input is currently inactive.
    fn released(&self, input_store: &CentralInputStore, gamepad: Entity) -> bool {
        !self.pressed(input_store, gamepad)
    }

    /// Gets the current value of the button as an `f32`.
    ///
    /// The returned value should be between `0.0` and `1.0`,
    /// with `0.0` representing the input being fully released
    /// and `1.0` representing the input being fully pressed.
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32 {
        f32::from(self.pressed(input_store, gamepad))
    }

    /// Simulates a press of the buttonlike input by sending the appropriate event.
    ///
    /// This method defaults to calling [`Buttonlike::press_as_gamepad`] if not overridden,
    /// as is the case for gamepad-reliant inputs.
    fn press(&self, world: &mut World) {
        self.press_as_gamepad(world, None);
    }

    /// Simulate a press of the buttonlike input, pretending to be the provided [`Gamepad`].
    ///
    /// This method defaults to calling [`Buttonlike::press`] if not overridden,
    /// as is the case for things like mouse buttons and keyboard keys.
    ///
    /// Use [`find_gamepad`] inside of this method to search for a gamepad to press the button on
    /// if the provided gamepad is `None`.
    fn press_as_gamepad(&self, world: &mut World, _gamepad: Option<Entity>) {
        self.press(world);
    }

    /// Simulates a release of the buttonlike input by sending the appropriate event.
    ///
    /// This method defaults to calling [`Buttonlike::release_as_gamepad`] if not overridden,
    /// as is the case for gamepad-reliant inputs.
    fn release(&self, world: &mut World) {
        self.release_as_gamepad(world, None);
    }

    /// Simulate a release of the buttonlike input, pretending to be the provided [`Gamepad`].
    ///
    /// This method defaults to calling [`Buttonlike::release`] if not overridden,
    /// as is the case for things like mouse buttons and keyboard keys.
    ///
    /// Use [`find_gamepad`] inside of this method to search for a gamepad to press the button on
    /// if the provided gamepad is `None`.
    fn release_as_gamepad(&self, world: &mut World, _gamepad: Option<Entity>) {
        self.release(world);
    }

    /// Simulate a value change of the buttonlike input by sending the appropriate event.
    ///
    /// This method defaults to calling [`Buttonlike::set_value_as_gamepad`] if not overridden,
    /// as is the case for gamepad-reliant inputs.
    ///
    /// Also updates the state of the button based on the `value`:
    /// - If `value > 0.0`, the button will be pressed.
    /// - If `value <= 0.0`, the button will be released.
    fn set_value(&self, world: &mut World, value: f32) {
        self.set_value_as_gamepad(world, value, None);
    }

    /// Simulate a value change of the buttonlike input, pretending to be the provided [`Gamepad`].
    ///
    /// This method defaults to calling [`Buttonlike::set_value`] if not overridden,
    /// as is the case for things like a mouse wheel.
    ///
    /// Also updates the state of the button based on the `value`:
    /// - If `value > 0.0`, the button will be pressed.
    /// - If `value <= 0.0`, the button will be released.
    ///
    /// Use [`find_gamepad`] inside of this method to search for a gamepad to press the button on
    /// if the provided gamepad is `None`.
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, _gamepad: Option<Entity>) {
        self.set_value(world, value);
    }
}

/// A trait used for axis-like user inputs, which provide a continuous value.
pub trait Axislike:
    UserInput + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Gets the current value of the input as an `f32`.
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32;

    /// Simulate an axis-like input by sending the appropriate event.
    ///
    /// This method defaults to calling [`Axislike::set_value_as_gamepad`] if not overridden,
    /// as is the case for gamepad-reliant inputs.
    fn set_value(&self, world: &mut World, value: f32) {
        self.set_value_as_gamepad(world, value, None);
    }

    /// Simulate an axis-like input, pretending to be the provided [`Gamepad`].
    ///
    /// This method defaults to calling [`Axislike::set_value`] if not overridden,
    /// as is the case for things like a mouse wheel.
    ///
    /// Use [`find_gamepad`] inside of this method to search for a gamepad to press the button on
    /// if the provided gamepad is `None`.
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, _gamepad: Option<Entity>) {
        self.set_value(world, value);
    }
}

/// A trait used for dual-axis-like user inputs, which provide separate X and Y values.
pub trait DualAxislike:
    UserInput + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Gets the values of this input along the X and Y axes (if applicable).
    fn axis_pair(&self, input_store: &CentralInputStore, gamepad: Entity) -> Vec2;

    /// Simulate a dual-axis-like input by sending the appropriate event.
    ///
    /// This method defaults to calling [`DualAxislike::set_axis_pair_as_gamepad`] if not overridden,
    /// as is the case for gamepad-reliant inputs.
    fn set_axis_pair(&self, world: &mut World, value: Vec2) {
        self.set_axis_pair_as_gamepad(world, value, None);
    }

    /// Simulate a dual-axis-like input, pretending to be the provided [`Gamepad`].
    ///
    /// This method defaults to calling [`DualAxislike::set_axis_pair`] if not overridden,
    /// as is the case for things like a mouse wheel.
    ///
    /// Use [`find_gamepad`] inside of this method to search for a gamepad to press the button on
    /// if the provided gamepad is `None`.
    fn set_axis_pair_as_gamepad(&self, world: &mut World, value: Vec2, _gamepad: Option<Entity>) {
        self.set_axis_pair(world, value);
    }
}

/// A trait used for triple-axis-like user inputs, which provide separate X, Y, and Z values.
pub trait TripleAxislike:
    UserInput + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Gets the values of this input along the X, Y, and Z axes (if applicable).
    fn axis_triple(&self, input_store: &CentralInputStore, gamepad: Entity) -> Vec3;

    /// Simulate a triple-axis-like input by sending the appropriate event.
    ///
    /// This method defaults to calling [`TripleAxislike::set_axis_triple_as_gamepad`] if not overridden,
    /// as is the case for gamepad-reliant inputs.
    fn set_axis_triple(&self, world: &mut World, value: Vec3) {
        self.set_axis_triple_as_gamepad(world, value, None);
    }

    /// Simulate a triple-axis-like input, pretending to be the provided [`Gamepad`].
    ///
    /// This method defaults to calling [`TripleAxislike::set_axis_triple`] if not overridden,
    /// as is the case for things like a space mouse.
    ///
    /// Use [`find_gamepad`] inside of this method to search for a gamepad to press the button on
    /// if the provided gamepad is `None`.
    fn set_axis_triple_as_gamepad(&self, world: &mut World, value: Vec3, _gamepad: Option<Entity>) {
        self.set_axis_triple(world, value);
    }
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
    /// Wraps a [`TripleAxislike`] input.
    TripleAxis(Box<dyn TripleAxislike>),
}

impl UserInput for UserInputWrapper {
    #[track_caller]
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
            UserInputWrapper::TripleAxis(input) => {
                debug_assert!(input.kind() == InputControlKind::TripleAxis);
                input.kind()
            }
        }
    }

    fn decompose(&self) -> BasicInputs {
        match self {
            UserInputWrapper::Button(input) => input.decompose(),
            UserInputWrapper::Axis(input) => input.decompose(),
            UserInputWrapper::DualAxis(input) => input.decompose(),
            UserInputWrapper::TripleAxis(input) => input.decompose(),
        }
    }
}
