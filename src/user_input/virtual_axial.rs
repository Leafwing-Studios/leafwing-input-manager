//! This module contains [`VirtualAxis`], [`VirtualDPad`], and [`VirtualDPad3D`].

use crate as leafwing_input_manager;
use crate::clashing_inputs::BasicInputs;
use crate::input_processing::{
    AxisProcessor, DualAxisProcessor, WithAxisProcessingPipelineExt,
    WithDualAxisProcessingPipelineExt,
};
use crate::prelude::updating::CentralInputStore;
use crate::prelude::{Axislike, DualAxislike, TripleAxislike, UserInput};
use crate::user_input::Buttonlike;
use crate::InputControlKind;
use bevy::math::{Vec2, Vec3};
#[cfg(feature = "gamepad")]
use bevy::prelude::GamepadButtonType;
#[cfg(feature = "keyboard")]
use bevy::prelude::KeyCode;
use bevy::prelude::{Gamepad, Reflect, World};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

/// A virtual single-axis control constructed from two [`Buttonlike`]s.
/// One button represents the negative direction (left for the X-axis, down for the Y-axis),
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
/// let axis = VirtualAxis::vertical_arrow_keys();
///
/// // Pressing either key activates the input
/// KeyCode::ArrowUp.press(app.world_mut());
/// app.update();
/// assert_eq!(app.read_axis_value(axis), 1.0);
///
/// // You can configure a processing pipeline (e.g., doubling the value)
/// let doubled = VirtualAxis::vertical_arrow_keys().sensitivity(2.0);
/// assert_eq!(app.read_axis_value(doubled), 2.0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct VirtualAxis {
    /// The button that represents the negative direction.
    pub(crate) negative: Box<dyn Buttonlike>,

    /// The button that represents the positive direction.
    pub(crate) positive: Box<dyn Buttonlike>,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<AxisProcessor>,
}

impl VirtualAxis {
    /// Creates a new [`VirtualAxis`] with two given [`Buttonlike`]s.
    /// No processing is applied to raw data.
    #[inline]
    pub fn new(negative: impl Buttonlike, positive: impl Buttonlike) -> Self {
        Self {
            negative: Box::new(negative),
            positive: Box::new(positive),
            processors: Vec::new(),
        }
    }

