//! Keyboard inputs

use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::{ButtonInput, ButtonState};
use bevy::prelude::{Entity, Events, Gamepad, KeyCode, Reflect, Res, ResMut, Vec2, Vec3, World};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessingPipelineExt,
    WithDualAxisProcessingPipelineExt,
};
use crate::user_input::{ButtonlikeChord, TripleAxislike, UserInput};
use crate::InputControlKind;

use super::updating::{CentralInputStore, UpdatableInput};
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
}

impl UpdatableInput for KeyCode {
    type SourceData = ButtonInput<KeyCode>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: Res<Self::SourceData>,
    ) {
        for key in source_data.get_pressed() {
            central_input_store.update_buttonlike(*key, true);
        }

        for key in source_data.get_just_released() {
            central_input_store.update_buttonlike(*key, false);
        }
    }
}

impl Buttonlike for KeyCode {
    /// Checks if the specified key is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> bool {
        input_store.pressed(self)
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Pressed`].
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn press(&self, world: &mut World) {
        let mut events = world.resource_mut::<Events<KeyboardInput>>();
        events.send(KeyboardInput {
            key_code: *self,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            window: Entity::PLACEHOLDER,
        });
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Released`].
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn release(&self, world: &mut World) {
        let mut events = world.resource_mut::<Events<KeyboardInput>>();
        events.send(KeyboardInput {
            key_code: *self,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Released,
            window: Entity::PLACEHOLDER,
        });
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
}

impl Buttonlike for ModifierKey {
    /// Checks if the specified modifier key is currently pressed down.
    #[must_use]
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> bool {
        input_store.pressed(&self.left()) || input_store.pressed(&self.right())
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Pressed`].
    ///
    /// The left and right keys will be pressed simultaneously.
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn press(&self, world: &mut World) {
        self.left().press(world);
        self.right().press(world);
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Released`].
    ///
    /// The left and right keys will be released simultaneously.
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn release(&self, world: &mut World) {
        self.left().release(world);
        self.right().release(world);
    }
}

