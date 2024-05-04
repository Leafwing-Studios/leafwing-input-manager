//! Keyboard inputs

use bevy::prelude::{KeyCode, Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::axislike::DualAxisData;
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessorExt, WithDualAxisProcessorExt,
};
use crate::input_streams::InputStreams;
use crate::prelude::raw_inputs::RawInputs;
use crate::user_input::{InputKind, UserInput};

/// A key or combination of keys used for capturing user input from the keyboard.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum KeyboardKey {
    /// A single physical key on the keyboard.
    PhysicalKey(KeyCode),

    /// Any of the specified physical keys.
    PhysicalKeyAny(Vec<KeyCode>),

    /// Keyboard modifiers like Alt, Control, Shift, and Super (OS symbol key).
    ModifierKey(ModifierKey),
}

impl KeyboardKey {
    /// Returns a list of [`KeyCode`]s used by this [`KeyboardKey`].
    #[must_use]
    #[inline]
    pub fn keycodes(&self) -> Vec<KeyCode> {
        match self {
            Self::PhysicalKey(keycode) => vec![*keycode],
            Self::PhysicalKeyAny(keycodes) => keycodes.clone(),
            Self::ModifierKey(modifier) => modifier.keycodes().to_vec(),
        }
    }
}

#[serde_typetag]
impl UserInput for KeyboardKey {
    /// [`KeyboardKey`] always acts as a button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Creates a [`RawInputs`] from the [`KeyCode`]s used by this [`KeyboardKey`].
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes(self.keycodes())
    }

    /// Returns a list of [`KeyCode`]s used by this [`KeyboardKey`].
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        self.keycodes()
            .iter()
            .map(|keycode| Box::new(*keycode) as Box<dyn UserInput>)
            .collect()
    }

    /// Checks if the specified [`KeyboardKey`] is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .keycodes
            .is_some_and(|keycodes| keycodes.any_pressed(self.keycodes()))
    }

    /// Retrieves the strength of the key press for the specified [`KeyboardKey`],
    /// returning `0.0` for no press and `1.0` for a currently pressed key.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`KeyboardKey`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

impl From<KeyCode> for KeyboardKey {
    #[inline]
    fn from(value: KeyCode) -> Self {
        Self::PhysicalKey(value)
    }
}

impl FromIterator<KeyCode> for KeyboardKey {
    #[inline]
    fn from_iter<T: IntoIterator<Item = KeyCode>>(iter: T) -> Self {
        Self::PhysicalKeyAny(iter.into_iter().collect())
    }
}

impl From<ModifierKey> for KeyboardKey {
    #[inline]
    fn from(value: ModifierKey) -> Self {
        Self::ModifierKey(value)
    }
}

// Built-in support for Bevy's KeyCode
#[serde_typetag]
impl UserInput for KeyCode {
    /// [`KeyCode`] always acts as a button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Creates a [`RawInputs`] from the key directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes([*self])
    }

    /// Returns a list that only contains the [`KeyCode`] itself,
    /// as it represents a simple physical button.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![Box::new(*self)]
    }

    /// Checks if the specified key is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .keycodes
            .is_some_and(|keys| keys.pressed(*self))
    }

    /// Retrieves the strength of the key press for the specified key,
    /// returning `0.0` for no press and `1.0` for a currently pressed key.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`KeyCode`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

/// Keyboard modifiers like Alt, Control, Shift, and Super (OS symbol key).
///
/// Each variant represents a pair of [`KeyCode`]s, the left and right version of the modifier key,
/// allowing for handling modifiers regardless of which side is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum ModifierKey {
    /// The Alt key, representing either [`KeyCode::AltLeft`] or [`KeyCode::AltRight`].
    Alt,

    /// The Control key, representing either [`KeyCode::ControlLeft`] or [`KeyCode::ControlRight`].
    Control,

    /// The Shift key, representing either [`KeyCode::ShiftLeft`] or [`KeyCode::ShiftRight`].
    Shift,

    /// The Super (OS symbol) key, representing either [`KeyCode::SuperLeft`] or [`KeyCode::SuperRight`].
    Super,
}

