//! Helpful abstractions over user inputs of all sorts

use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode, mouse::MouseButton};

use bevy::utils::HashSet;
use petitset::PetitSet;
use serde::{Deserialize, Serialize};

use crate::{
    axislike::{AxisType, DualAxis, SingleAxis, VirtualDPad},
    buttonlike::{MouseMotionDirection, MouseWheelDirection},
};

/// Some combination of user input, which may cross [`Input`]-mode boundaries
///
/// Suitable for use in an [`InputMap`](crate::input_map::InputMap)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UserInput {
    /// A single button
    Single(InputKind),
    /// A combination of buttons, pressed simultaneously
    ///
    /// Up to 8 (!!) buttons can be chorded together at once.
    /// Chords are considered to belong to all of the [InputMode]s of their constituent buttons.
    Chord(PetitSet<InputKind, 8>),
    /// A virtual DPad that you can get an [`AxisPair`] from
    VirtualDPad(VirtualDPad),
}

impl UserInput {
    /// Creates a [`UserInput::Chord`] from a [`Modifier`] and an `input` that can be converted into an [`InputKind`]
    ///
    /// When working with keyboard modifiers, should be preferred over manually specifying both the left and right variant.
    pub fn modified(modifier: Modifier, input: impl Into<InputKind>) -> UserInput {
        let modifier: InputKind = modifier.into();
        let input: InputKind = input.into();
        let mut set: PetitSet<InputKind, 8> = PetitSet::default();
        set.insert(modifier);
        set.insert(input);

        UserInput::Chord(set)
    }

    /// Creates a [`UserInput::Chord`] from an iterator of inputs of the same type that can be converted into an [`InputKind`]s
    ///
    /// If `inputs` has a length of 1, a [`UserInput::Single`] variant will be returned instead.
    pub fn chord(inputs: impl IntoIterator<Item = impl Into<InputKind>>) -> Self {
        // We can't just check the length unless we add an ExactSizeIterator bound :(
        let mut length: u8 = 0;

        let mut set: PetitSet<InputKind, 8> = PetitSet::default();
        for button in inputs {
            length += 1;
            set.insert(button.into());
        }

        match length {
            1 => UserInput::Single(set.into_iter().next().unwrap()),
            _ => UserInput::Chord(set),
        }
    }

