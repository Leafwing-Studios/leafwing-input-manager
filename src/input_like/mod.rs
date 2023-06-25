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
/// [`InputLike`], which is how they are stored in [`InputMap`].
#[allow(clippy::len_without_is_empty)]
pub trait InputLikeObject: Send + Sync + Debug {
    /// Does `self` clash with `other`?
    ///
    /// Inputs "clash" if and only if one [`UserInput`] is a strict subset of the other.
    /// By example:
    ///
    /// - `S` and `W`: does not clash
    /// - `LControl + S` and `S`: clashes
    /// - `S` and `S`: does not clash
    /// - `LControl + S` and ` LAlt + S`: clashes
    /// - `LControl + S`, `LAlt + S` and `LControl + LAlt + S`: clashes
    #[must_use]
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        // Single inputs don't clash with other single inputs
        if self.len() <= 1 && other.len() <= 1 {
            return false;
        }

        // If the inputs are equal, they don't clash
        if self
            .as_reflect()
            .reflect_partial_eq(other.as_reflect())
            .unwrap_or_default()
        {
            return false;
        }

        let a_is_subset_of_b = self.raw_inputs().iter().all(|input| {
            other
                .raw_inputs()
                .iter()
                .any(|other_input| input.eq(other_input))
        });
        let b_is_subset_of_a = other.raw_inputs().iter().all(|input| {
            self.raw_inputs()
                .iter()
                .any(|other_input| input.eq(other_input))
        });
        if self.as_reflect().type_name().contains("Virtual")
            | other.as_reflect().type_name().contains("Virtual")
        {
            println!(
                "{} clashes {} = a_is_subset_of_b {} b_is_subset_of_a {} raw_inputs_a {:?} raw_inputs_b {:?}",
                self.as_reflect().type_name(),
                other.as_reflect().type_name(),
                a_is_subset_of_b,
                b_is_subset_of_a,
                self.raw_inputs(),
                other.raw_inputs()
            );
        }
        a_is_subset_of_b || b_is_subset_of_a
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

    /// Returns a string that can be used to display this input to the user.
    fn input_display(&self) -> String {
        format!("{:?}", self)
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
