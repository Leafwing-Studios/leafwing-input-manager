//! Keyboard inputs

use bevy::prelude::{KeyCode, Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessingPipelineExt,
    WithDualAxisProcessingPipelineExt,
};
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::user_input::{ButtonlikeChord, UserInput};
use crate::InputControlKind;

use super::{Axislike, Buttonlike, DualAxislike};

// Built-in support for Bevy's KeyCode
#[serde_typetag]
impl UserInput for KeyCode {
    /// [`KeyCode`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Returns a [`BasicInputs`] that only contains the [`KeyCode`] itself,
    /// as it represents a simple physical button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }

    /// Creates a [`RawInputs`] from the key directly.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes([*self])
    }
}

impl Buttonlike for KeyCode {
    /// Checks if the specified key is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .keycodes
            .is_some_and(|keys| keys.pressed(*self))
    }
}

/// Keyboard modifiers like Alt, Control, Shift, and Super (OS symbol key).
///
/// Each variant represents a pair of [`KeyCode`]s, the left and right version of the modifier key,
/// allowing for handling modifiers regardless of which side is pressed.
///
/// # Behaviors
///
/// - Activation: Only if at least one corresponding keys is currently pressed down.
/// - Single-Axis Value:
///   - `1.0`: The input is currently active.
///   - `0.0`: The input is inactive.
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

    /// Create an [`ButtonlikeChord`] that includes this [`ModifierKey`] and the given `input`.
    #[inline]
    pub fn with(&self, other: impl Buttonlike) -> ButtonlikeChord {
        ButtonlikeChord::from_single(*self).with(other)
    }
}

#[serde_typetag]
impl UserInput for ModifierKey {
    /// [`ModifierKey`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Returns the two [`KeyCode`]s used by this [`ModifierKey`].
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![Box::new(self.left()), Box::new(self.right())])
    }

    /// Creates a [`RawInputs`] from two [`KeyCode`]s used by this [`ModifierKey`].
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes(self.keycodes())
    }
}

impl Buttonlike for ModifierKey {
    /// Checks if the specified modifier key is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        input_streams
            .keycodes
            .is_some_and(|keycodes| keycodes.any_pressed(self.keycodes()))
    }
}

/// A virtual single-axis control constructed from two [`KeyCode`]s.
/// One key represents the negative direction (left for the X-axis, down for the Y-axis),
/// while the other represents the positive direction (right for the X-axis, up for the Y-axis).
///
/// # Behaviors
///
/// - Raw Value:
///   - `-1.0`: Only the negative key is currently pressed.
///   - `1.0`: Only the positive key is currently pressed.
///   - `0.0`: Neither key is pressed, or both are pressed simultaneously.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithAxisProcessingPipelineExt`] for details.
/// - Activation: Only if the processed value is non-zero.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Define a virtual Y-axis using arrow "up" and "down" keys
/// let axis = KeyboardVirtualAxis::VERTICAL_ARROW_KEYS;
///
/// // Pressing either key activates the input
/// app.press_input(KeyCode::ArrowUp);
/// app.update();
/// assert_eq!(app.read_axis_value(axis), 1.0);
///
/// // You can configure a processing pipeline (e.g., doubling the value)
/// let doubled = KeyboardVirtualAxis::VERTICAL_ARROW_KEYS.sensitivity(2.0);
/// assert_eq!(app.read_axis_value(doubled), 2.0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct KeyboardVirtualAxis {
    /// The key that represents the negative direction.
    pub(crate) negative: KeyCode,

    /// The key that represents the positive direction.
    pub(crate) positive: KeyCode,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<AxisProcessor>,
}

impl KeyboardVirtualAxis {
    /// Creates a new [`KeyboardVirtualAxis`] with two given [`KeyCode`]s.
    /// No processing is applied to raw data from the gamepad.
    #[inline]
    pub fn new(negative: KeyCode, positive: KeyCode) -> Self {
        Self {
            negative,
            positive,
            processors: Vec::new(),
        }
    }