    /// The [`VirtualAxis`] using the vertical arrow key mappings.
    ///
    /// - [`KeyCode::ArrowDown`] for negative direction.
    /// - [`KeyCode::ArrowUp`] for positive direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn vertical_arrow_keys() -> Self {
        Self::new(KeyCode::ArrowDown, KeyCode::ArrowUp)
    }

    /// The [`VirtualAxis`] using the horizontal arrow key mappings.
    ///
    /// - [`KeyCode::ArrowLeft`] for negative direction.
    /// - [`KeyCode::ArrowRight`] for positive direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn horizontal_arrow_keys() -> Self {
        Self::new(KeyCode::ArrowLeft, KeyCode::ArrowRight)
    }

    /// The [`VirtualAxis`] using the common W/S key mappings.
    ///
    /// - [`KeyCode::KeyS`] for negative direction.
    /// - [`KeyCode::KeyW`] for positive direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn ws() -> Self {
        Self::new(KeyCode::KeyS, KeyCode::KeyW)
    }

    /// The [`VirtualAxis`] using the common A/D key mappings.
    ///
    /// - [`KeyCode::KeyA`] for negative direction.
    /// - [`KeyCode::KeyD`] for positive direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn ad() -> Self {
        Self::new(KeyCode::KeyA, KeyCode::KeyD)
    }

    /// The [`VirtualAxis`] using the vertical numpad key mappings.
    ///
    /// - [`KeyCode::Numpad2`] for negative direction.
    /// - [`KeyCode::Numpad8`] for positive direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn vertical_numpad() -> Self {
        Self::new(KeyCode::Numpad2, KeyCode::Numpad8)
    }

    /// The [`VirtualAxis`] using the horizontal numpad key mappings.
    ///
    /// - [`KeyCode::Numpad4`] for negative direction.
    /// - [`KeyCode::Numpad6`] for positive direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn horizontal_numpad() -> Self {
        Self::new(KeyCode::Numpad4, KeyCode::Numpad6)
    }

    /// The [`VirtualAxis`] using the horizontal D-Pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::DPadLeft`] for negative direction.
    /// - [`GamepadButtonType::DPadRight`] for positive direction.
    #[cfg(feature = "gamepad")]
    #[inline]
    pub fn dpad_x() -> Self {
        Self::new(GamepadButtonType::DPadLeft, GamepadButtonType::DPadRight)
    }

    /// The [`VirtualAxis`] using the vertical D-Pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::DPadDown`] for negative direction.
    /// - [`GamepadButtonType::DPadUp`] for positive direction.
    #[cfg(feature = "gamepad")]
    #[inline]
    pub fn dpad_y() -> Self {
        Self::new(GamepadButtonType::DPadDown, GamepadButtonType::DPadUp)
    }

    /// The [`VirtualAxis`] using the horizontal action pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::West`] for negative direction.
    /// - [`GamepadButtonType::East`] for positive direction.
    #[cfg(feature = "gamepad")]
    #[inline]
    pub fn action_pad_x() -> Self {
        Self::new(GamepadButtonType::West, GamepadButtonType::East)
    }

    /// The [`VirtualAxis`] using the vertical action pad button mappings.
    /// No processing is applied to raw data from the gamepad.
    ///
    /// - [`GamepadButtonType::South`] for negative direction.
    /// - [`GamepadButtonType::North`] for positive direction.
    #[cfg(feature = "gamepad")]
    #[inline]
    pub fn action_pad_y() -> Self {
        Self::new(GamepadButtonType::South, GamepadButtonType::North)
    }
}

impl UserInput for VirtualAxis {
    /// [`VirtualAxis`] acts as a virtual axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// [`VirtualAxis`] represents a compositions of two buttons.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![self.negative.clone(), self.positive.clone()])
    }
}

#[serde_typetag]
impl Axislike for VirtualAxis {
    /// Retrieves the current value of this axis after processing by the associated processors.
    #[must_use]
    #[inline]
    fn value(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> f32 {
        let negative = f32::from(self.negative.pressed(input_store, gamepad));
        let positive = f32::from(self.positive.pressed(input_store, gamepad));
        let value = positive - negative;
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Presses the corresponding button based on the given value.
    ///
    /// If the value is negative, the negative button is pressed.
    /// If the value is positive, the positive button is pressed.
    /// If the value is zero, neither button is pressed.
    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Gamepad>) {
        if value < 0.0 {
            self.negative.press_as_gamepad(world, gamepad);
        } else if value > 0.0 {
            self.positive.press_as_gamepad(world, gamepad);
        }
    }
}

impl WithAxisProcessingPipelineExt for VirtualAxis {
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

/// A virtual dual-axis control constructed from four [`Buttonlike`]s.
/// Each button represents a specific direction (up, down, left, right),
/// functioning similarly to a directional pad (D-pad) on both X and Y axes,
/// and offering intermediate diagonals by means of two-button combinations.
///
/// By default, it reads from **any connected gamepad**.
/// Use the [`InputMap::set_gamepad`](crate::input_map::InputMap::set_gamepad) for specific ones.
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
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));
///
/// // Define a virtual D-pad using the WASD keys
/// let input = VirtualDPad::wasd();
///
/// // Pressing the W key activates the corresponding axis
/// KeyCode::KeyW.press(app.world_mut());
/// app.update();
/// assert_eq!(app.read_dual_axis_values(input), Vec2::new(0.0, 1.0));
///
/// // You can configure a processing pipeline (e.g., doubling the Y value)
/// let doubled = VirtualDPad::wasd().sensitivity_y(2.0);
/// assert_eq!(app.read_dual_axis_values(doubled), Vec2::new(0.0, 2.0));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct VirtualDPad {
    /// The button for the upward direction.
    pub(crate) up: Box<dyn Buttonlike>,

    /// The button for the downward direction.
    pub(crate) down: Box<dyn Buttonlike>,

    /// The button for the leftward direction.
    pub(crate) left: Box<dyn Buttonlike>,

    /// The button for the rightward direction.
    pub(crate) right: Box<dyn Buttonlike>,

    /// A processing pipeline that handles input values.
    pub(crate) processors: Vec<DualAxisProcessor>,
}

