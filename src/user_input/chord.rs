//! This module contains [`ButtonlikeChord`] and its impls.

use bevy::math::Vec2;
use bevy::prelude::Reflect;
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::clashing_inputs::BasicInputs;
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::user_input::{Buttonlike, UserInput};
use crate::InputControlKind;

use super::keyboard::ModifierKey;
use super::{Axislike, DualAxislike};

/// A combined input that groups multiple [`Buttonlike`]s together,
/// allowing you to define complex input combinations like hotkeys, shortcuts, and macros.
///
/// # Behaviors
///
/// - Activation: All included inputs must be active simultaneously.
/// - Deduplication: Adding duplicate inputs within a chord will ignore the extras,
///     preventing redundant data fetching.
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy::input::InputPlugin;
/// use leafwing_input_manager::plugin::AccumulatorPlugin;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
/// app.add_plugins((InputPlugin, AccumulatorPlugin));
///
/// // Define a chord using A and B keys
/// let input = ButtonlikeChord::new([KeyCode::KeyA, KeyCode::KeyB]);
///
/// // Pressing only one key doesn't activate the input
/// app.press_input(KeyCode::KeyA);
/// app.update();
/// assert!(!app.pressed(input.clone()));
///
/// // Pressing both keys activates the input
/// app.press_input(KeyCode::KeyB);
/// app.update();
/// assert!(app.pressed(input.clone()));
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
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
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
    pub fn modified(modifier: ModifierKey, input: impl Buttonlike) -> Self {
        Self::default().with(modifier).with(input)
    }

    /// Adds the given [`Buttonlike`] into this chord, avoiding duplicates.
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    pub fn with(mut self, input: impl Buttonlike) -> Self {
        self.push_boxed_unique(Box::new(input));
        self
    }

    /// Adds multiple [`Buttonlike`]s into this chord, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    pub fn with_multiple<U: Buttonlike>(mut self, inputs: impl IntoIterator<Item = U>) -> Self {
        for input in inputs.into_iter() {
            self.push_boxed_unique(Box::new(input));
        }
        self
    }

    /// Adds the given boxed dyn [`Buttonlike`] to this chord, avoiding duplicates.
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    fn push_boxed_unique(&mut self, input: Box<dyn Buttonlike>) {
        if !self.0.contains(&input) {
            self.0.push(input);
        }
    }
}

#[serde_typetag]
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

    /// Returns the [`RawInputs`] that combines the raw input events of all inner inputs.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        self.0.iter().fold(RawInputs::default(), |inputs, next| {
            inputs.merge_input(&next.raw_inputs())
        })
    }
}

impl Buttonlike for ButtonlikeChord {
    /// Checks if all the inner inputs within the chord are active simultaneously.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.0.iter().all(|input| input.pressed(input_streams))
    }
}

impl<U: Buttonlike> FromIterator<U> for ButtonlikeChord {
    /// Creates a [`ButtonlikeChord`] from an iterator over multiple [`Buttonlike`]s, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    /// You can still use other methods to add different types of inputs into the chord.
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
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

#[serde_typetag]
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

    /// Returns the [`RawInputs`] that combines the raw input events of all inner inputs.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        let raw_button_input = self.button.raw_inputs();
        let raw_axis_input = self.axis.raw_inputs();

        raw_button_input.merge_input(&raw_axis_input)
    }
}

impl Axislike for AxislikeChord {
    fn value(&self, input_streams: &InputStreams) -> f32 {
        if self.button.pressed(input_streams) {
            self.axis.value(input_streams)
        } else {
            0.0
        }
    }
}

/// A combined input that groups a [`Buttonlike`] and a [`DualAxislike`] together,
/// allowing you to only read the dual axis data when the button is pressed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct DualAxislikeChord {
    /// The button that must be pressed to read the axis value.
    pub button: Box<dyn Buttonlike>,
    /// The dual axis data that is read when the button is pressed.
    pub dual_axis: Box<dyn DualAxislike>,
}

impl DualAxislikeChord {
    /// Creates a new [`AxislikeChord`] from the given [`Buttonlike`] and [`Axislike`].
    #[inline]
    pub fn new(button: impl Buttonlike, dual_axis: impl DualAxislike) -> Self {
        Self {
            button: Box::new(button),
            dual_axis: Box::new(dual_axis),
        }
    }
}

#[serde_typetag]
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

    /// Returns the [`RawInputs`] that combines the raw input events of all inner inputs.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        let raw_button_input = self.button.raw_inputs();
        let raw_axis_input = self.dual_axis.raw_inputs();

        raw_button_input.merge_input(&raw_axis_input)
    }
}

impl DualAxislike for DualAxislikeChord {
    fn axis_pair(&self, input_streams: &InputStreams) -> Vec2 {
        if self.button.pressed(input_streams) {
            self.dual_axis.axis_pair(input_streams)
        } else {
            Vec2::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
    };
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use super::*;
    use crate::plugin::AccumulatorPlugin;
    use crate::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(InputPlugin)
            .add_plugins(AccumulatorPlugin);

        // WARNING: you MUST register your gamepad during tests,
        // or all gamepad input mocking actions will fail
        let mut gamepad_events = app.world_mut().resource_mut::<Events<GamepadEvent>>();
        gamepad_events.send(GamepadEvent::Connection(GamepadConnectionEvent {
            // This MUST be consistent with any other mocked events
            gamepad: Gamepad { id: 1 },
            connection: GamepadConnection::Connected(GamepadInfo {
                name: "TestController".into(),
            }),
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

        let expected_raw_inputs = RawInputs::from_keycodes(required_keys);
        assert_eq!(chord.raw_inputs(), expected_raw_inputs);

        // No keys pressed, resulting in a released chord with a value of zero.
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);
        assert!(!chord.pressed(&inputs));

        // All required keys pressed, resulting in a pressed chord with a value of one.
        let mut app = test_app();
        for key in required_keys {
            app.press_input(key);
        }
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);
        assert!(chord.pressed(&inputs));

        // Some required keys pressed, but not all required keys for the chord,
        // resulting in a released chord with a value of zero.
        for i in 1..=4 {
            let mut app = test_app();
            for key in required_keys.iter().take(i) {
                app.press_input(*key);
            }
            app.update();
            let inputs = InputStreams::from_world(app.world(), None);
            assert!(!chord.pressed(&inputs));
        }

        // Five keys pressed, but not all required keys for the chord,
        // resulting in a released chord with a value of zero.
        let mut app = test_app();
        for key in required_keys.iter().take(4) {
            app.press_input(*key);
        }
        app.press_input(KeyCode::KeyB);
        app.update();
        let inputs = InputStreams::from_world(app.world(), None);
        assert!(!chord.pressed(&inputs));
    }
}