    /// The [`KeyboardVirtualAxis`] using the vertical arrow key mappings.
    ///
    /// - [`KeyCode::ArrowDown`] for negative direction.
    /// - [`KeyCode::ArrowUp`] for positive direction.
    pub const VERTICAL_ARROW_KEYS: Self = Self {
        negative: KeyCode::ArrowDown,
        positive: KeyCode::ArrowUp,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualAxis`] using the horizontal arrow key mappings.
    ///
    /// - [`KeyCode::ArrowLeft`] for negative direction.
    /// - [`KeyCode::ArrowRight`] for positive direction.
    pub const HORIZONTAL_ARROW_KEYS: Self = Self {
        negative: KeyCode::ArrowLeft,
        positive: KeyCode::ArrowRight,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualAxis`] using the common W/S key mappings.
    ///
    /// - [`KeyCode::KeyS`] for negative direction.
    /// - [`KeyCode::KeyW`] for positive direction.
    pub const WS: Self = Self {
        negative: KeyCode::KeyS,
        positive: KeyCode::KeyW,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualAxis`] using the common A/D key mappings.
    ///
    /// - [`KeyCode::KeyA`] for negative direction.
    /// - [`KeyCode::KeyD`] for positive direction.
    pub const AD: Self = Self {
        negative: KeyCode::KeyA,
        positive: KeyCode::KeyD,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualAxis`] using the vertical numpad key mappings.
    ///
    /// - [`KeyCode::Numpad2`] for negative direction.
    /// - [`KeyCode::Numpad8`] for positive direction.
    pub const VERTICAL_NUMPAD: Self = Self {
        negative: KeyCode::Numpad2,
        positive: KeyCode::Numpad8,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualAxis`] using the horizontal numpad key mappings.
    ///
    /// - [`KeyCode::Numpad4`] for negative direction.
    /// - [`KeyCode::Numpad6`] for positive direction.
    pub const HORIZONTAL_NUMPAD: Self = Self {
        negative: KeyCode::Numpad4,
        positive: KeyCode::Numpad6,
        processors: Vec::new(),
    };
}

#[serde_typetag]
impl UserInput for KeyboardVirtualAxis {
    /// [`KeyboardVirtualAxis`] acts as a virtual axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// [`KeyboardVirtualAxis`] represents a compositions of two [`KeyCode`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![Box::new(self.negative), Box::new(self.negative)])
    }

    /// Creates a [`RawInputs`] from two [`KeyCode`]s used by this axis.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes([self.negative, self.positive])
    }
}

impl Axislike for KeyboardVirtualAxis {
    /// Retrieves the current value of this axis after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let Some(keycodes) = input_streams.keycodes else {
            return 0.0;
        };

        let negative = f32::from(keycodes.pressed(self.negative));
        let positive = f32::from(keycodes.pressed(self.positive));
        let value = positive - negative;
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }
}

impl WithAxisProcessingPipelineExt for KeyboardVirtualAxis {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processors: impl IntoIterator<Item = AxisProcessor>,
    ) -> Self {
        self.processors = processors.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<AxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

/// A virtual single-axis control constructed from four [`KeyCode`]s.
/// Each key represents a specific direction (up, down, left, right),
/// functioning similarly to a directional pad (D-pad) on both X and Y axes,
/// and offering intermediate diagonals by means of two-key combinations.
///
/// # Behaviors
///
/// - Raw Value: Each axis behaves as follows:
///   - `-1.0`: Only the negative key is currently pressed (Down/Left).
///   - `1.0`: Only the positive key is currently pressed (Up/Right).
///   - `0.0`: Neither key is pressed, or both keys on the same axis are pressed simultaneously.
/// - Value Processing: Configure a pipeline to modify the raw value before use,
///     see [`WithDualAxisProcessingPipelineExt`] for details.
/// - Activation: Only if the processed value is non-zero on either axis.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins(InputPlugin);
///
/// // Define a virtual D-pad using the arrow keys
/// let input = KeyboardVirtualDPad::ARROW_KEYS;
///
/// // Pressing an arrow key activates the corresponding axis
/// app.press_input(KeyCode::ArrowUp);
/// app.update();
/// assert_eq!(app.read_dual_axis_values(input), Vec2::new(0.0, 1.0));
///
/// // You can configure a processing pipeline (e.g., doubling the Y value)
/// let doubled = KeyboardVirtualDPad::ARROW_KEYS.sensitivity_y(2.0);
/// assert_eq!(app.read_dual_axis_values(doubled), Vec2::new(0.0, 2.0));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct KeyboardVirtualDPad {
    /// The key for the upward direction.
    pub(crate) up: KeyCode,

    /// The key for the downward direction.
    pub(crate) down: KeyCode,

    /// The key for the leftward direction.
    pub(crate) left: KeyCode,

    /// The key for the rightward direction.
    pub(crate) right: KeyCode,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<DualAxisProcessor>,
}

impl KeyboardVirtualDPad {
    /// Creates a new [`KeyboardVirtualDPad`] with four given [`KeyCode`]s.
    /// No processing is applied to raw data from the keyboard.
    #[inline]
    pub fn new(up: KeyCode, down: KeyCode, left: KeyCode, right: KeyCode) -> Self {
        Self {
            up,
            down,
            left,
            right,
            processors: Vec::new(),
        }
    }

    /// The [`KeyboardVirtualDPad`] using the common arrow key mappings.
    ///
    /// - [`KeyCode::ArrowUp`] for upward direction.
    /// - [`KeyCode::ArrowDown`] for downward direction.
    /// - [`KeyCode::ArrowLeft`] for leftward direction.
    /// - [`KeyCode::ArrowRight`] for rightward direction.
    pub const ARROW_KEYS: Self = Self {
        up: KeyCode::ArrowUp,
        down: KeyCode::ArrowDown,
        left: KeyCode::ArrowLeft,
        right: KeyCode::ArrowRight,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualDPad`] using the common WASD key mappings.
    ///
    /// - [`KeyCode::KeyW`] for upward direction.
    /// - [`KeyCode::KeyS`] for downward direction.
    /// - [`KeyCode::KeyA`] for leftward direction.
    /// - [`KeyCode::KeyD`] for rightward direction.
    pub const WASD: Self = Self {
        up: KeyCode::KeyW,
        down: KeyCode::KeyS,
        left: KeyCode::KeyA,
        right: KeyCode::KeyD,
        processors: Vec::new(),
    };

