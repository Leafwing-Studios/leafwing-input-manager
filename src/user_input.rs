//! Helpful abstractions over user inputs of all sorts

use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode, mouse::MouseButton};
use std::any::Any;
use std::fmt::Debug;

use bevy::prelude::{Reflect, ScanCode, World};
use bevy::reflect::FromType;
use serde::{Deserialize, Serialize, Serializer};

use crate::axislike::DualAxisData;
use crate::input_streams::InputStreams;
use crate::scan_codes::QwertyScanCode;
use crate::{
    axislike::{AxisType, DualAxis, SingleAxis},
    buttonlike::{MouseMotionDirection, MouseWheelDirection},
};

pub trait InputLike<'a>: InputLikeObject + Deserialize<'a> + Clone + Eq {
    fn input_streams(world: &World) -> Box<dyn InputStreams>;
}

#[derive(Clone)]
pub struct ReflectInputLike {
    pub input_streams: fn(&World) -> Box<dyn InputStreams>,
}

impl<'a, T: InputLike<'a>> FromType<T> for ReflectInputLike {
    fn from_type() -> Self {
        Self {
            input_streams: T::input_streams,
        }
    }
}

/// This trait is the
/// [object safe](https://doc.rust-lang.org/reference/items/traits.html#object-safety) part of
/// [`InputLike`], which is how they are stored in [`InputMap`].
#[allow(clippy::len_without_is_empty)]
pub trait InputLikeObject: Send + Sync + Debug {
    /// Does `self` clash with `other`?
    #[must_use]
    fn clashes(&self, other: &dyn InputLikeObject) -> bool;

    /// Returns [`ButtonLike`] if it is implemented.
    fn as_button(&self) -> Option<Box<dyn ButtonLike>>;

    /// Returns [`AxisLike`] if it is implemented.
    fn as_axis(&self) -> Option<Box<dyn AxisLike>>;

    /// The number of logical inputs that make up the [`UserInput`].
    ///
    /// TODO: Update this
    /// - A [`Single`][UserInput::Single] input returns 1
    /// - A [`Chord`][UserInput::Chord] returns the number of buttons in the chord
    /// - A [`VirtualDPad`][UserInput::VirtualDPad] returns 1
    fn len(&self) -> usize;

    /// Returns the raw inputs that make up this [`UserInput`]
    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>>;

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

impl PartialEq<Self> for dyn InputLikeObject {
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

pub trait ButtonLike: InputLikeObject {}

pub trait AxisLike: InputLikeObject {}

impl InputLikeObject for Box<dyn InputLikeObject> {
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        self.as_ref().clashes(other)
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        self.as_ref().as_button()
    }

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
        self.as_ref().as_axis()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn raw_inputs(&self) -> Vec<Box<(dyn InputLikeObject)>> {
        self.as_ref().raw_inputs()
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        self.as_ref().clone_dyn()
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self.as_ref().as_serialize()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self.as_ref().as_reflect()
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

impl InputLikeObject for KeyCode {
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        if let Some(other) = other.as_reflect().downcast_ref::<KeyCode>() {
            return self == other;
        }
        false
    }

    fn as_button(&self) -> Option<Box<dyn ButtonLike>> {
        todo!()
    }

    fn as_axis(&self) -> Option<Box<dyn AxisLike>> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn raw_inputs(&self) -> Vec<Box<(dyn InputLikeObject)>> {
        todo!()
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        todo!()
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        todo!()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        <KeyCode as bevy::prelude::Reflect>::as_reflect(self)
    }
}

pub struct KeyCodeInputStreams {}

impl<'a> InputStreams for KeyCodeInputStreams {
    fn input_pressed(&self, world: &World, input: &dyn InputLikeObject) -> bool {
        todo!()
    }

    fn input_value(&self, world: &World, input: &dyn InputLikeObject) -> f32 {
        todo!()
    }