impl ModifierKey {
    /// Returns a pair of [`KeyCode`]s corresponding to both modifier keys.
    #[must_use]
    #[inline]
    pub const fn keycodes(&self) -> [KeyCode; 2] {
        [self.left(), self.right()]
    }

    /// Returns the [`KeyCode`] corresponding to the left modifier key.
    #[must_use]
    #[inline]
    pub const fn left(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltLeft,
            ModifierKey::Control => KeyCode::ControlLeft,
            ModifierKey::Shift => KeyCode::ShiftLeft,
            ModifierKey::Super => KeyCode::SuperLeft,
        }
    }

    /// Returns the [`KeyCode`] corresponding to the right modifier key.
    #[must_use]
    #[inline]
    pub const fn right(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltRight,
            ModifierKey::Control => KeyCode::ControlRight,
            ModifierKey::Shift => KeyCode::ShiftRight,
            ModifierKey::Super => KeyCode::SuperRight,
        }
    }
}

#[serde_typetag]
impl UserInput for ModifierKey {
    /// [`ModifierKey`] always acts as a button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Creates a [`RawInputs`] from two [`KeyCode`]s used by this [`ModifierKey`].
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes(self.keycodes())
    }

    /// Returns the two [`KeyCode`]s used by this [`ModifierKey`].
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![Box::new(self.left()), Box::new(self.right())]
    }

    /// Checks if the specified modifier key is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .keycodes
            .is_some_and(|keycodes| keycodes.any_pressed(self.keycodes()))
    }

    /// Gets the strength of the key press for the specified modifier key,
    /// returning `0.0` for no press and `1.0` for a currently pressed key.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        f32::from(self.pressed(input_streams))
    }

    /// Always returns [`None`] as [`ModifierKey`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

/// A virtual single-axis control constructed from two [`KeyboardKey`]s.
/// One button represents the negative direction (typically left or down),
/// while the other represents the positive direction (typically right or up).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct KeyboardVirtualAxis {
    /// The [`KeyboardKey`] used for the negative direction (typically left or down).
    pub(crate) negative: KeyboardKey,

    /// The [`KeyboardKey`] used for the negative direction (typically left or down).
    pub(crate) positive: KeyboardKey,

    /// The [`AxisProcessor`] used to handle input values.
    pub(crate) processor: AxisProcessor,
}

impl KeyboardVirtualAxis {
    /// Creates a new [`KeyboardVirtualAxis`] with two given [`KeyboardKey`]s.
    /// One button represents the negative direction (typically left or down),
    /// while the other represents the positive direction (typically right or up).
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub fn new(negative: impl Into<KeyboardKey>, positive: impl Into<KeyboardKey>) -> Self {
        Self {
            negative: negative.into(),
            positive: positive.into(),
            processor: AxisProcessor::None,
        }
    }

    /// The [`KeyboardVirtualAxis`] using the vertical arrow key mappings.
    ///
    /// - [`KeyCode::ArrowDown`] for negative direction.
    /// - [`KeyCode::ArrowUp`] for positive direction.
    pub const VERTICAL_ARROW_KEYS: Self = Self {
        negative: KeyboardKey::PhysicalKey(KeyCode::ArrowDown),
        positive: KeyboardKey::PhysicalKey(KeyCode::ArrowUp),
        processor: AxisProcessor::None,
    };

    /// The [`KeyboardVirtualAxis`] using the horizontal arrow key mappings.
    ///
    /// - [`KeyCode::ArrowLeft`] for negative direction.
    /// - [`KeyCode::ArrowRight`] for positive direction.
    pub const HORIZONTAL_ARROW_KEYS: Self = Self {
        negative: KeyboardKey::PhysicalKey(KeyCode::ArrowLeft),
        positive: KeyboardKey::PhysicalKey(KeyCode::ArrowRight),
        processor: AxisProcessor::None,
    };

