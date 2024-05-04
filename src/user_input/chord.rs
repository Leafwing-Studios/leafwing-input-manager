//! This module contains [`InputChord`] and its impls.

use bevy::prelude::Reflect;
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::clashing_inputs::BasicInputs;
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::user_input::{DualAxisData, InputKind, UserInput};

/// A combined input that groups multiple [`UserInput`]s together,
/// which is useful for creating input combinations like hotkeys, shortcuts, and macros.
///
/// # Behaviors
///
/// - Simultaneous Activation Check: You can check if all the included inputs
///   are actively pressed at the same time.
/// - Single-Axis Input Combination: If some inner inputs are single-axis (like mouse wheel),
///   the input chord can combine (sum) their values into a single value.
/// - First Dual-Axis Input Only: Retrieves the values only from the first included
///   dual-axis input (like gamepad triggers). The state of other dual-axis inputs is ignored.
///
/// # Warning
///
/// Adding the same input multiple times into an input chord has no effect,
/// preventing redundant data fetching from multiple instances of the same input.
///
/// When using an input chord within another input that can hold multiple [`UserInput`]s,
/// the chord itself will always be treated as a button.
/// Any additional functionalities it offered (like single-axis values) will be ignored in this context.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub struct InputChord(
    // Note: We can't use a HashSet here because of
    // https://users.rust-lang.org/t/hash-not-implemented-why-cant-it-be-derived/92416/8
    // We can't use a BTreeSet because the underlying types don't impl Ord
    // We don't want to use a PetitSet here because of memory bloat
    // So a vec it is!
    pub(crate) Vec<Box<dyn UserInput>>,
);

impl InputChord {
    /// Creates a [`InputChord`] that only contains the given [`UserInput`].
    /// You can still use other methods to add different types of inputs into the chord.
    #[inline]
    pub fn from_single(input: impl UserInput) -> Self {
        Self::default().with(input)
    }

    /// Creates a [`InputChord`] from multiple [`UserInput`]s, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    /// You can still use other methods to add different types of inputs into the chord.
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    pub fn from_multiple<U: UserInput>(inputs: impl IntoIterator<Item = U>) -> Self {
        Self::default().with_multiple(inputs)
    }

    /// Adds the given [`UserInput`] into this chord, avoiding duplicates.
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    pub fn with(mut self, input: impl UserInput) -> Self {
        self.push_boxed(Box::new(input));
        self
    }

    /// Adds multiple [`UserInput`]s into this chord, avoiding duplicates.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    pub fn with_multiple<U: UserInput>(mut self, inputs: impl IntoIterator<Item = U>) -> Self {
        for input in inputs.into_iter() {
            self.push_boxed(Box::new(input));
        }
        self
    }

    /// Adds the given boxed dyn [`UserInput`] to this chord, avoiding duplicates.
    ///
    /// This ensures that the same input isn't added multiple times,
    /// preventing redundant data fetching from multiple instances of the same input.
    #[inline]
    fn push_boxed(&mut self, input: Box<dyn UserInput>) {
        if !self.0.contains(&input) {
            self.0.push(input);
        }
    }
}

#[serde_typetag]
impl UserInput for InputChord {
    /// [`InputChord`] always acts as a virtual button.
    #[inline]
    fn kind(&self) -> InputKind {
        InputKind::Button
    }

    /// Checks if all the inner inputs within the chord are active simultaneously.
    #[must_use]
    #[inline]
    fn pressed(&self, input_streams: &InputStreams) -> bool {
        self.0.iter().all(|input| input.pressed(input_streams))
    }

