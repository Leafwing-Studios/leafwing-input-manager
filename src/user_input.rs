//! Helpful abstractions over user inputs of all sorts

use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode, mouse::MouseButton};
use bevy::reflect::Reflect;
use bevy::utils::HashSet;
use serde::{Deserialize, Serialize};

use crate::axislike::VirtualAxis;
use crate::{
    axislike::{AxisType, DualAxis, SingleAxis, VirtualDPad},
    buttonlike::{MouseMotionDirection, MouseWheelDirection},
};

/// Some combination of user input, which may cross input-mode boundaries.
///
/// For example, this may store mouse, keyboard or gamepad input, including cross-device chords!
///
/// Suitable for use in an [`InputMap`](crate::input_map::InputMap)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum UserInput {
    /// A single button
    Single(InputKind),
    /// A combination of buttons, pressed simultaneously
    // Note: we cannot use a HashSet here because of https://users.rust-lang.org/t/hash-not-implemented-why-cant-it-be-derived/92416/8
    // We cannot use a BTreeSet because the underlying types don't impl Ord
    // We don't want to use a PetitSet here because of memory bloat
    // So a vec it is!
    // RIP your uniqueness guarantees
    Chord(Vec<InputKind>),
    /// A virtual D-pad that you can get a [`DualAxis`] from
    VirtualDPad(VirtualDPad),
    /// A virtual axis that you can get a [`SingleAxis`] from
    VirtualAxis(VirtualAxis),
}

impl UserInput {
    /// Creates a [`UserInput::Chord`] from a [`Modifier`] and an `input` which can be converted into an [`InputKind`]
    ///
    /// When working with keyboard modifiers,
    /// should be preferred to manually specifying both the left and right variant.
    pub fn modified(modifier: Modifier, input: impl Into<InputKind>) -> UserInput {
        let modifier: InputKind = modifier.into();
        let input: InputKind = input.into();

        UserInput::chord(vec![modifier, input])
    }

    /// Creates a [`UserInput::Chord`] from an iterator over inputs of the same type convertible into an [`InputKind`]s
    ///
    /// If `inputs` has a length of 1, a [`UserInput::Single`] variant will be returned instead.
    pub fn chord(inputs: impl IntoIterator<Item = impl Into<InputKind>>) -> Self {
        // We can't just check the length unless we add an ExactSizeIterator bound :(
        let vec: Vec<InputKind> = inputs.into_iter().map(|input| input.into()).collect();

        match vec.len() {
            1 => UserInput::Single(vec[0].clone()),
            _ => UserInput::Chord(vec),
        }
    }

    /// The number of logical inputs that make up the [`UserInput`].
    ///
    /// - A [`Single`][UserInput::Single] input returns 1
    /// - A [`Chord`][UserInput::Chord] returns the number of buttons in the chord
    /// - A [`VirtualDPad`][UserInput::VirtualDPad] returns 1
    /// - A [`VirtualAxis`][UserInput::VirtualAxis] returns 1
    pub fn len(&self) -> usize {
        match self {
            UserInput::Chord(button_set) => button_set.len(),
            _ => 1,
        }
    }

    /// Is the number of buttons in the [`UserInput`] 0?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many of the provided `buttons` are found in the [`UserInput`]
    ///
    /// # Example
    /// ```rust
    /// use bevy::input::keyboard::KeyCode::*;
    /// use bevy::utils::HashSet;
    /// use leafwing_input_manager::user_input::UserInput;
    ///
    /// let buttons = HashSet::from_iter([ControlLeft.into(), AltLeft.into()]);
    /// let a: UserInput  = KeyA.into();
    /// let ctrl_a = UserInput::chord([ControlLeft, KeyA]);
    /// let ctrl_alt_a = UserInput::chord([ControlLeft, AltLeft, KeyA]);
    ///
    /// assert_eq!(a.n_matching(&buttons), 0);
    /// assert_eq!(ctrl_a.n_matching(&buttons), 1);
    /// assert_eq!(ctrl_alt_a.n_matching(&buttons), 2);
    /// ```
    pub fn n_matching(&self, buttons: &HashSet<InputKind>) -> usize {
        self.iter()
            .filter(|button| buttons.contains(button))
            .count()
    }