    /// The [`KeyboardVirtualAxis`] using the common W/S key mappings.
    ///
    /// - [`KeyCode::KeyS`] for negative direction.
    /// - [`KeyCode::KeyW`] for positive direction.
    pub const WS: Self = Self {
        negative: KeyboardKey::PhysicalKey(KeyCode::KeyS),
        positive: KeyboardKey::PhysicalKey(KeyCode::KeyW),
        processor: AxisProcessor::None,
    };

    /// The [`KeyboardVirtualAxis`] using the common A/D key mappings.
    ///
    /// - [`KeyCode::KeyA`] for negative direction.
    /// - [`KeyCode::KeyD`] for positive direction.
    pub const AD: Self = Self {
        negative: KeyboardKey::PhysicalKey(KeyCode::KeyA),
        positive: KeyboardKey::PhysicalKey(KeyCode::KeyD),
        processor: AxisProcessor::None,
    };

    /// The [`KeyboardVirtualAxis`] using the vertical numpad key mappings.
    ///
    /// - [`KeyCode::Numpad2`] for negative direction.
    /// - [`KeyCode::Numpad8`] for positive direction.
    pub const VERTICAL_NUMPAD: Self = Self {
        negative: KeyboardKey::PhysicalKey(KeyCode::Numpad2),
        positive: KeyboardKey::PhysicalKey(KeyCode::Numpad8),
        processor: AxisProcessor::None,
    };

    /// The [`KeyboardVirtualAxis`] using the horizontal numpad key mappings.
    ///
    /// - [`KeyCode::Numpad4`] for negative direction.
    /// - [`KeyCode::Numpad6`] for positive direction.
    pub const HORIZONTAL_NUMPAD: Self = Self {
        negative: KeyboardKey::PhysicalKey(KeyCode::Numpad4),
        positive: KeyboardKey::PhysicalKey(KeyCode::Numpad6),
        processor: AxisProcessor::None,
    };
}

#[serde_typetag]
impl UserInput for KeyboardVirtualAxis {
    /// [`KeyboardVirtualAxis`] always acts as a virtual axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Axis
    }

    /// Creates a [`RawInputs`] from two [`KeyCode`]s used by this axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        let keycodes = self
            .negative
            .keycodes()
            .into_iter()
            .chain(self.positive.keycodes());
        RawInputs::from_keycodes(keycodes)
    }

    /// Returns the two [`KeyboardKey`]s used by this axis.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![
            Box::new(self.negative.clone()),
            Box::new(self.positive.clone()),
        ]
    }

    /// Checks if this axis has a non-zero value after processing by the associated processor.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.value(input_streams) != 0.0
    }

    /// Retrieves the current value of this axis after processing by the associated processor.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let Some(keycodes) = input_streams.keycodes else {
            return 0.0;
        };

        let negative = f32::from(keycodes.any_pressed(self.negative.keycodes()));
        let positive = f32::from(keycodes.any_pressed(self.positive.keycodes()));
        let value = positive - negative;
        self.processor.process(value)
    }

    /// Always returns [`None`] as [`KeyboardVirtualAxis`] doesn't represent dual-axis input.
    #[must_use]
    #[inline]
    fn axis_pair(&self, _input_streams: &InputStreams) -> Option<DualAxisData> {
        None
    }
}

