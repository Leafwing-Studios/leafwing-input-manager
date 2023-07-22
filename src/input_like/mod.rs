//! Helpful abstractions over user inputs of all sorts

pub mod chords;
pub mod dual_axis;
pub mod gamepad_axis;
pub mod gamepad_axis_type;
pub mod gamepad_button;
pub mod gamepad_button_type;
pub mod keycode;
pub mod mouse_button;
pub mod mouse_motion;
pub mod mouse_wheel;
pub mod scancode;
pub mod virtual_axis;
pub mod virtual_dpad;

use std::fmt::Debug;

use bevy::prelude::{Reflect, World};
use serde::{Deserialize, Serialize, Serializer};

use crate::axislike::DualAxisData;

pub trait InputLike<'a>: InputLikeObject + Deserialize<'a> + Clone + Eq {}

/// This trait is the
/// [object safe](https://doc.rust-lang.org/reference/items/traits.html#object-safety) part of
/// [`InputLike`], which is how they are stored in [`InputMap`](crate::InputMap).
#[allow(clippy::len_without_is_empty)]
pub trait InputLikeObject: Send + Sync + Debug {
    /// Returns [`ButtonLike`] if it is implemented.
    fn as_button(&self) -> Option<&dyn ButtonLike>;

    /// Returns [`SingleAxisLike`] if it is implemented.
    fn as_axis(&self) -> Option<&dyn SingleAxisLike>;

    /// Returns [`DualAxisLike`] if it is implemented.
    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike>;

    /// The number of logical inputs that make up the [`InputLike`](crate::input_like::InputLike).
    ///
    /// - A single input returns 1 (e.g. [`KeyCode`][bevy::prelude::KeyCode])
    /// - A [`Chord`][crate::prelude::Chord] returns the number of buttons in the chord
    /// - A [`VirtualDPad`][crate::prelude::VirtualDPad] returns 1
    fn len(&self) -> usize {
        1
    }

    /// Returns the raw inputs that make up this [`InputLike`](crate::input_like::InputLike)
    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>> {
        vec![self.clone_dyn()]
    }

    /// Enables [`Clone`]ing [`InputLikeObject`]s while keeping dynamic dispatch support.
    fn clone_dyn(&self) -> Box<dyn InputLikeObject>;

    fn as_serialize(&self) -> &dyn erased_serde::Serialize;

    fn as_reflect(&self) -> &dyn Reflect;

    /// Returns a string that can be used to display this input to the user.
    fn input_display(&self) -> String {
        format!("{:?}", self)
    }

    /// Returns true if other is a strict subset of this input, i.e other contains all the
    /// inputs in self and contains at least one more input.
    ///
    /// Override this if you need to change how clash handling detects if an input is a subset
    /// of this input. An example of this is [`VirtualDPad`](virtual_dpad::VirtualDPad), which
    /// considers each of it's inputs separately instead of a combination.
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::KeyCode::*;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// assert!(Chord::new(vec![A, B, C]).is_strict_subset(Chord::new(vec![A, B]).into()));
    /// assert!(!Chord::new(vec![A, B]).is_strict_subset(Chord::new(vec![A, B, C]).into()));
    /// assert!(Chord::new(vec![A, B]).is_strict_subset(A.into()));
    /// ```
    fn is_strict_subset(&self, other: Box<dyn InputLikeObject>) -> bool {
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

        other.raw_inputs().iter().all(|input| {
            self.raw_inputs()
                .iter()
                .any(|other_input| input.eq(other_input))
        })
    }
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