impl VirtualDPad {
    /// Creates a new [`VirtualDPad`] with four given [`Buttonlike`]s.
    /// Each button represents a specific direction (up, down, left, right).
    #[inline]
    pub fn new(
        up: impl Buttonlike,
        down: impl Buttonlike,
        left: impl Buttonlike,
        right: impl Buttonlike,
    ) -> Self {
        Self {
            up: Box::new(up),
            down: Box::new(down),
            left: Box::new(left),
            right: Box::new(right),
            processors: Vec::new(),
        }
    }

    /// The [`VirtualDPad`] using the common arrow key mappings.
    ///
    /// - [`KeyCode::ArrowUp`] for upward direction.
    /// - [`KeyCode::ArrowDown`] for downward direction.
    /// - [`KeyCode::ArrowLeft`] for leftward direction.
    /// - [`KeyCode::ArrowRight`] for rightward direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn arrow_keys() -> Self {
        Self::new(
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
        )
    }

    /// The [`VirtualDPad`] using the common WASD key mappings.
    ///
    /// - [`KeyCode::KeyW`] for upward direction.
    /// - [`KeyCode::KeyS`] for downward direction.
    /// - [`KeyCode::KeyA`] for leftward direction.
    /// - [`KeyCode::KeyD`] for rightward direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn wasd() -> Self {
        Self::new(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD)
    }

    /// The [`VirtualDPad`] using the common numpad key mappings.
    ///
    /// - [`KeyCode::Numpad8`] for upward direction.
    /// - [`KeyCode::Numpad2`] for downward direction.
    /// - [`KeyCode::Numpad4`] for leftward direction.
    /// - [`KeyCode::Numpad6`] for rightward direction.
    #[cfg(feature = "keyboard")]
    #[inline]
    pub fn numpad() -> Self {
        Self::new(
            KeyCode::Numpad8,
            KeyCode::Numpad2,
            KeyCode::Numpad4,
            KeyCode::Numpad6,
        )
    }

    /// Creates a new [`VirtualDPad`] using the common D-Pad button mappings.
    ///
    /// - [`GamepadButtonType::DPadUp`] for upward direction.
    /// - [`GamepadButtonType::DPadDown`] for downward direction.
    /// - [`GamepadButtonType::DPadLeft`] for leftward direction.
    /// - [`GamepadButtonType::DPadRight`] for rightward direction.
    #[cfg(feature = "gamepad")]
    #[inline]
    pub fn dpad() -> Self {
        Self::new(
            GamepadButtonType::DPadUp,
            GamepadButtonType::DPadDown,
            GamepadButtonType::DPadLeft,
            GamepadButtonType::DPadRight,
        )
    }

    /// Creates a new [`VirtualDPad`] using the common action pad button mappings.
    ///
    /// - [`GamepadButtonType::North`] for upward direction.
    /// - [`GamepadButtonType::South`] for downward direction.
    /// - [`GamepadButtonType::West`] for leftward direction.
    /// - [`GamepadButtonType::East`] for rightward direction.
    #[cfg(feature = "gamepad")]
    #[inline]
    pub fn action_pad() -> Self {
        Self::new(
            GamepadButtonType::North,
            GamepadButtonType::South,
            GamepadButtonType::West,
            GamepadButtonType::East,
        )
    }
}

impl UserInput for VirtualDPad {
    /// [`VirtualDPad`] acts as a dual-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// Returns the four [`GamepadButtonType`]s used by this D-pad.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            self.up.clone(),
            self.down.clone(),
            self.left.clone(),
            self.right.clone(),
        ])
    }
}