impl WithAxisProcessorExt for KeyboardVirtualAxis {
    #[inline]
    fn no_processor(mut self) -> Self {
        self.processor = AxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

/// A virtual single-axis control constructed from four [`KeyboardKey`]s.
/// Each button represents a specific direction (up, down, left, right),
/// functioning similarly to a directional pad (D-pad) on both X and Y axes,
/// and offering intermediate diagonals by means of two-button combinations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct KeyboardVirtualDPad {
    /// The [`KeyboardKey`] used for the upward direction.
    pub(crate) up: KeyboardKey,

    /// The [`KeyboardKey`] used for the downward direction.
    pub(crate) down: KeyboardKey,

    /// The [`KeyboardKey`] used for the leftward direction.
    pub(crate) left: KeyboardKey,

    /// The [`KeyboardKey`] used for the rightward direction.
    pub(crate) right: KeyboardKey,

    /// The [`DualAxisProcessor`] used to handle input values.
    pub(crate) processor: DualAxisProcessor,
}

impl KeyboardVirtualDPad {
    /// Creates a new [`KeyboardVirtualDPad`] with four given [`KeyboardKey`]s.
    /// Each button represents a specific direction (up, down, left, right).
    /// No processing is applied to raw data from the keyboard.
    #[inline]
    pub fn new(
        up: impl Into<KeyboardKey>,
        down: impl Into<KeyboardKey>,
        left: impl Into<KeyboardKey>,
        right: impl Into<KeyboardKey>,
    ) -> Self {
        Self {
            up: up.into(),
            down: down.into(),
            left: left.into(),
            right: right.into(),
            processor: DualAxisProcessor::None,
        }
    }

    /// The [`KeyboardVirtualDPad`] using the common arrow key mappings.
    ///
    /// - [`KeyCode::ArrowUp`] for upward direction.
    /// - [`KeyCode::ArrowDown`] for downward direction.
    /// - [`KeyCode::ArrowLeft`] for leftward direction.
    /// - [`KeyCode::ArrowRight`] for rightward direction.
    pub const ARROW_KEYS: Self = Self {
        up: KeyboardKey::PhysicalKey(KeyCode::ArrowUp),
        down: KeyboardKey::PhysicalKey(KeyCode::ArrowDown),
        left: KeyboardKey::PhysicalKey(KeyCode::ArrowLeft),
        right: KeyboardKey::PhysicalKey(KeyCode::ArrowRight),
        processor: DualAxisProcessor::None,
    };

    /// The [`KeyboardVirtualDPad`] using the common WASD key mappings.
    ///
    /// - [`KeyCode::KeyW`] for upward direction.
    /// - [`KeyCode::KeyS`] for downward direction.
    /// - [`KeyCode::KeyA`] for leftward direction.
    /// - [`KeyCode::KeyD`] for rightward direction.
    pub const WASD: Self = Self {
        up: KeyboardKey::PhysicalKey(KeyCode::KeyW),
        down: KeyboardKey::PhysicalKey(KeyCode::KeyS),
        left: KeyboardKey::PhysicalKey(KeyCode::KeyA),
        right: KeyboardKey::PhysicalKey(KeyCode::KeyD),
        processor: DualAxisProcessor::None,
    };

    /// The [`KeyboardVirtualDPad`] using the common numpad key mappings.
    ///
    /// - [`KeyCode::Numpad8`] for upward direction.
    /// - [`KeyCode::Numpad2`] for downward direction.
    /// - [`KeyCode::Numpad4`] for leftward direction.
    /// - [`KeyCode::Numpad6`] for rightward direction.
    pub const NUMPAD: Self = Self {
        up: KeyboardKey::PhysicalKey(KeyCode::Numpad8),
        down: KeyboardKey::PhysicalKey(KeyCode::Numpad2),
        left: KeyboardKey::PhysicalKey(KeyCode::Numpad4),
        right: KeyboardKey::PhysicalKey(KeyCode::Numpad6),
        processor: DualAxisProcessor::None,
    };

    /// Retrieves the current X and Y values of this D-pad after processing by the associated processor.
    #[must_use]
    #[inline]
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let Some(keycodes) = input_streams.keycodes else {
            return Vec2::ZERO;
        };

        let up = f32::from(keycodes.any_pressed(self.up.keycodes()));
        let down = f32::from(keycodes.any_pressed(self.down.keycodes()));
        let left = f32::from(keycodes.any_pressed(self.left.keycodes()));
        let right = f32::from(keycodes.any_pressed(self.right.keycodes()));
        let value = Vec2::new(right - left, up - down);
        self.processor.process(value)
    }
}