/// A virtual single-axis control constructed from two [`KeyCode`]s.
/// One key represents the negative direction (left for the X-axis, down for the Y-axis),
/// while the other represents the positive direction (right for the X-axis, up for the Y-axis).
///
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithAxisProcessingPipelineExt`] for details.
///
/// The raw value is determined based on the state of the associated buttons:
/// - `-1.0` if only the negative button is currently pressed.
/// - `1.0` if only the positive button is currently pressed.
/// - `0.0` if neither button is pressed, or both are pressed simultaneously.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Define a virtual Y-axis using arrow "up" and "down" keys
/// let axis = KeyboardVirtualAxis::VERTICAL_ARROW_KEYS;
///
/// // Pressing either key activates the input
/// KeyCode::ArrowUp.press(app.world_mut());
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
}

impl Axislike for KeyboardVirtualAxis {
    /// Retrieves the current value of this axis after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> f32 {
        let negative = f32::from(input_store.pressed(&self.negative));
        let positive = f32::from(input_store.pressed(&self.positive));
        let value = positive - negative;
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Sends a [`KeyboardInput`] event.
    ///
    /// If the value is negative, the negative button is pressed.
    /// If the value is positive, the positive button is pressed.
    /// If the value is zero, neither button is pressed.
    fn set_value(&self, world: &mut World, value: f32) {
        if value < 0.0 {
            self.negative.press(world);
        } else if value > 0.0 {
            self.positive.press(world);
        }
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

/// A virtual dual-axis control constructed from four [`KeyCode`]s.
/// Each key represents a specific direction (up, down, left, right),
/// functioning similarly to a directional pad (D-pad) on both X and Y axes,
/// and offering intermediate diagonals by means of two-key combinations.
///
/// # Value Processing
///
/// You can customize how the values are processed using a pipeline of processors.
/// See [`WithDualAxisProcessingPipelineExt`] for details.
///
/// The raw axis values are determined based on the state of the associated buttons:
/// - `-1.0` if only the negative button is currently pressed (Down/Left).
/// - `1.0` if only the positive button is currently pressed (Up/Right).
/// - `0.0` if neither button is pressed, or both are pressed simultaneously.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Define a virtual D-pad using the arrow keys
/// let input = KeyboardVirtualDPad::ARROW_KEYS;
///
/// // Pressing an arrow key activates the corresponding axis
/// KeyCode::ArrowUp.press(app.world_mut());
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
}

impl DualAxislike for KeyboardVirtualDPad {
    /// Retrieves the current X and Y values of this D-pad after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> Vec2 {
        let up = f32::from(input_store.pressed(&self.up));
        let down = f32::from(input_store.pressed(&self.down));
        let left = f32::from(input_store.pressed(&self.left));
        let right = f32::from(input_store.pressed(&self.right));
        let value = Vec2::new(right - left, up - down);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Presses the corresponding buttons based on the quadrant of the given value.
    fn set_axis_pair(&self, world: &mut World, value: Vec2) {
        if value.x < 0.0 {
            self.left.press(world);
        } else if value.x > 0.0 {
            self.right.press(world);
        }

        if value.y < 0.0 {
            self.down.press(world);
        } else if value.y > 0.0 {
            self.up.press(world);
        }
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

/// A virtual triple-axis control constructed from six [`KeyCode`]s.
/// Each key represents a specific direction (up, down, left, right, forward, backward),
/// functioning similarly to a three-dimensional directional pad (D-pad) on all X, Y, and Z axes,
/// and offering intermediate diagonals by means of two/three-key combinations.
///
/// The raw axis values are determined based on the state of the associated buttons:
/// - `-1.0` if only the negative button is currently pressed (Down/Left/Forward).
/// - `1.0` if only the positive button is currently pressed (Up/Right/Backward).
/// - `0.0` if neither button is pressed, or both are pressed simultaneously.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct KeyboardVirtualDPad3D {
    /// The key for the upward direction.
    pub(crate) up: KeyCode,

    /// The key for the downward direction.
    pub(crate) down: KeyCode,

    /// The key for the leftward direction.
    pub(crate) left: KeyCode,

    /// The key for the rightward direction.
    pub(crate) right: KeyCode,

    /// The key for the forward direction.
    pub(crate) forward: KeyCode,

    /// The key for the backward direction.
    pub(crate) backward: KeyCode,
}

impl KeyboardVirtualDPad3D {
    /// Creates a new [`KeyboardVirtualDPad3D`] with six given [`KeyCode`]s.
    /// No processing is applied to raw data from the keyboard.
    #[inline]
    pub fn new(
        up: KeyCode,
        down: KeyCode,
        left: KeyCode,
        right: KeyCode,
        forward: KeyCode,
        backward: KeyCode,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            forward,
            backward,
        }
    }
}

#[serde_typetag]
impl UserInput for KeyboardVirtualDPad3D {
    /// [`KeyboardVirtualDPad3D`] acts as a virtual triple-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::TripleAxis
    }

    /// [`KeyboardVirtualDPad3D`] represents a compositions of six [`KeyCode`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            Box::new(self.up),
            Box::new(self.down),
            Box::new(self.left),
            Box::new(self.right),
            Box::new(self.forward),
            Box::new(self.backward),
        ])
    }
}

impl TripleAxislike for KeyboardVirtualDPad3D {
    /// Retrieves the current X, Y, and Z values of this D-pad.
    #[must_use]
    #[inline]
    fn axis_triple(&self, input_store: &CentralInputStore, _gamepad: Gamepad) -> Vec3 {
        let up = f32::from(input_store.pressed(&self.up));
        let down = f32::from(input_store.pressed(&self.down));
        let left = f32::from(input_store.pressed(&self.left));
        let right = f32::from(input_store.pressed(&self.right));
        let forward = f32::from(input_store.pressed(&self.left));
        let back = f32::from(input_store.pressed(&self.right));
        Vec3::new(right - left, up - down, back - forward)
    }

    /// Presses the corresponding buttons based on the octant of the given value.
    fn set_axis_triple(&self, world: &mut World, value: Vec3) {
        if value.x < 0.0 {
            self.left.press(world);
        } else if value.x > 0.0 {
            self.right.press(world);
        }

        if value.y < 0.0 {
            self.down.press(world);
        } else if value.y > 0.0 {
            self.up.press(world);
        }

        if value.z < 0.0 {
            self.forward.press(world);
        } else if value.z > 0.0 {
            self.backward.press(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .add_plugins((AccumulatorPlugin, CentralInputStorePlugin));
        app
    }

    #[test]
    fn test_keyboard_input() {
        let up = KeyCode::ArrowUp;
        assert_eq!(up.kind(), InputControlKind::Button);

        let left = KeyCode::ArrowLeft;
        assert_eq!(left.kind(), InputControlKind::Button);

        let alt = ModifierKey::Alt;
        assert_eq!(alt.kind(), InputControlKind::Button);

        let arrow_y = KeyboardVirtualAxis::VERTICAL_ARROW_KEYS;
        assert_eq!(arrow_y.kind(), InputControlKind::Axis);

        let arrows = KeyboardVirtualDPad::ARROW_KEYS;
        assert_eq!(arrows.kind(), InputControlKind::DualAxis);

        // No inputs
        let zeros = Vec2::new(0.0, 0.0);
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        let gamepad = Gamepad::new(0);

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), 0.0);
        assert_eq!(arrows.axis_pair(inputs, gamepad), zeros);

        // Press arrow up
        let data = Vec2::new(0.0, 1.0);
        let mut app = test_app();
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), data.y);
        assert_eq!(arrows.axis_pair(inputs, gamepad), data);

        // Press arrow down
        let data = Vec2::new(0.0, -1.0);
        let mut app = test_app();
        KeyCode::ArrowDown.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), data.y);
        assert_eq!(arrows.axis_pair(inputs, gamepad), data);

        // Press arrow left
        let data = Vec2::new(-1.0, 0.0);
        let mut app = test_app();
        KeyCode::ArrowLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), 0.0);
        assert_eq!(arrows.axis_pair(inputs, gamepad), data);

        // Press arrow down and arrow up
        let mut app = test_app();
        KeyCode::ArrowDown.press(app.world_mut());
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), 0.0);
        assert_eq!(arrows.axis_pair(inputs, gamepad), zeros);

        // Press arrow left and arrow up
        let data = Vec2::new(-1.0, 1.0);
        let mut app = test_app();
        KeyCode::ArrowLeft.press(app.world_mut());
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(up.pressed(inputs, gamepad));
        assert!(left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), data.y);
        assert_eq!(arrows.axis_pair(inputs, gamepad), data);

        // Press left Alt
        let mut app = test_app();
        KeyCode::AltLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), 0.0);
        assert_eq!(arrows.axis_pair(inputs, gamepad), zeros);

        // Press right Alt
        let mut app = test_app();
        KeyCode::AltRight.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(alt.pressed(inputs, gamepad));
        assert_eq!(arrow_y.value(inputs, gamepad), 0.0);
        assert_eq!(arrows.axis_pair(inputs, gamepad), zeros);
    }
}