    /// The number of logical inputs that make up the [`UserInput`].
    ///
    /// - A [`Single`][UserInput::Single] input returns 1
    /// - A [`Chord`][UserInput::Chord] returns the number of buttons in the chord
    /// - A [`VirtualDPad`][UserInput::VirtualDPad] returns 1
    pub fn len(&self) -> usize {
        match self {
            UserInput::Single(_) => 1,
            UserInput::Chord(button_set) => button_set.len(),
            UserInput::VirtualDPad { .. } => 1,
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
    /// let buttons = HashSet::from_iter([LControl.into(), LAlt.into()]);
    /// let a: UserInput  = A.into();
    /// let ctrl_a = UserInput::chord([LControl, A]);
    /// let ctrl_alt_a = UserInput::chord([LControl, LAlt, A]);
    ///
    /// assert_eq!(a.n_matching(&buttons), 0);
    /// assert_eq!(ctrl_a.n_matching(&buttons), 1);
    /// assert_eq!(ctrl_alt_a.n_matching(&buttons), 2);
    /// ```
    pub fn n_matching(&self, buttons: &HashSet<InputKind>) -> usize {
        match self {
            UserInput::Single(button) => usize::from(buttons.contains(button)),
            UserInput::Chord(chord_buttons) => {
                let mut n_matching = 0;
                for button in buttons.iter() {
                    if chord_buttons.contains(button) {
                        n_matching += 1;
                    }
                }

                n_matching
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                let mut n_matching = 0;
                for button in buttons.iter() {
                    for dpad_button in [up, down, left, right] {
                        if button == dpad_button {
                            n_matching += 1;
                        }
                    }
                }

                n_matching
            }
        }
    }

    /// Returns the raw inputs that make up this [`UserInput`]
    pub fn raw_inputs(&self) -> RawInputs {
        let mut raw_inputs = RawInputs::default();

        match self {
            UserInput::Single(button) => match *button {
                InputKind::DualAxis(dual_axis) => {
                    raw_inputs
                        .axis_data
                        .push((dual_axis.x.axis_type, dual_axis.x.value));
                    raw_inputs
                        .axis_data
                        .push((dual_axis.y.axis_type, dual_axis.y.value));
                }
                InputKind::SingleAxis(single_axis) => raw_inputs
                    .axis_data
                    .push((single_axis.axis_type, single_axis.value)),
                InputKind::GamepadButton(button) => raw_inputs.gamepad_buttons.push(button),
                InputKind::Keyboard(button) => raw_inputs.keycodes.push(button),
                InputKind::Modifier(modifier) => {
                    let key_codes = modifier.key_codes();
                    raw_inputs.keycodes.push(key_codes[0]);
                    raw_inputs.keycodes.push(key_codes[1]);
                }
                InputKind::Mouse(button) => raw_inputs.mouse_buttons.push(button),
                InputKind::MouseWheel(button) => raw_inputs.mouse_wheel.push(button),
                InputKind::MouseMotion(button) => raw_inputs.mouse_motion.push(button),
            },
            UserInput::Chord(button_set) => {
                for button in button_set.iter() {
                    match *button {
                        InputKind::DualAxis(dual_axis) => {
                            raw_inputs
                                .axis_data
                                .push((dual_axis.x.axis_type, dual_axis.x.value));
                            raw_inputs
                                .axis_data
                                .push((dual_axis.y.axis_type, dual_axis.y.value));
                        }
                        InputKind::SingleAxis(single_axis) => raw_inputs
                            .axis_data
                            .push((single_axis.axis_type, single_axis.value)),
                        InputKind::GamepadButton(button) => raw_inputs.gamepad_buttons.push(button),
                        InputKind::Keyboard(button) => raw_inputs.keycodes.push(button),
                        InputKind::Modifier(modifier) => {
                            let key_codes = modifier.key_codes();
                            raw_inputs.keycodes.push(key_codes[0]);
                            raw_inputs.keycodes.push(key_codes[1]);
                        }
                        InputKind::Mouse(button) => raw_inputs.mouse_buttons.push(button),
                        InputKind::MouseWheel(button) => raw_inputs.mouse_wheel.push(button),
                        InputKind::MouseMotion(button) => raw_inputs.mouse_motion.push(button),
                    }
                }
            }
            UserInput::VirtualDPad(VirtualDPad {
                up,
                down,
                left,
                right,
            }) => {
                for button in [up, down, left, right] {
                    match *button {
                        InputKind::DualAxis(dual_axis) => {
                            raw_inputs
                                .axis_data
                                .push((dual_axis.x.axis_type, dual_axis.x.value));
                            raw_inputs
                                .axis_data
                                .push((dual_axis.y.axis_type, dual_axis.y.value));
                        }
                        InputKind::SingleAxis(single_axis) => raw_inputs
                            .axis_data
                            .push((single_axis.axis_type, single_axis.value)),
                        InputKind::GamepadButton(button) => raw_inputs.gamepad_buttons.push(button),
                        InputKind::Keyboard(button) => raw_inputs.keycodes.push(button),
                        InputKind::Modifier(modifier) => {
                            let key_codes = modifier.key_codes();
                            raw_inputs.keycodes.push(key_codes[0]);
                            raw_inputs.keycodes.push(key_codes[1]);
                        }
                        InputKind::Mouse(button) => raw_inputs.mouse_buttons.push(button),
                        InputKind::MouseWheel(button) => raw_inputs.mouse_wheel.push(button),
                        InputKind::MouseMotion(button) => raw_inputs.mouse_motion.push(button),
                    }
                }
            }
        };

        raw_inputs
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

impl From<GamepadButtonType> for UserInput {
    fn from(input: GamepadButtonType) -> Self {
        UserInput::Single(InputKind::GamepadButton(input))
    }
}

impl From<KeyCode> for UserInput {
    fn from(input: KeyCode) -> Self {
        UserInput::Single(InputKind::Keyboard(input))
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
/// See [`InputMode`] for the value-less equivalent. Commonly stored in the [`UserInput`] enum.
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
    /// A single axis of continous motion
    SingleAxis(SingleAxis),
    /// Two paired axes of continous motion
    DualAxis(DualAxis),
    /// A button on a keyboard
    Keyboard(KeyCode),
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
    /// Physical keyboard buttons
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
