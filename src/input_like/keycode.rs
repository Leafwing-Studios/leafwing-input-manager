use crate::axislike::DualAxisData;
use crate::input_like::{AxisLike, ButtonLike, InputLike, InputLikeObject};
use crate::input_streams::InputStreams;
use bevy::input::Input;
use bevy::prelude::{KeyCode, World};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

impl ButtonLike for KeyCode {}

impl InputLikeObject for KeyCode {
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        if let Some(other) = other.as_reflect().downcast_ref::<KeyCode>() {
            return self == other;
        }
        false
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        Some(Box::new(*self))
    }

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn raw_inputs(&self) -> Vec<Box<(dyn InputLikeObject)>> {
        vec![Box::new(*self)]
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        Box::new(*self)
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        <KeyCode as bevy::prelude::Reflect>::as_reflect(self)
    }
}

pub struct KeyCodeInputStreams {}

impl InputStreams for KeyCodeInputStreams {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool {
        input
            .as_reflect()
            .downcast_ref::<KeyCode>()
            .map(|input| world.resource::<Input<KeyCode>>().pressed(*input))
            .unwrap_or(false)
    }

    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32 {
        if self.input_pressed(world, input) {
            1.0
        } else {
            0.0
        }
    }

    fn input_axis_pair(
        &self,
        _world: &World,
        _input: &dyn InputLikeObject,
    ) -> Option<DualAxisData> {
        None
    }
}

impl<'a> InputLike<'a> for KeyCode {
    fn input_streams(_world: &World) -> Box<dyn InputStreams> {
        Box::new(KeyCodeInputStreams {})
    }
}

/// A keyboard modifier that combines two [`KeyCode`] values into one representation.
///
/// This buttonlike input is stored in [`InputKind`], and will be triggered whenever either of these buttons are pressed.
/// This will be decomposed into both values when converted into [`RawInputs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum Modifier {
    /// Corresponds to [`KeyCode::LAlt`] and [`KeyCode::RAlt`].
    Alt,
    /// Corresponds to [`KeyCode::LControl`] and [`KeyCode::RControl`].
    Control,
    /// The key that makes letters capitalized, corresponding to [`KeyCode::LShift`] and [`KeyCode::RShift`]
    Shift,
    /// The OS or "Windows" key, corresponding to [`KeyCode::LWin`] and [`KeyCode::RWin`].
    Win,
}

impl Modifier {
    /// Returns the pair of [`KeyCode`] values associated with this modifier.
    ///
    /// The left variant will always be in the first position, and the right variant is always in the second position.
    #[inline]
    pub fn key_codes(self) -> [KeyCode; 2] {
        match self {
            Modifier::Alt => [KeyCode::LAlt, KeyCode::RAlt],
            Modifier::Control => [KeyCode::LControl, KeyCode::RControl],
            Modifier::Shift => [KeyCode::LShift, KeyCode::RShift],
            Modifier::Win => [KeyCode::LWin, KeyCode::RWin],
        }
    }
}

impl ButtonLike for Modifier {}

impl InputLikeObject for Modifier {
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        if let Some(other) = other.as_reflect().downcast_ref::<Modifier>() {
            return self == other;
        }
        false
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        Some(Box::new(*self))
    }

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
        None
    }

    fn len(&self) -> usize {
        1
    }

    fn raw_inputs(&self) -> Vec<Box<(dyn InputLikeObject)>> {
        vec![Box::new(*self)]
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        Box::new(*self)
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }
}