    /// Returns the raw inputs that make up this [`UserInput`]
    pub fn raw_inputs(&self) -> RawInputs {
        self.iter()
            .fold(RawInputs::default(), |mut raw_inputs, input| {
                raw_inputs.merge_input_data(&input);
                raw_inputs
            })
    }

    pub(crate) fn iter(&self) -> UserInputIter {
        match self {
            UserInput::Single(button) => UserInputIter::Single(Some(button.clone())),
            UserInput::Chord(buttons) => UserInputIter::Chord(buttons.iter()),
            UserInput::VirtualDPad(dpad) => UserInputIter::VirtualDPad(
                Some(dpad.up.clone()),
                Some(dpad.down.clone()),
                Some(dpad.left.clone()),
                Some(dpad.right.clone()),
            ),
            UserInput::VirtualAxis(axis) => {
                UserInputIter::VirtualAxis(Some(axis.negative.clone()), Some(axis.positive.clone()))
            }
        }
    }
}

pub(crate) enum UserInputIter<'a> {
    Single(Option<InputKind>),
    Chord(std::slice::Iter<'a, InputKind>),
    VirtualDPad(
        Option<InputKind>,
        Option<InputKind>,
        Option<InputKind>,
        Option<InputKind>,
    ),
    VirtualAxis(Option<InputKind>, Option<InputKind>),
}

impl<'a> Iterator for UserInputIter<'a> {
    type Item = InputKind;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(ref mut input) => input.take(),
            Self::Chord(ref mut iter) => iter.next().cloned(),
            Self::VirtualDPad(ref mut up, ref mut down, ref mut left, ref mut right) => up
                .take()
                .or_else(|| down.take().or_else(|| left.take().or_else(|| right.take()))),
            Self::VirtualAxis(ref mut negative, ref mut positive) => {
                negative.take().or_else(|| positive.take())
            }
        }
    }
}

impl From<InputKind> for UserInput {
    fn from(input: InputKind) -> Self {
        UserInput::Single(input)
    }
}

impl From<DualAxis> for UserInput {
    fn from(input: DualAxis) -> Self {
        UserInput::Single(InputKind::DualAxis(input))
    }
}

impl From<SingleAxis> for UserInput {
    fn from(input: SingleAxis) -> Self {
        UserInput::Single(InputKind::SingleAxis(input))
    }
}

impl From<VirtualDPad> for UserInput {
    fn from(input: VirtualDPad) -> Self {
        UserInput::VirtualDPad(input)
    }
}

impl From<VirtualAxis> for UserInput {
    fn from(input: VirtualAxis) -> Self {
        UserInput::VirtualAxis(input)
    }
}

impl From<GamepadButtonType> for UserInput {
    fn from(input: GamepadButtonType) -> Self {
        UserInput::Single(InputKind::GamepadButton(input))
    }
}

impl From<KeyCode> for UserInput {
    fn from(input: KeyCode) -> Self {
        UserInput::Single(InputKind::PhysicalKey(input))
    }
}

impl From<MouseButton> for UserInput {
    fn from(input: MouseButton) -> Self {
        UserInput::Single(InputKind::Mouse(input))
    }
}

impl From<MouseWheelDirection> for UserInput {
    fn from(input: MouseWheelDirection) -> Self {
        UserInput::Single(InputKind::MouseWheel(input))
    }
}

impl From<MouseMotionDirection> for UserInput {
    fn from(input: MouseMotionDirection) -> Self {
        UserInput::Single(InputKind::MouseMotion(input))
    }
}

impl From<Modifier> for UserInput {
    fn from(input: Modifier) -> Self {
        UserInput::Single(InputKind::Modifier(input))
    }
}

/// The different kinds of supported input bindings.
///
/// Commonly stored in the [`UserInput`] enum.
///
/// Unfortunately, we cannot use a trait object here, as the types used by `ButtonInput`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum InputKind {
    /// A button on a gamepad
    GamepadButton(GamepadButtonType),
    /// A single axis of continuous motion
    SingleAxis(SingleAxis),
    /// Two paired axes of continuous motion
    DualAxis(DualAxis),
    /// The physical location of a key on the keyboard.
    PhysicalKey(KeyCode),
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
        InputKind::PhysicalKey(input)
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
/// This buttonlike input is stored in [`InputKind`], and will be triggered whenever either of these buttons is pressed.
/// This will be decomposed into both values when converted into [`RawInputs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum Modifier {
    /// Corresponds to [`KeyCode::AltLeft`] and [`KeyCode::AltRight`].
    Alt,
    /// Corresponds to [`KeyCode::ControlLeft`] and [`KeyCode::ControlRight`].
    Control,
    /// The key that makes letters capitalized, corresponding to [`KeyCode::ShiftLeft`] and [`KeyCode::ShiftRight`]
    Shift,
    /// The OS or "Windows" key, corresponding to [`KeyCode::SuperLeft`] and [`KeyCode::SuperRight`].
    Super,
}