#[serde_typetag]
impl DualAxislike for VirtualDPad {
    /// Retrieves the current X and Y values of this D-pad after processing by the associated processors.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> Vec2 {
        let up = f32::from(self.up.pressed(input_store, gamepad));
        let down = f32::from(self.down.pressed(input_store, gamepad));
        let left = f32::from(self.left.pressed(input_store, gamepad));
        let right = f32::from(self.right.pressed(input_store, gamepad));
        let value = Vec2::new(right - left, up - down);
        self.processors
            .iter()
            .fold(value, |value, processor| processor.process(value))
    }

    /// Presses the corresponding buttons based on the quadrant of the given value.
    fn set_axis_pair_as_gamepad(&self, world: &mut World, value: Vec2, gamepad: Option<Gamepad>) {
        if value.x < 0.0 {
            self.left.press_as_gamepad(world, gamepad);
        } else if value.x > 0.0 {
            self.right.press_as_gamepad(world, gamepad);
        }

        if value.y < 0.0 {
            self.down.press_as_gamepad(world, gamepad);
        } else if value.y > 0.0 {
            self.up.press_as_gamepad(world, gamepad);
        }
    }
}

impl WithDualAxisProcessingPipelineExt for VirtualDPad {
    #[inline]
    fn reset_processing_pipeline(mut self) -> Self {
        self.processors.clear();
        self
    }

    #[inline]
    fn replace_processing_pipeline(
        mut self,
        processor: impl IntoIterator<Item = DualAxisProcessor>,
    ) -> Self {
        self.processors = processor.into_iter().collect();
        self
    }

    #[inline]
    fn with_processor(mut self, processor: impl Into<DualAxisProcessor>) -> Self {
        self.processors.push(processor.into());
        self
    }
}

/// A virtual triple-axis control constructed from six [`Buttonlike`]s.
/// Each button represents a specific direction (up, down, left, right, forward, backward),
/// functioning similarly to a three-dimensional directional pad (D-pad) on all X, Y, and Z axes,
/// and offering intermediate diagonals by means of two/three-key combinations.
///
/// The raw axis values are determined based on the state of the associated buttons:
/// - `-1.0` if only the negative button is currently pressed (Down/Left/Forward).
/// - `1.0` if only the positive button is currently pressed (Up/Right/Backward).
/// - `0.0` if neither button is pressed, or both are pressed simultaneously.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct VirtualDPad3D {
    /// The button for the upward direction.
    pub(crate) up: Box<dyn Buttonlike>,

    /// The button for the downward direction.
    pub(crate) down: Box<dyn Buttonlike>,

    /// The button for the leftward direction.
    pub(crate) left: Box<dyn Buttonlike>,

    /// The button for the rightward direction.
    pub(crate) right: Box<dyn Buttonlike>,

    /// The button for the forward direction.
    pub(crate) forward: Box<dyn Buttonlike>,

    /// The button for the backward direction.
    pub(crate) backward: Box<dyn Buttonlike>,
}

impl VirtualDPad3D {
    /// Creates a new [`VirtualDPad3D`] with six given [`Buttonlike`]s.
    /// Each button represents a specific direction (up, down, left, right, forward, backward).
    #[inline]
    pub fn new(
        up: impl Buttonlike,
        down: impl Buttonlike,
        left: impl Buttonlike,
        right: impl Buttonlike,
        forward: impl Buttonlike,
        backward: impl Buttonlike,
    ) -> Self {
        Self {
            up: Box::new(up),
            down: Box::new(down),
            left: Box::new(left),
            right: Box::new(right),
            forward: Box::new(forward),
            backward: Box::new(backward),
        }
    }
}

impl UserInput for VirtualDPad3D {
    /// [`VirtualDPad3D`] acts as a virtual triple-axis input.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::TripleAxis
    }

    /// [`VirtualDPad3D`] represents a compositions of six [`Buttonlike`]s.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![
            self.up.clone(),
            self.down.clone(),
            self.left.clone(),
            self.right.clone(),
            self.forward.clone(),
            self.backward.clone(),
        ])
    }
}