#[serde_typetag]
impl UserInput for KeyboardVirtualDPad {
    /// [`KeyboardVirtualDPad`] always acts as a virtual dual-axis input.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::DualAxis
    }

    /// Creates a [`RawInputs`] from four [`KeyCode`]s used by this D-pad.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        let keycodes = self
            .up
            .keycodes()
            .into_iter()
            .chain(self.down.keycodes())
            .chain(self.left.keycodes())
            .chain(self.right.keycodes());
        RawInputs::from_keycodes(keycodes)
    }

    /// Returns the four [`KeyboardKey`]s used by this D-pad.
    #[must_use]
    #[inline]
    fn destructure(&self) -> Vec<Box<dyn UserInput>> {
        vec![
            Box::new(self.up.clone()),
            Box::new(self.down.clone()),
            Box::new(self.left.clone()),
            Box::new(self.right.clone()),
        ]
    }

    /// Checks if this D-pad has a non-zero magnitude after processing by the associated processor.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.processed_value(input_streams) != Vec2::ZERO
    }

    /// Retrieves the magnitude of the value from this D-pad after processing by the associated processor.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        self.processed_value(input_streams).length()
    }

    /// Retrieves the current X and Y values of this D-pad after processing by the associated processor.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        let value = self.processed_value(input_streams);
        Some(DualAxisData::from_xy(value))
    }
}

impl WithDualAxisProcessorExt for KeyboardVirtualDPad {
    #[inline]
    fn no_processor(mut self) -> Self {
        self.processor = DualAxisProcessor::None;
        self
    }

