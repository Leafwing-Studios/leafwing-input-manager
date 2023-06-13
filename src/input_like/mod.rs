//! Helpful abstractions over user inputs of all sorts

pub mod chords;
pub mod dual_axis;
pub mod gamepad_axis;
pub mod gamepad_axis_type;
pub mod gamepad_button;
pub mod gamepad_button_type;
pub mod keycode;
pub mod mouse_button;
pub mod mouse_wheel;
pub mod scancode;
pub mod virtual_dpad;

use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode, mouse::MouseButton};
use std::fmt::Debug;

use bevy::prelude::{Reflect, ScanCode, World};
use serde::{Deserialize, Serialize, Serializer};

use crate::axislike::DualAxisData;
use crate::input_like::keycode::Modifier;
use crate::input_like::mouse_wheel::MouseWheelDirection;
use crate::scan_codes::QwertyScanCode;
use crate::{axislike::SingleAxis, buttonlike::MouseMotionDirection};

pub trait InputLike<'a>: InputLikeObject + Deserialize<'a> + Clone + Eq {}

/// This trait is the
/// [object safe](https://doc.rust-lang.org/reference/items/traits.html#object-safety) part of
/// [`InputLike`], which is how they are stored in [`InputMap`].
#[allow(clippy::len_without_is_empty)]
pub trait InputLikeObject: Send + Sync + Debug {
    /// Does `self` clash with `other`?
    #[must_use]
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        if self.len() <= 1 && other.len() <= 1 {
            return false;
        }

        if self
            .as_reflect()
            .reflect_partial_eq(other.as_reflect())
            .unwrap_or_default()
        {
            return false;
        }

        self.raw_inputs().iter().any(|input| {
            for other in other.raw_inputs() {
                if other.eq(input) {
                    return true;
                }
            }
            false
        })
    }

    /// Returns [`ButtonLike`] if it is implemented.
    fn as_button(&self) -> Option<&dyn ButtonLike>;

    /// Returns [`SingleAxisLike`] if it is implemented.
    fn as_axis(&self) -> Option<&dyn SingleAxisLike>;

    /// Returns [`DualAxisLike`] if it is implemented.
    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike>;

    /// The number of logical inputs that make up the [`UserInput`].
    ///
    /// TODO: Update this
    /// - A [`Single`][UserInput::Single] input returns 1
    /// - A [`Chord`][UserInput::Chord] returns the number of buttons in the chord
    /// - A [`VirtualDPad`][UserInput::VirtualDPad] returns 1
    fn len(&self) -> usize {
        1
    }

    /// Returns the raw inputs that make up this [`UserInput`]
    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>> {
        vec![self.clone_dyn()]
    }

    /// Enables [`Clone`]ing [`InputLikeObject`]s while keeping dynamic dispatch support.
    fn clone_dyn(&self) -> Box<dyn InputLikeObject>;

    fn as_serialize(&self) -> &dyn erased_serde::Serialize;

    fn as_reflect(&self) -> &dyn Reflect;
}

impl Clone for Box<dyn InputLikeObject> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl Clone for Box<dyn ButtonLike> {
    fn clone(&self) -> Self {
        self.clone_button()
    }
}

impl PartialEq<Self> for dyn InputLikeObject + '_ {
    /// # Panics
    ///
    /// Panics If the underlying type does not support equality testing.
    fn eq(&self, other: &Self) -> bool {
        self.as_reflect().type_id() == other.as_reflect().type_id()
            && self
                .as_reflect()
                .reflect_partial_eq(other.as_reflect())
                .unwrap()
    }
}

impl Eq for dyn InputLikeObject {}

impl PartialEq for dyn ButtonLike {
    fn eq(&self, other: &Self) -> bool {
        self.as_reflect().type_id() == other.as_reflect().type_id()
            && self
                .as_reflect()
                .reflect_partial_eq(other.as_reflect())
                .unwrap()
    }
}

impl PartialEq for dyn SingleAxisLike {
    fn eq(&self, other: &Self) -> bool {
        self.as_reflect().type_id() == other.as_reflect().type_id()
            && self
                .as_reflect()
                .reflect_partial_eq(other.as_reflect())
                .unwrap()
    }
}

