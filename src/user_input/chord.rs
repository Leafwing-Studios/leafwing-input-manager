//! This module contains [`ButtonlikeChord`] and its impls.

use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, Reflect, World};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::clashing_inputs::BasicInputs;
use crate::user_input::{Buttonlike, TripleAxislike, UserInput};
use crate::InputControlKind;

use super::updating::CentralInputStore;
use super::{Axislike, DualAxislike};

/// A combined input that groups multiple [`Buttonlike`]s together,
/// allowing you to define complex input combinations like hotkeys, shortcuts, and macros.
///
/// A chord is pressed only if all its constituent buttons are pressed simultaneously.
///
/// Adding duplicate buttons within a chord will ignore the extras,
/// preventing redundant data fetching from multiple instances of the same input.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::CentralInputStorePlugin;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::user_input::testing_utils::FetchUserInput;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, CentralInputStorePlugin));
///
/// // Define a chord using A and B keys
/// let input = ButtonlikeChord::new([KeyCode::KeyA, KeyCode::KeyB]);
///
/// // Pressing only one key doesn't activate the input
/// KeyCode::KeyA.press(app.world_mut());
/// app.update();
/// assert!(!app.read_pressed(input.clone()));
///
/// // Pressing both keys activates the input
/// KeyCode::KeyA.press(app.world_mut());
/// KeyCode::KeyB.press(app.world_mut());
/// app.update();
/// assert!(app.read_pressed(input.clone()));
/// ```
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct ButtonlikeChord(
    // Note: We can't use a HashSet here because of
    // https://users.rust-lang.org/t/hash-not-implemented-why-cant-it-be-derived/92416/8
    // We can't use a BTreeSet because the underlying types don't impl Ord
    // We don't want to use a PetitSet here because of memory bloat
    // So a vec it is!
    pub(crate) Vec<Box<dyn Buttonlike>>,
);

impl ButtonlikeChord {
    /// Creates a [`ButtonlikeChord`] from multiple [`Buttonlike`]s, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    /// You can still use other methods to add different types of inputs into the chord.
    #[inline]
    pub fn new<U: Buttonlike>(inputs: impl IntoIterator<Item = U>) -> Self {
        Self::default().with_multiple(inputs)
    }

    /// Creates a [`ButtonlikeChord`] that only contains the given [`Buttonlike`].
    /// You can still use other methods to add different types of inputs into the chord.
    #[inline]
    pub fn from_single(input: impl Buttonlike) -> Self {
        Self::default().with(input)
    }

    /// Creates a [`ButtonlikeChord`] that combines the provided modifier and the given [`Buttonlike`].
    #[cfg(feature = "keyboard")]
    pub fn modified(modifier: super::keyboard::ModifierKey, input: impl Buttonlike) -> Self {
        Self::default().with(modifier).with(input)
    }

    /// Adds the given [`Buttonlike`] into this chord, avoiding duplicates.
    #[inline]
    pub fn with(mut self, input: impl Buttonlike) -> Self {
        self.push_boxed_unique(Box::new(input));
        self
    }

    /// Adds multiple [`Buttonlike`]s into this chord, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    #[inline]
    pub fn with_multiple<U: Buttonlike>(mut self, inputs: impl IntoIterator<Item = U>) -> Self {
        for input in inputs.into_iter() {
            self.push_boxed_unique(Box::new(input));
        }
        self
    }

    /// Adds the given boxed dyn [`Buttonlike`] to this chord, avoiding duplicates.
    #[inline]
    fn push_boxed_unique(&mut self, input: Box<dyn Buttonlike>) {
        if !self.0.contains(&input) {
            self.0.push(input);
        }
    }
}

impl UserInput for ButtonlikeChord {
    /// [`ButtonlikeChord`] acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Retrieves a list of simple, atomic [`Buttonlike`]s that compose the chord.
    ///
    /// The length of the basic inputs is the sum of the lengths of the inner inputs.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        let inputs = self
            .0
            .iter()
            .flat_map(|input| input.decompose().inputs())
            .collect();
        BasicInputs::Chord(inputs)
    }
}