    /// The [`KeyboardVirtualDPad`] using the common numpad key mappings.
    ///
    /// - [`KeyCode::Numpad8`] for upward direction.
    /// - [`KeyCode::Numpad2`] for downward direction.
    /// - [`KeyCode::Numpad4`] for leftward direction.
    /// - [`KeyCode::Numpad6`] for rightward direction.
    pub const NUMPAD: Self = Self {
        up: KeyCode::Numpad8,
        down: KeyCode::Numpad2,
        left: KeyCode::Numpad4,
        right: KeyCode::Numpad6,
        processors: Vec::new(),
    };

    /// Retrieves the current X and Y values of this D-pad after processing by the associated processors.
    #[must_use]
    #[inline]
    fn processed_value(&self, input_streams: &InputStreams) -> Vec2 {
        let Some(keycodes) = input_streams.keycodes else {
            return Vec2::ZERO;
        };

        let up = f32::from(keycodes.pressed(self.up));
        let down = f32::from(keycodes.pressed(self.down));
        let left = f32::from(keycodes.pressed(self.left));
        let right = f32::from(keycodes.pressed(self.right));
        let value = Vec2::new(right - left, up - down);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }
}

#[serde_typetag]
impl UserInput for KeyboardVirtualDPad {
    /// [`KeyboardVirtualDPad`] acts as a virtual dual-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// [`KeyboardVirtualDPad`] represents a compositions of four [`KeyCode`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(self.up),
            Box::new(self.down),
            Box::new(self.left),
            Box::new(self.right),
        ])
    }

    /// Creates a [`RawInputs`] from four [`KeyCode`]s used by this D-pad.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        RawInputs::from_keycodes([self.up, self.down, self.left, self.right])
    }
}