pub trait ButtonLike: InputLikeObject {
    fn input_pressed(&self, world: &World) -> bool;

    fn clone_button(&self) -> Box<dyn ButtonLike>;
}

pub trait SingleAxisLike: InputLikeObject + ButtonLike {
    fn input_value(&self, world: &World) -> f32 {
        if self.input_pressed(world) {
            1.0
        } else {
            0.0
        }
    }

    fn clone_axis(&self) -> Box<dyn SingleAxisLike>;
}

pub trait DualAxisLike: InputLikeObject {
    fn input_axis_pair(&self, world: &World) -> DualAxisData;
}

impl<T: InputLikeObject> From<T> for Box<dyn InputLikeObject> {
    fn from(input: T) -> Self {
        input.clone_dyn()
    }
}

impl<T: ButtonLike> From<T> for Box<dyn ButtonLike> {
    fn from(button: T) -> Self {
        button.clone_button()
    }
}

impl<T: SingleAxisLike> From<T> for Box<dyn SingleAxisLike> {
    fn from(input: T) -> Self {
        input.clone_axis()
    }
}

impl Serialize for dyn InputLikeObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_serialize().serialize(serializer)
    }
}

impl Serialize for dyn ButtonLike {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_serialize().serialize(serializer)
    }
}

impl Serialize for dyn SingleAxisLike {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_serialize().serialize(serializer)
    }
}

/// The different kinds of supported input bindings.
///
/// Commonly stored in the [`UserInput`] enum.
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputKind {
    /// A button on a gamepad
    GamepadButton(GamepadButtonType),
    /// A single axis of continuous motion
    SingleAxis(SingleAxis),
    /// A logical key on the keyboard.
    ///
    /// The actual (physical) key that has to be pressed depends on the keyboard layout.
    /// If you care about the position of the key rather than what it stands for,
    /// use [`InputKind::KeyLocation`] instead.
    Keyboard(KeyCode),
    /// The physical location of a key on the keyboard.
    ///
    /// The logical key which is emitted by this key depends on the keyboard layout.
    /// If you care about the output of the key rather than where it is positioned,
    /// use [`InputKind::Keyboard`] instead.
    KeyLocation(ScanCode),
    /// A keyboard modifier, like `Ctrl` or `Alt`, which doesn't care about which side it's on.
    Modifier(Modifier),
    /// A button on a mouse
    Mouse(MouseButton),
    /// A discretized mousewheel movement
    MouseWheel(MouseWheelDirection),
    /// A discretized mouse movement
    MouseMotion(MouseMotionDirection),
}

impl From<SingleAxis> for InputKind {
    fn from(input: SingleAxis) -> Self {
        InputKind::SingleAxis(input)
    }
}

impl From<GamepadButtonType> for InputKind {
    fn from(input: GamepadButtonType) -> Self {
        InputKind::GamepadButton(input)
    }
}

impl From<KeyCode> for InputKind {
    fn from(input: KeyCode) -> Self {
        InputKind::Keyboard(input)
    }
}

impl From<ScanCode> for InputKind {
    fn from(input: ScanCode) -> Self {
        InputKind::KeyLocation(input)
    }
}

impl From<QwertyScanCode> for InputKind {
    fn from(input: QwertyScanCode) -> Self {
        InputKind::KeyLocation(input.into())
    }
}

impl From<MouseButton> for InputKind {
    fn from(input: MouseButton) -> Self {
        InputKind::Mouse(input)
    }
}

impl From<MouseWheelDirection> for InputKind {
    fn from(input: MouseWheelDirection) -> Self {
        InputKind::MouseWheel(input)
    }
}

impl From<MouseMotionDirection> for InputKind {
    fn from(input: MouseMotionDirection) -> Self {
        InputKind::MouseMotion(input)
    }
}

impl From<Modifier> for InputKind {
    fn from(input: Modifier) -> Self {
        InputKind::Modifier(input)
    }
}