    fn input_axis_pair(&self, world: &World, input: &dyn InputLikeObject) -> Option<DualAxisData> {
        todo!()
    }
}

impl<'a> InputLike<'a> for KeyCode {
    fn input_streams(world: &World) -> Box<dyn InputStreams> {
        Box::new(KeyCodeInputStreams {})
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
    /// Two paired axes of continuous motion
    DualAxis(DualAxis),
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

impl From<DualAxis> for InputKind {
    fn from(input: DualAxis) -> Self {
        InputKind::DualAxis(input)
    }
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

/// A keyboard modifier that combines two [`KeyCode`] values into one representation.
///
/// This buttonlike input is stored in [`InputKind`], and will be triggered whenever either of these buttons are pressed.
/// This will be decomposed into both values when converted into [`RawInputs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// The basic input events that make up a [`UserInput`].
///
/// Obtained by calling [`UserInput::raw_inputs()`].
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RawInputs {
    /// Logical keyboard keys.
    pub keycodes: Vec<KeyCode>,
    /// Physical key locations.
    pub scan_codes: Vec<ScanCode>,
    /// Mouse buttons
    pub mouse_buttons: Vec<MouseButton>,
    /// Discretized mouse wheel inputs
    pub mouse_wheel: Vec<MouseWheelDirection>,
    /// Discretized mouse motion inputs
    pub mouse_motion: Vec<MouseMotionDirection>,
    /// Gamepad buttons, independent of a [`Gamepad`](bevy::input::gamepad::Gamepad)
    pub gamepad_buttons: Vec<GamepadButtonType>,
    /// Axis-like data
    ///
    /// The `f32` stores the magnitude of the axis motion, and is only used for input mocking.
    pub axis_data: Vec<(AxisType, Option<f32>)>,
}

#[cfg(test)]
impl RawInputs {
    fn from_keycode(keycode: KeyCode) -> RawInputs {
        RawInputs {
            keycodes: vec![keycode],
            ..Default::default()
        }
    }

    fn from_mouse_button(mouse_button: MouseButton) -> RawInputs {
        RawInputs {
            mouse_buttons: vec![mouse_button],
            ..Default::default()
        }
    }

    fn from_gamepad_button(gamepad_button: GamepadButtonType) -> RawInputs {
        RawInputs {
            gamepad_buttons: vec![gamepad_button],
            ..Default::default()
        }
    }

    fn from_mouse_wheel(direction: MouseWheelDirection) -> RawInputs {
        RawInputs {
            mouse_wheel: vec![direction],
            ..Default::default()
        }
    }

    fn from_mouse_direction(direction: MouseMotionDirection) -> RawInputs {
        RawInputs {
            mouse_motion: vec![direction],
            ..Default::default()
        }
    }

    fn from_dual_axis(axis: DualAxis) -> RawInputs {
        RawInputs {
            axis_data: vec![
                (axis.x.axis_type, axis.x.value),
                (axis.y.axis_type, axis.y.value),
            ],
            ..Default::default()
        }
    }

    fn from_single_axis(axis: SingleAxis) -> RawInputs {
        RawInputs {
            axis_data: vec![(axis.axis_type, axis.value)],
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod raw_input_tests {
    use crate::{
        axislike::AxisType,
        user_input::{InputKind, RawInputs, UserInput},
    };

    #[test]
    fn simple_chord() {
        use bevy::input::gamepad::GamepadButtonType;

        let buttons = vec![GamepadButtonType::Start, GamepadButtonType::Select];
        let raw_inputs = UserInput::chord(buttons.clone()).raw_inputs();
        let expected = RawInputs {
            gamepad_buttons: buttons,
            ..Default::default()
        };

        assert_eq!(expected, raw_inputs);
    }

    #[test]
    fn mixed_chord() {
        use crate::axislike::SingleAxis;
        use bevy::input::gamepad::GamepadAxisType;
        use bevy::input::gamepad::GamepadButtonType;

        let chord = UserInput::chord([
            InputKind::GamepadButton(GamepadButtonType::Start),
            InputKind::SingleAxis(SingleAxis::symmetric(GamepadAxisType::LeftZ, 0.)),
        ]);

        let raw = chord.raw_inputs();
        let expected = RawInputs {
            gamepad_buttons: vec![GamepadButtonType::Start],
            axis_data: vec![(AxisType::Gamepad(GamepadAxisType::LeftZ), None)],
            ..Default::default()
        };

        assert_eq!(expected, raw);
    }

    mod gamepad {
        use crate::user_input::{RawInputs, UserInput};

        #[test]
        fn gamepad_button() {
            use bevy::input::gamepad::GamepadButtonType;

            let button = GamepadButtonType::Start;
            let expected = RawInputs::from_gamepad_button(button);
            let raw = UserInput::from(button).raw_inputs();
            assert_eq!(expected, raw);
        }

        #[test]
        fn single_gamepad_axis() {
            use crate::axislike::SingleAxis;
            use bevy::input::gamepad::GamepadAxisType;

            let direction = SingleAxis::from_value(GamepadAxisType::LeftStickX, 1.0);
            let expected = RawInputs::from_single_axis(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn dual_gamepad_axis() {
            use crate::axislike::DualAxis;
            use bevy::input::gamepad::GamepadAxisType;

            let direction = DualAxis::from_value(
                GamepadAxisType::LeftStickX,
                GamepadAxisType::LeftStickY,
                0.5,
                0.7,
            );
            let expected = RawInputs::from_dual_axis(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }
    }

    mod keyboard {
        use crate::user_input::{RawInputs, UserInput};

        #[test]
        fn keyboard_button() {
            use bevy::input::keyboard::KeyCode;

            let button = KeyCode::A;
            let expected = RawInputs::from_keycode(button);
            let raw = UserInput::from(button).raw_inputs();
            assert_eq!(expected, raw);
        }

        #[test]
        fn modifier_key_decomposes_into_both_inputs() {
            use crate::user_input::Modifier;
            use bevy::input::keyboard::KeyCode;

            let input = UserInput::modified(Modifier::Control, KeyCode::S);
            let expected = RawInputs {
                keycodes: vec![KeyCode::LControl, KeyCode::RControl, KeyCode::S],
                ..Default::default()
            };
            let raw = input.raw_inputs();
            assert_eq!(expected, raw);
        }
    }

    mod mouse {
        use crate::user_input::{RawInputs, UserInput};

        #[test]
        fn mouse_button() {
            use bevy::input::mouse::MouseButton;

            let button = MouseButton::Left;
            let expected = RawInputs::from_mouse_button(button);
            let raw = UserInput::from(button).raw_inputs();
            assert_eq!(expected, raw);
        }

        #[test]
        fn mouse_wheel() {
            use crate::buttonlike::MouseWheelDirection;

            let direction = MouseWheelDirection::Down;
            let expected = RawInputs::from_mouse_wheel(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw);
        }

        #[test]
        fn mouse_motion() {
            use crate::buttonlike::MouseMotionDirection;

            let direction = MouseMotionDirection::Up;
            let expected = RawInputs::from_mouse_direction(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw);
        }

        #[test]
        fn single_mousewheel_axis() {
            use crate::axislike::{MouseWheelAxisType, SingleAxis};

            let direction = SingleAxis::from_value(MouseWheelAxisType::X, 1.0);
            let expected = RawInputs::from_single_axis(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn dual_mousewheel_axis() {
            use crate::axislike::{DualAxis, MouseWheelAxisType};

            let direction =
                DualAxis::from_value(MouseWheelAxisType::X, MouseWheelAxisType::Y, 1.0, 1.0);
            let expected = RawInputs::from_dual_axis(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn single_mouse_motion_axis() {
            use crate::axislike::{MouseMotionAxisType, SingleAxis};

            let direction = SingleAxis::from_value(MouseMotionAxisType::X, 1.0);
            let expected = RawInputs::from_single_axis(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn dual_mouse_motion_axis() {
            use crate::axislike::{DualAxis, MouseMotionAxisType};

            let direction =
                DualAxis::from_value(MouseMotionAxisType::X, MouseMotionAxisType::Y, 1.0, 1.0);
            let expected = RawInputs::from_dual_axis(direction);
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }
    }
}