#[serde_typetag]
impl TripleAxislike for VirtualDPad3D {
    /// Retrieves the current X, Y, and Z values of this D-pad.
    #[must_use]
    #[inline]
    fn axis_triple(&self, input_store: &CentralInputStore, gamepad: Gamepad) -> Vec3 {
        let up = f32::from(self.up.pressed(input_store, gamepad));
        let down = f32::from(self.down.pressed(input_store, gamepad));
        let left = f32::from(self.left.pressed(input_store, gamepad));
        let right = f32::from(self.right.pressed(input_store, gamepad));
        let forward = f32::from(self.forward.pressed(input_store, gamepad));
        let backward = f32::from(self.backward.pressed(input_store, gamepad));
        Vec3::new(right - left, up - down, backward - forward)
    }

    /// Presses the corresponding buttons based on the octant of the given value.
    fn set_axis_triple_as_gamepad(&self, world: &mut World, value: Vec3, gamepad: Option<Gamepad>) {
        if value.x < 0.0 {
            self.left.press_as_gamepad(world, gamepad);
        } else if value.x > 0.0 {
            self.right.press_as_gamepad(world, gamepad);
        }

        if value.y < 0.0 {
            self.down.press_as_gamepad(world, gamepad);
        } else if value.y > 0.0 {
            self.up.press_as_gamepad(world, gamepad);
        }

        if value.z < 0.0 {
            self.forward.press_as_gamepad(world, gamepad);
        } else if value.z > 0.0 {
            self.backward.press_as_gamepad(world, gamepad);
        }
    }
}

#[cfg(feature = "keyboard")]
#[cfg(test)]
mod tests {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
    use crate::prelude::updating::CentralInputStore;
    use crate::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .add_plugins((AccumulatorPlugin, CentralInputStorePlugin));
        app
    }

    #[test]
    fn test_virtual() {
        let x = VirtualAxis::horizontal_arrow_keys();
        let xy = VirtualDPad::arrow_keys();
        let xyz = VirtualDPad3D::new(
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            KeyCode::KeyF,
            KeyCode::KeyB,
        );

        // No inputs
        let mut app = test_app();
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        let gamepad = Gamepad::new(0);

        assert_eq!(x.value(inputs, gamepad), 0.0);
        assert_eq!(xy.axis_pair(inputs, gamepad), Vec2::ZERO);
        assert_eq!(xyz.axis_triple(inputs, gamepad), Vec3::ZERO);

        // Press arrow left
        let mut app = test_app();
        KeyCode::ArrowLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert_eq!(x.value(inputs, gamepad), -1.0);
        assert_eq!(xy.axis_pair(inputs, gamepad), Vec2::new(-1.0, 0.0));
        assert_eq!(xyz.axis_triple(inputs, gamepad), Vec3::new(-1.0, 0.0, 0.0));

        // Press arrow up
        let mut app = test_app();
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert_eq!(x.value(inputs, gamepad), 0.0);
        assert_eq!(xy.axis_pair(inputs, gamepad), Vec2::new(0.0, 1.0));
        assert_eq!(xyz.axis_triple(inputs, gamepad), Vec3::new(0.0, 1.0, 0.0));

        // Press arrow right
        let mut app = test_app();
        KeyCode::ArrowRight.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert_eq!(x.value(inputs, gamepad), 1.0);
        assert_eq!(xy.axis_pair(inputs, gamepad), Vec2::new(1.0, 0.0));
        assert_eq!(xyz.axis_triple(inputs, gamepad), Vec3::new(1.0, 0.0, 0.0));

        // Press key B
        let mut app = test_app();
        KeyCode::KeyB.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert_eq!(x.value(inputs, gamepad), 0.0);
        assert_eq!(xy.axis_pair(inputs, gamepad), Vec2::new(0.0, 0.0));
        assert_eq!(xyz.axis_triple(inputs, gamepad), Vec3::new(0.0, 0.0, 1.0));
    }
}