    #[inline]
    fn replace_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = processor.into();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processor = self.processor.with_processor(processor);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mocking::MockInput;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin);
        app
    }

    fn check(
        input: &impl UserInput,
        input_streams: &InputStreams,
        expected_pressed: bool,
        expected_value: f32,
        expected_axis_pair: Option<DualAxisData>,
    ) {
        assert_eq!(input.pressed(input_streams), expected_pressed);
        assert_eq!(input.value(input_streams), expected_value);
        assert_eq!(input.axis_pair(input_streams), expected_axis_pair);
    }

    fn pressed(input: &impl UserInput, input_streams: &InputStreams) {
        check(input, input_streams, true, 1.0, None);
    }

    fn released(input: &impl UserInput, input_streams: &InputStreams) {
        check(input, input_streams, false, 0.0, None);
    }

    #[test]
    fn test_keyboard_input() {
        let up = KeyCode::ArrowUp;
        assert_eq!(up.kind(), InputKind::Button);
        assert_eq!(up.raw_inputs(), RawInputs::from_keycodes([up]));

        let left = KeyCode::ArrowLeft;
        assert_eq!(left.kind(), InputKind::Button);
        assert_eq!(left.raw_inputs(), RawInputs::from_keycodes([left]));

        let alt = ModifierKey::Alt;
        assert_eq!(alt.kind(), InputKind::Button);
        let alt_raw_inputs = RawInputs::from_keycodes([KeyCode::AltLeft, KeyCode::AltRight]);
        assert_eq!(alt.raw_inputs(), alt_raw_inputs);

        let physical_up = KeyboardKey::PhysicalKey(up);
        assert_eq!(physical_up.kind(), InputKind::Button);
        assert_eq!(physical_up.raw_inputs(), RawInputs::from_keycodes([up]));

        let physical_any_up_left = KeyboardKey::PhysicalKeyAny(vec![up, left]);
        assert_eq!(physical_any_up_left.kind(), InputKind::Button);
        let raw_inputs = RawInputs::from_keycodes([up, left]);
        assert_eq!(physical_any_up_left.raw_inputs(), raw_inputs);

        let keyboard_alt = KeyboardKey::ModifierKey(alt);
        assert_eq!(keyboard_alt.kind(), InputKind::Button);
        assert_eq!(keyboard_alt.raw_inputs(), alt_raw_inputs);

        let arrow_y = KeyboardVirtualAxis::VERTICAL_ARROW_KEYS;
        assert_eq!(arrow_y.kind(), InputKind::Axis);
        let raw_inputs = RawInputs::from_keycodes([KeyCode::ArrowDown, KeyCode::ArrowUp]);
        assert_eq!(arrow_y.raw_inputs(), raw_inputs);

        let arrows = KeyboardVirtualDPad::ARROW_KEYS;
        assert_eq!(arrows.kind(), InputKind::DualAxis);
        let raw_inputs = RawInputs::from_keycodes([
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]);
        assert_eq!(arrows.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Some(DualAxisData::ZERO);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&left, &inputs);
        released(&alt, &inputs);
        released(&physical_up, &inputs);
        released(&physical_any_up_left, &inputs);
        released(&keyboard_alt, &inputs);
        released(&arrow_y, &inputs);
        check(&arrows, &inputs, false, 0.0, zeros);

        // Press arrow up
        let data = DualAxisData::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowUp);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&up, &inputs);
        released(&left, &inputs);
        released(&alt, &inputs);
        pressed(&physical_up, &inputs);
        pressed(&physical_any_up_left, &inputs);
        released(&keyboard_alt, &inputs);
        check(&arrow_y, &inputs, true, data.y(), None);
        check(&arrows, &inputs, true, data.length(), Some(data));

        // Press arrow down
        let data = DualAxisData::new(0.0, -1.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowDown);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&left, &inputs);
        released(&alt, &inputs);
        released(&physical_up, &inputs);
        released(&physical_any_up_left, &inputs);
        released(&keyboard_alt, &inputs);
        check(&arrow_y, &inputs, true, data.y(), None);
        check(&arrows, &inputs, true, data.length(), Some(data));

        // Press arrow left
        let data = DualAxisData::new(-1.0, 0.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowLeft);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        pressed(&left, &inputs);
        released(&alt, &inputs);
        released(&physical_up, &inputs);
        pressed(&physical_any_up_left, &inputs);
        released(&keyboard_alt, &inputs);
        released(&arrow_y, &inputs);
        check(&arrows, &inputs, true, data.length(), Some(data));

        // Press arrow down and arrow up
        let mut app = test_app();
        app.press_input(KeyCode::ArrowDown);
        app.press_input(KeyCode::ArrowUp);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&up, &inputs);
        released(&left, &inputs);
        released(&alt, &inputs);
        pressed(&physical_up, &inputs);
        pressed(&physical_any_up_left, &inputs);
        released(&keyboard_alt, &inputs);
        released(&arrow_y, &inputs);
        check(&arrows, &inputs, false, 0.0, zeros);

        // Press arrow left and arrow up
        let data = DualAxisData::new(-1.0, 1.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowLeft);
        app.press_input(KeyCode::ArrowUp);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&up, &inputs);
        pressed(&left, &inputs);
        released(&alt, &inputs);
        pressed(&physical_up, &inputs);
        pressed(&physical_any_up_left, &inputs);
        released(&keyboard_alt, &inputs);
        check(&arrow_y, &inputs, true, data.y(), None);
        check(&arrows, &inputs, true, data.length(), Some(data));

        // Press left Alt
        let mut app = test_app();
        app.press_input(KeyCode::AltLeft);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&left, &inputs);
        pressed(&alt, &inputs);
        released(&physical_up, &inputs);
        released(&physical_any_up_left, &inputs);
        pressed(&keyboard_alt, &inputs);
        released(&arrow_y, &inputs);
        check(&arrows, &inputs, false, 0.0, zeros);

        // Press right Alt
        let mut app = test_app();
        app.press_input(KeyCode::AltRight);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&up, &inputs);
        released(&left, &inputs);
        pressed(&alt, &inputs);
        released(&physical_up, &inputs);
        released(&physical_any_up_left, &inputs);
        pressed(&keyboard_alt, &inputs);
        released(&arrow_y, &inputs);
        check(&arrows, &inputs, false, 0.0, zeros);
    }
}