#[serde_typetag]
impl Buttonlike for ButtonlikeChord {
    /// Checks if all the inner inputs within the chord are active simultaneously.
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, gamepad: Entity) -> bool {
        self.0
            .iter()
            .all(|input| input.pressed(input_store, gamepad))
    }

    fn press(&self, world: &mut World) {
        for input in &self.0 {
            input.press(world);
        }
    }

    fn release(&self, world: &mut World) {
        for input in &self.0 {
            input.release(world);
        }
    }

    fn press_as_gamepad(&self, world: &mut World, gamepad: Option<Entity>) {
        for input in &self.0 {
            input.press_as_gamepad(world, gamepad);
        }
    }

    fn release_as_gamepad(&self, world: &mut World, gamepad: Option<Entity>) {
        for input in &self.0 {
            input.release_as_gamepad(world, gamepad);
        }
    }
}

impl<U: Buttonlike> FromIterator<U> for ButtonlikeChord {
    /// Creates a [`ButtonlikeChord`] from an iterator over multiple [`Buttonlike`]s, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    /// You can still use other methods to add different types of inputs into the chord.
    #[inline]
    fn from_iter<T: IntoIterator<Item = U>>(iter: T) -> Self {
        Self::default().with_multiple(iter)
    }
}

/// A combined input that groups a [`Buttonlike`] and a [`Axislike`] together,
/// allowing you to only read the axis value when the button is pressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct AxislikeChord {
    /// The button that must be pressed to read the axis value.
    pub button: Box<dyn Buttonlike>,
    /// The axis value that is read when the button is pressed.
    pub axis: Box<dyn Axislike>,
}

impl AxislikeChord {
    /// Creates a new [`AxislikeChord`] from the given [`Buttonlike`] and [`Axislike`].
    #[inline]
    pub fn new(button: impl Buttonlike, axis: impl Axislike) -> Self {
        Self {
            button: Box::new(button),
            axis: Box::new(axis),
        }
    }
}

impl UserInput for AxislikeChord {
    /// [`AxislikeChord`] acts as a virtual axis.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    /// Retrieves a list of simple, atomic [`Buttonlike`]s that compose the chord.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::compose(self.button.decompose(), self.axis.decompose())
    }
}

#[serde_typetag]
impl Axislike for AxislikeChord {
    fn value(&self, input_store: &CentralInputStore, gamepad: Entity) -> f32 {
        if self.button.pressed(input_store, gamepad) {
            self.axis.value(input_store, gamepad)
        } else {
            0.0
        }
    }

    fn set_value(&self, world: &mut World, value: f32) {
        self.axis.set_value(world, value);
    }

    fn set_value_as_gamepad(&self, world: &mut World, value: f32, gamepad: Option<Entity>) {
        self.axis.set_value_as_gamepad(world, value, gamepad);
    }
}

/// A combined input that groups a [`Buttonlike`] and a [`DualAxislike`] together,
/// allowing you to only read the dual axis data when the button is pressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxislikeChord {
    /// The button that must be pressed to read the axis values.
    pub button: Box<dyn Buttonlike>,
    /// The dual axis data that is read when the button is pressed.
    pub dual_axis: Box<dyn DualAxislike>,
}

impl DualAxislikeChord {
    /// Creates a new [`DualAxislikeChord`] from the given [`Buttonlike`] and [`DualAxislike`].
    #[inline]
    pub fn new(button: impl Buttonlike, dual_axis: impl DualAxislike) -> Self {
        Self {
            button: Box::new(button),
            dual_axis: Box::new(dual_axis),
        }
    }
}

impl UserInput for DualAxislikeChord {
    /// [`DualAxislikeChord`] acts as a virtual dual-axis.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::DualAxis
    }

    /// Retrieves a list of simple, atomic [`Buttonlike`]s that compose the chord.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::compose(self.button.decompose(), self.dual_axis.decompose())
    }
}

#[serde_typetag]
impl DualAxislike for DualAxislikeChord {
    fn axis_pair(&self, input_store: &CentralInputStore, gamepad: Entity) -> Vec2 {
        if self.button.pressed(input_store, gamepad) {
            self.dual_axis.axis_pair(input_store, gamepad)
        } else {
            Vec2::ZERO
        }
    }

    fn set_axis_pair(&self, world: &mut World, axis_pair: Vec2) {
        self.dual_axis.set_axis_pair(world, axis_pair);
    }

    fn set_axis_pair_as_gamepad(
        &self,
        world: &mut World,
        axis_pair: Vec2,
        gamepad: Option<Entity>,
    ) {
        self.dual_axis
            .set_axis_pair_as_gamepad(world, axis_pair, gamepad);
    }
}

