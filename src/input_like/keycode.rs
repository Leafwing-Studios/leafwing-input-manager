use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::input::Input;
use bevy::prelude::{KeyCode, World};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

impl ButtonLike for KeyCode {
    fn input_pressed(&self, world: &World) -> bool {
        world.resource::<Input<KeyCode>>().pressed(*self)
    }
}

impl SingleAxisLike for KeyCode {
    fn input_value(&self, world: &World) -> f32 {
        if self.input_pressed(world) {
            1.0
        } else {
            0.0
        }
    }
}

impl InputLikeObject for KeyCode {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        None
    }

    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike> {
        None
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

impl<'a> InputLike<'a> for KeyCode {}

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

impl ButtonLike for Modifier {
    fn input_pressed(&self, world: &World) -> bool {
        world
            .resource::<Input<KeyCode>>()
            .any_just_pressed(self.key_codes())
    }
}

impl InputLikeObject for Modifier {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        None
    }

    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike> {
        None
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