    /// Returns a single value representing the combined state of inner [`UserInput`]s within the chord.
    ///
    /// # Behaviors
    ///
    /// This function behaves differently depending on the kind of inputs contained.
    ///
    /// When the chord contains **one or more single-axis inputs** (e.g., mouse wheel),
    /// this method returns the **sum** of their individual values,
    /// allowing you to combine the effects of multiple controls into a single value.
    ///
    /// When the chord contains **only non-single-axis inputs** (e.g., buttons),
    /// this method returns `0.0` when any of the input is inactive
    /// or `1.0` when all inputs are active simultaneously.
    /// This behavior is consistent with how buttons function as digital inputs (either on or off).
    #[must_use]
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        let mut has_axis = false;
        let mut axis_value = 0.0;
        for input in self.0.iter() {
            if input.kind() == InputKind::Axis {
                has_axis = true;
                axis_value += input.value(input_streams);
            }
        }

        if has_axis {
            axis_value
        } else {
            f32::from(self.pressed(input_streams))
        }
    }

    /// Attempts to retrieve the X and Y values from the **first** inner dual-axis input within the chord.
    #[must_use]
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
        self.0
            .iter()
            .filter(|input| input.kind() == InputKind::DualAxis)
            .flat_map(|input| input.axis_pair(input_streams))
            .next()
    }

    /// Retrieves a list of simple, atomic [`UserInput`]s that compose the chord.
    #[must_use]
    #[inline]
    fn basic_inputs(&self) -> BasicInputs {
        let inputs = self
            .0
            .iter()
            .flat_map(|input| input.basic_inputs().inputs())
            .collect();
        BasicInputs::Group(inputs)
    }

    /// Returns the [`RawInputs`] that combines the raw input events of all inner inputs.
    #[inline]
    fn raw_inputs(&self) -> RawInputs {
        self.0.iter().fold(RawInputs::default(), |inputs, next| {
            inputs.merge_input(&next.raw_inputs())
        })
    }
}

impl<U: UserInput> FromIterator<U> for InputChord {
    /// Creates a [`InputChord`] from an iterator over multiple [`UserInput`]s, avoiding duplicates.
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

#[cfg(test)]
mod tests {
    use bevy::input::gamepad::{
        GamepadConnection, GamepadConnectionEvent, GamepadEvent, GamepadInfo,
    };
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use super::*;
    use crate::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(InputPlugin);

        // WARNING: you MUST register your gamepad during tests,
        // or all gamepad input mocking actions will fail
        let mut gamepad_events = app.world.resource_mut::<Events<GamepadEvent>>();
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
    fn test_chord_with_buttons_only() {
        let chord = InputChord::from_multiple([KeyCode::KeyC, KeyCode::KeyH])
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
            .map(|key| Box::new(*key) as Box<dyn UserInput>)
            .collect::<Vec<_>>();
        assert_eq!(chord.0, expected_inners);

        let expected_raw_inputs = RawInputs::from_keycodes(required_keys);
        assert_eq!(chord.raw_inputs(), expected_raw_inputs);

        // No keys pressed, resulting in a released chord with a value of zero.
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&chord, &inputs);

        // All required keys pressed, resulting in a pressed chord with a value of one.
        let mut app = test_app();
        for key in required_keys {
            app.press_input(key);
        }
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        pressed(&chord, &inputs);

        // Some required keys pressed, but not all required keys for the chord,
        // resulting in a released chord with a value of zero.
        for i in 1..=4 {
            let mut app = test_app();
            for key in required_keys.iter().take(i) {
                app.press_input(*key);
            }
            app.update();
            let inputs = InputStreams::from_world(&app.world, None);
            released(&chord, &inputs);
        }

        // Five keys pressed, but not all required keys for the chord,
        // resulting in a released chord with a value of zero.
        let mut app = test_app();
        for key in required_keys.iter().take(4) {
            app.press_input(*key);
        }
        app.press_input(KeyCode::KeyB);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        released(&chord, &inputs);
    }