impl Modifier {
    /// Returns the pair of [`KeyCode`] values associated with this modifier.
    ///
    /// The left variant will always be in the first position, and the right variant is always in the second position.
    #[inline]
    pub fn key_codes(self) -> [KeyCode; 2] {
        match self {
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        }
    }
}

/// The basic input events that make up a [`UserInput`].
///
/// Obtained by calling [`UserInput::raw_inputs()`].
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RawInputs {
    /// Physical key locations.
    pub keycodes: Vec<KeyCode>,
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

impl RawInputs {
    /// Merges the data from the given `input_kind` into `self`.
    fn merge_input_data(&mut self, input_kind: &InputKind) {
        match input_kind {
            InputKind::DualAxis(DualAxis {
                x_axis_type,
                y_axis_type,
                value,
                ..
            }) => {
                self.axis_data.push((*x_axis_type, value.map(|v| v.x)));
                self.axis_data.push((*y_axis_type, value.map(|v| v.y)));
            }
            InputKind::SingleAxis(single_axis) => self
                .axis_data
                .push((single_axis.axis_type, single_axis.value)),
            InputKind::GamepadButton(button) => self.gamepad_buttons.push(*button),
            InputKind::PhysicalKey(key_code) => self.keycodes.push(*key_code),
            InputKind::Modifier(modifier) => {
                self.keycodes.extend_from_slice(&modifier.key_codes());
            }
            InputKind::Mouse(button) => self.mouse_buttons.push(*button),
            InputKind::MouseWheel(button) => self.mouse_wheel.push(*button),
            InputKind::MouseMotion(button) => self.mouse_motion.push(*button),
        }
    }
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
                (axis.x_axis_type, axis.value.map(|v| v.x)),
                (axis.y_axis_type, axis.value.map(|v| v.y)),
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
            InputKind::SingleAxis(SingleAxis::new(GamepadAxisType::LeftZ)),
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
            let expected = RawInputs::from_single_axis(direction.clone());
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
            let expected = RawInputs::from_dual_axis(direction.clone());
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }
    }

    mod keyboard {
        use crate::user_input::{RawInputs, UserInput};

        #[test]
        fn keyboard_button() {
            use bevy::input::keyboard::KeyCode;

            let button = KeyCode::KeyA;
            let expected = RawInputs::from_keycode(button);
            let raw = UserInput::from(button).raw_inputs();
            assert_eq!(expected, raw);
        }

        #[test]
        fn modifier_key_decomposes_into_both_inputs() {
            use crate::user_input::Modifier;
            use bevy::input::keyboard::KeyCode;

            let input = UserInput::modified(Modifier::Control, KeyCode::KeyS);
            let expected = RawInputs {
                keycodes: vec![KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::KeyS],
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
            let expected = RawInputs::from_single_axis(direction.clone());
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn dual_mousewheel_axis() {
            use crate::axislike::{DualAxis, MouseWheelAxisType};

            let direction =
                DualAxis::from_value(MouseWheelAxisType::X, MouseWheelAxisType::Y, 1.0, 1.0);
            let expected = RawInputs::from_dual_axis(direction.clone());
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn single_mouse_motion_axis() {
            use crate::axislike::{MouseMotionAxisType, SingleAxis};

            let direction = SingleAxis::from_value(MouseMotionAxisType::X, 1.0);
            let expected = RawInputs::from_single_axis(direction.clone());
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }

        #[test]
        fn dual_mouse_motion_axis() {
            use crate::axislike::{DualAxis, MouseMotionAxisType};

            let direction =
                DualAxis::from_value(MouseMotionAxisType::X, MouseMotionAxisType::Y, 1.0, 1.0);
            let expected = RawInputs::from_dual_axis(direction.clone());
            let raw = UserInput::from(direction).raw_inputs();
            assert_eq!(expected, raw)
        }
    }
}