/// A combined input that groups a [`Buttonlike`] and a [`TripleAxislike`] together,
/// allowing you to only read the dual axis data when the button is pressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct TripleAxislikeChord {
    /// The button that must be pressed to read the axis values.
    pub button: Box<dyn Buttonlike>,
    /// The triple axis data that is read when the button is pressed.
    pub triple_axis: Box<dyn TripleAxislike>,
}

impl TripleAxislikeChord {
    /// Creates a new [`TripleAxislikeChord`] from the given [`Buttonlike`] and [`TripleAxislike`].
    #[inline]
    pub fn new(button: impl Buttonlike, triple_axis: impl TripleAxislike) -> Self {
        Self {
            button: Box::new(button),
            triple_axis: Box::new(triple_axis),
        }
    }
}

impl UserInput for TripleAxislikeChord {
    /// [`TripleAxislikeChord`] acts as a virtual triple-axis.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::TripleAxis
    }

    /// Retrieves a list of simple, atomic [`Buttonlike`]s that compose the chord.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::compose(self.button.decompose(), self.triple_axis.decompose())
    }
}

#[serde_typetag]
impl TripleAxislike for TripleAxislikeChord {
    fn axis_triple(&self, input_store: &CentralInputStore, gamepad: Entity) -> Vec3 {
        if self.button.pressed(input_store, gamepad) {
            self.triple_axis.axis_triple(input_store, gamepad)
        } else {
            Vec3::ZERO
        }
    }

    fn set_axis_triple(&self, world: &mut World, axis_triple: Vec3) {
        self.triple_axis.set_axis_triple(world, axis_triple);
    }

    fn set_axis_triple_as_gamepad(
        &self,
        world: &mut World,
        axis_triple: Vec3,
        gamepad: Option<Entity>,
    ) {
        self.triple_axis
            .set_axis_triple_as_gamepad(world, axis_triple, gamepad);
    }
}

#[cfg(feature = "keyboard")]
#[cfg(test)]
mod tests {
    use super::ButtonlikeChord;
    use crate::plugin::CentralInputStorePlugin;
    use crate::user_input::updating::CentralInputStore;
    use crate::user_input::Buttonlike;
    use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent, GamepadEvent};
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(InputPlugin)
            .add_plugins(CentralInputStorePlugin);

        // WARNING: you MUST register your gamepad during tests,
        // or all gamepad input mocking actions will fail
        let gamepad = app.world_mut().spawn(()).id();
        let mut gamepad_events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
            // This MUST be consistent with any other mocked events
            gamepad,
            connection: GamepadConnection::Connected {
                name: "TestController".into(),
                vendor_id: None,
                product_id: None,
            },
        }));

        // Ensure that the gamepad is picked up by the appropriate system
        app.update();
        // Ensure that the connection event is flushed through
        app.update();
        app
    }

    #[test]
    fn test_chord_with_buttons_only() {
        let chord = ButtonlikeChord::new([KeyCode::KeyC, KeyCode::KeyH])
            .with(KeyCode::KeyO)
            .with_multiple([KeyCode::KeyR, KeyCode::KeyD]);

        let required_keys = [
            KeyCode::KeyC,
            KeyCode::KeyH,
            KeyCode::KeyO,
            KeyCode::KeyR,
            KeyCode::KeyD,
        ];

        let expected_inners = required_keys
            .iter()
            .map(|key| Box::new(*key) as Box<dyn Buttonlike>)
            .collect::<Vec<_>>();
        assert_eq!(chord.0, expected_inners);

        // No keys pressed, resulting in a released chord with a value of zero.
        let mut app = test_app();
        app.update();
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();
        assert!(!chord.pressed(inputs, gamepad));

        // All required keys pressed, resulting in a pressed chord with a value of one.
        let mut app = test_app();
        for key in required_keys {
            key.press(app.world_mut());
        }
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();
        assert!(chord.pressed(inputs, gamepad));

        // Some required keys pressed, but not all required keys for the chord,
        // resulting in a released chord with a value of zero.
        for i in 1..=4 {
            let mut app = test_app();
            for key in required_keys.iter().take(i) {
                key.press(app.world_mut());
            }
            app.update();
            let inputs = app.world().resource::<CentralInputStore>();
            assert!(!chord.pressed(inputs, gamepad));
        }

        // Five keys pressed, but not all required keys for the chord,
        // resulting in a released chord with a value of zero.
        let mut app = test_app();
        for key in required_keys.iter().take(4) {
            key.press(app.world_mut());
        }
        KeyCode::KeyB.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();
        assert!(!chord.pressed(inputs, gamepad));
    }
}