    #[test]
    fn test_chord_with_buttons_and_axes() {
        let chord = InputChord::from_multiple([KeyCode::KeyA, KeyCode::KeyB])
            .with(MouseScrollAxis::X)
            .with(MouseScrollAxis::Y)
            .with(GamepadStick::LEFT)
            .with(GamepadStick::RIGHT);

        let required_keys = [KeyCode::KeyA, KeyCode::KeyB];

        let expected_inners = required_keys
            .iter()
            .map(|key| Box::new(*key) as Box<dyn UserInput>)
            .chain(Some(Box::new(MouseScrollAxis::X) as Box<dyn UserInput>))
            .chain(Some(Box::new(MouseScrollAxis::Y) as Box<dyn UserInput>))
            .chain(Some(Box::new(GamepadStick::LEFT) as Box<dyn UserInput>))
            .chain(Some(Box::new(GamepadStick::RIGHT) as Box<dyn UserInput>))
            .collect::<Vec<_>>();
        assert_eq!(chord.0, expected_inners);

        let expected_raw_inputs = RawInputs::from_keycodes(required_keys)
            .merge_input(&MouseScrollAxis::X.raw_inputs())
            .merge_input(&MouseScrollAxis::Y.raw_inputs())
            .merge_input(&GamepadStick::LEFT.raw_inputs())
            .merge_input(&GamepadStick::RIGHT.raw_inputs());
        assert_eq!(chord.raw_inputs(), expected_raw_inputs);

        // No input events, resulting in a released chord with values of zeros.
        let zeros = Some(DualAxisData::default());
        let mut app = test_app();
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, false, 0.0, zeros);

        // Only one or all required keys pressed without axial inputs,
        // resulting in a released chord with values of zeros.
        for i in 1..=2 {
            let mut app = test_app();
            for key in required_keys.iter().take(i) {
                app.press_input(*key);
            }
            app.update();
            let inputs = InputStreams::from_world(&app.world, None);
            check(&chord, &inputs, false, 0.0, zeros);
        }

        // Send changes in values of some required single-axis inputs,
        // resulting in a released chord with a combined value from the given single-axis values.
        let value = 2.0;
        let mut app = test_app();
        app.send_axis_values(MouseScrollAxis::X, [value]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, false, value, zeros);

        let data = DualAxisData::new(2.0, 3.0);
        let mut app = test_app();
        app.send_axis_values(MouseScroll::RAW, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, false, data.x() + data.y(), zeros);

        // Send changes in values of first dual-axis input,
        // resulting in a released chord with the given value.
        let data = DualAxisData::new(0.5, 0.6);
        let mut app = test_app();
        app.send_axis_values(GamepadStick::LEFT, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, false, 0.0, Some(data));

        // Send changes in values of second dual-axis input,
        // resulting in a released chord with values of zeros.
        let data = DualAxisData::new(0.5, 0.6);
        let mut app = test_app();
        app.send_axis_values(GamepadStick::RIGHT, [data.x(), data.y()]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, false, 0.0, zeros);

        // Send changes in values of first dual-axis input and all single-axis inputs,
        // resulting in a released chord with a combined value from the given single-axis values
        // and an axis pair of the first dual-axis input.
        let data = DualAxisData::new(0.5, 0.6);
        let single = Vec2::new(0.8, -0.2);
        let mut app = test_app();
        app.send_axis_values(GamepadStick::LEFT, [data.x(), data.y()]);
        app.send_axis_values(MouseScrollAxis::X, [single.x]);
        app.send_axis_values(MouseScrollAxis::Y, [single.y]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, false, single.x + single.y, Some(data));

        // The chord are pressed only if all inputs are activated.
        let first_data = DualAxisData::new(0.5, 0.6);
        let second_data = DualAxisData::new(0.4, 0.8);
        let single = Vec2::new(0.8, -0.2);
        let mut app = test_app();
        for key in required_keys {
            app.press_input(key);
        }
        app.send_axis_values(GamepadStick::LEFT, [first_data.x(), first_data.y()]);
        app.send_axis_values(GamepadStick::RIGHT, [second_data.x(), second_data.y()]);
        app.send_axis_values(MouseScrollAxis::X, [single.x]);
        app.send_axis_values(MouseScrollAxis::Y, [single.y]);
        app.update();
        let inputs = InputStreams::from_world(&app.world, None);
        check(&chord, &inputs, true, single.x + single.y, Some(first_data));
    }
}