impl DualAxislike for KeyboardVirtualDPad {
    /// Retrieves the current X and Y values of this D-pad after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2 {
        self.processed_value(input_streams)
    }
}

impl WithDualAxisProcessingPipelineExt for KeyboardVirtualDPad {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processors: impl IntoIterator<Item = DualAxisProcessor>,
    ) -> Self {
        self.processors = processors.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_mocking::MockInput;
    use crate::plugin::AccumulatorPlugin;
    use crate::raw_inputs::RawInputs;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin).add_plugins(AccumulatorPlugin);
        app
    }

    #[test]
    fn test_keyboard_input() {
        let up = KeyCode::ArrowUp;
        assert_eq!(up.kind(), InputControlKind::Button);
        assert_eq!(up.raw_inputs(), RawInputs::from_keycodes([up]));

        let left = KeyCode::ArrowLeft;
        assert_eq!(left.kind(), InputControlKind::Button);
        assert_eq!(left.raw_inputs(), RawInputs::from_keycodes([left]));

        let alt = ModifierKey::Alt;
        assert_eq!(alt.kind(), InputControlKind::Button);
        let alt_raw_inputs = RawInputs::from_keycodes([KeyCode::AltLeft, KeyCode::AltRight]);
        assert_eq!(alt.raw_inputs(), alt_raw_inputs);

        let arrow_y = KeyboardVirtualAxis::VERTICAL_ARROW_KEYS;
        assert_eq!(arrow_y.kind(), InputControlKind::Axis);
        let raw_inputs = RawInputs::from_keycodes([KeyCode::ArrowDown, KeyCode::ArrowUp]);
        assert_eq!(arrow_y.raw_inputs(), raw_inputs);

        let arrows = KeyboardVirtualDPad::ARROW_KEYS;
        assert_eq!(arrows.kind(), InputControlKind::DualAxis);
        let raw_inputs = RawInputs::from_keycodes([
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        ]);
        assert_eq!(arrows.raw_inputs(), raw_inputs);

        // No inputs
        let zeros = Vec2::new(0.0, 0.0);
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(!alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), 0.0);
        assert_eq!(arrows.axis_pair(&inputs), zeros);

        // Press arrow up
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowUp);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(!alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), data.y);
        assert_eq!(arrows.axis_pair(&inputs), data);

        // Press arrow down
        let data = Vec2::new(0.0, -1.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowDown);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(!alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), data.y);
        assert_eq!(arrows.axis_pair(&inputs), data);

        // Press arrow left
        let data = Vec2::new(-1.0, 0.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowLeft);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(left.pressed(&inputs));
        assert!(!alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), 0.0);
        assert_eq!(arrows.axis_pair(&inputs), data);

        // Press arrow down and arrow up
        let mut app = test_app();
        app.press_input(KeyCode::ArrowDown);
        app.press_input(KeyCode::ArrowUp);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(!alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), 0.0);
        assert_eq!(arrows.axis_pair(&inputs), zeros);

        // Press arrow left and arrow up
        let data = Vec2::new(-1.0, 1.0);
        let mut app = test_app();
        app.press_input(KeyCode::ArrowLeft);
        app.press_input(KeyCode::ArrowUp);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(up.pressed(&inputs));
        assert!(left.pressed(&inputs));
        assert!(!alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), data.y);
        assert_eq!(arrows.axis_pair(&inputs), data);

        // Press left Alt
        let mut app = test_app();
        app.press_input(KeyCode::AltLeft);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), 0.0);
        assert_eq!(arrows.axis_pair(&inputs), zeros);

        // Press right Alt
        let mut app = test_app();
        app.press_input(KeyCode::AltRight);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);

        assert!(!up.pressed(&inputs));
        assert!(!left.pressed(&inputs));
        assert!(alt.pressed(&inputs));
        assert_eq!(arrow_y.value(&inputs), 0.0);
        assert_eq!(arrows.axis_pair(&inputs), zeros);
    }
}
