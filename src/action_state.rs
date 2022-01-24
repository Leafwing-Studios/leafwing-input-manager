//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::{Actionlike, InputMap};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::utils::{Duration, Instant};

/// The current state of a particular virtual button,
/// corresponding to a single [`Actionlike`] action.
///
/// Detailed timing information for the button can be accessed through the stored [`Timing`] value
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum VirtualButtonState {
    /// This button is currently pressed
    Pressed(Timing),
    /// This button is currently released
    Released(Timing),
}

/// Stores the timing information for a [`VirtualButtonState`]
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Timing {
    /// The [`Instant`] at which the button was pressed or released
    ///
    /// Recorded as the [`Time`](bevy::core::Time) at the start of the tick after the state last changed.
    /// If this is none, [`ActionState::update`] has not been called yet.
    pub instant_started: Option<Instant>,
    /// The [`Duration`] for which the button has been pressed or released.
    ///
    /// This begins at [`Duration::ZERO`] when [`ActionState::update`] is called.
    pub current_duration: Duration,
    /// The [`Duration`] for which the button was pressed or released before the state last changed.
    pub previous_duration: Duration,
}

impl VirtualButtonState {
    /// Is the button currently pressed?
    #[inline]
    pub fn pressed(&self) -> bool {
        match self {
            VirtualButtonState::Pressed(_) => true,
            VirtualButtonState::Released(_) => false,
        }
    }

    /// Is the button currently released?
    #[inline]
    pub fn released(&self) -> bool {
        match self {
            VirtualButtonState::Pressed(_) => false,
            VirtualButtonState::Released(_) => true,
        }
    }

    /// Was the button pressed since the last time [`ActionState::update`] was called?
    #[inline]
    pub fn just_pressed(&self) -> bool {
        match self {
            VirtualButtonState::Pressed(timing) => timing.instant_started.is_none(),
            VirtualButtonState::Released(_timing) => false,
        }
    }

    /// Was the button released since the last time [`ActionState::update`] was called?
    #[inline]
    pub fn just_released(&self) -> bool {
        match self {
            VirtualButtonState::Pressed(_timing) => false,
            VirtualButtonState::Released(timing) => timing.instant_started.is_none(),
        }
    }

    /// The [`Instant`] at which the button was pressed or released
    ///
    /// Recorded as the [`Time`](bevy::core::Time) at the start of the tick after the state last changed.
    /// If this is none, [`ActionState::update`] has not been called yet.
    #[inline]
    pub fn instant_started(&self) -> Option<Instant> {
        match self {
            VirtualButtonState::Pressed(timing) => timing.instant_started,
            VirtualButtonState::Released(timing) => timing.instant_started,
        }
    }

    /// The [`Duration`] for which the button has been pressed or released.
    ///
    /// This begins at [`Duration::ZERO`] when [`ActionState::update`] is called.
    #[inline]
    pub fn current_duration(&self) -> Duration {
        match self {
            VirtualButtonState::Pressed(timing) => timing.current_duration,
            VirtualButtonState::Released(timing) => timing.current_duration,
        }
    }
    /// The [`Duration`] for which the button was pressed or released before the state last changed.
    #[inline]
    pub fn previous_duration(&self) -> Duration {
        match self {
            VirtualButtonState::Pressed(timing) => timing.previous_duration,
            VirtualButtonState::Released(timing) => timing.previous_duration,
        }
    }
}

impl Default for VirtualButtonState {
    fn default() -> Self {
        VirtualButtonState::Released(Timing::default())
    }
}

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Intended to be used as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Example
/// ```rust
/// use leafwing_input_manager::prelude::*;
/// use bevy::utils::Instant;
/// use strum::EnumIter;
///
/// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
/// enum Action {
///     Left,
///     Right,
///     Jump,
/// }
///
/// let mut action_state = ActionState::<Action>::default();
///
/// // Typically, this is done automatically by the `InputManagerPlugin` from user inputs
/// // using the `ActionState::update` method
/// action_state.press(Action::Jump);
///
/// assert!(action_state.pressed(Action::Jump));
/// assert!(action_state.just_pressed(Action::Jump));
/// assert!(action_state.released(Action::Left));
///
/// // Resets just_pressed and just_released
/// action_state.tick(Instant::now());
/// assert!(action_state.pressed(Action::Jump));
/// assert!(!action_state.just_pressed(Action::Jump));
///
/// action_state.release(Action::Jump);
/// assert!(!action_state.pressed(Action::Jump));
/// assert!(action_state.released(Action::Jump));
/// assert!(action_state.just_released(Action::Jump));
///
/// action_state.tick(Instant::now());
/// assert!(action_state.released(Action::Jump));
/// assert!(!action_state.just_released(Action::Jump));
/// ```
#[derive(Component)]
pub struct ActionState<A: Actionlike> {
    map: HashMap<A, VirtualButtonState>,
}

impl<A: Actionlike> ActionState<A> {
    /// Updates the [`ActionState`] based on the [`InputMap`] and the provided [`Input`]s
    ///
    /// Presses and releases buttons according to the current state of the inputs.
    /// Combine with [`ActionState::tick`] to update `just_pressed` and `just_released`.
    pub fn update(
        &mut self,
        input_map: &InputMap<A>,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) {
        for action in A::iter() {
            if input_map.pressed(
                action,
                gamepad_input_stream,
                keyboard_input_stream,
                mouse_input_stream,
            ) {
                self.press(action);
            } else {
                self.release(action);
            }
        }
    }

    /// Advances the time for all virtual buttons
    ///
    /// The underlying [`VirtualButtonState`] state will be advanced according to the `current_time`.
    /// - if no [`Instant`] is set, the `current_time` will be set as the initial time at which the button was pressed / released
    /// - the [`Duration`] will advance to reflect elapsed time
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::action_state::VirtualButtonState;
    /// use strum::EnumIter;
    /// use bevy::utils::Instant;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    /// // Virtual buttons start released
    /// assert!(action_state.state(Action::Run).just_released());
    /// assert!(action_state.just_released(Action::Jump));
    ///
    /// // Ticking time moves causes buttons that were just released to no longer be just released
    /// action_state.tick(Instant::now());
    /// assert!(action_state.released(Action::Jump));
    /// assert!(!action_state.just_released(Action::Jump));
    ///
    /// action_state.press(Action::Jump);
    /// assert!(action_state.just_pressed(Action::Jump));
    ///
    /// // Ticking time moves causes buttons that were just pressed to no longer be just pressed
    /// action_state.tick(Instant::now());
    /// assert!(action_state.pressed(Action::Jump));
    /// assert!(!action_state.just_pressed(Action::Jump));
    /// ```
    pub fn tick(&mut self, current_instant: Instant) {
        use VirtualButtonState::*;

        for state in self.map.values_mut() {
            *state = match state {
                Pressed(timing) => match timing.instant_started {
                    Some(instant) => Pressed(Timing {
                        current_duration: current_instant - instant,
                        ..*timing
                    }),
                    None => Pressed(Timing {
                        instant_started: Some(current_instant),
                        current_duration: Duration::ZERO,
                        ..*timing
                    }),
                },
                Released(timing) => match timing.instant_started {
                    Some(instant) => Released(Timing {
                        current_duration: current_instant - instant,
                        ..*timing
                    }),
                    None => Released(Timing {
                        instant_started: Some(current_instant),
                        current_duration: Duration::ZERO,
                        ..*timing
                    }),
                },
            };
        }
    }

    /// Gets the [`VirtualButtonState`] of the corresponding `action`
    #[inline]
    pub fn state(&self, action: A) -> VirtualButtonState {
        if let Some(state) = self.map.get(&action) {
            state.clone()
        } else {
            VirtualButtonState::default()
        }
    }

    /// Press the `action` virtual button
    pub fn press(&mut self, action: A) {
        if let VirtualButtonState::Released(timing) = self.state(action) {
            self.map.insert(
                action,
                VirtualButtonState::Pressed(Timing {
                    instant_started: None,
                    current_duration: Duration::ZERO,
                    previous_duration: timing.current_duration,
                }),
            );
        }
    }

    /// Release the `action` virtual button
    pub fn release(&mut self, action: A) {
        if let VirtualButtonState::Pressed(timing) = self.state(action) {
            self.map.insert(
                action,
                VirtualButtonState::Released(Timing {
                    instant_started: None,
                    current_duration: Duration::ZERO,
                    previous_duration: timing.current_duration,
                }),
            );
        }
    }

    /// Releases all action virtual buttons
    pub fn release_all(&mut self) {
        for action in A::iter() {
            self.release(action);
        }
    }

    /// Is this `action` currently pressed?
    #[must_use]
    pub fn pressed(&self, action: A) -> bool {
        self.state(action).pressed()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[must_use]
    pub fn just_pressed(&self, action: A) -> bool {
        self.state(action).just_pressed()
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[must_use]
    pub fn released(&self, action: A) -> bool {
        self.state(action).released()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[must_use]
    pub fn just_released(&self, action: A) -> bool {
        self.state(action).just_released()
    }

    /// Creates a Hashmap with all of the possible A variants as keys, and false as the values
    #[must_use]
    pub fn default_map<V: Default>() -> HashMap<A, V> {
        // PERF: optimize construction through pre-allocation or constification
        let mut map: HashMap<A, V> = HashMap::default();

        for action in A::iter() {
            map.insert(action, V::default());
        }
        map
    }
}

impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            map: Self::default_map(),
        }
    }
}

/// A component that allows the attached entity to drive the [`ActionState`] of the associated entity
///
/// Used in [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction).
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionStateDriver<A: Actionlike> {
    /// The action triggered by this entity
    pub action: A,
    /// The entity whose action state should be updated
    pub entity: Entity,
}

/// Thresholds for when the `value` of a button will cause it to be pressed or released
///
/// Both `pressed` and `released` must be between 0.0 and 1.0 inclusive,
/// and `pressed` must be greater than `released`
/// Defaults to 0.5 for both values
#[derive(Debug, Clone)]
pub struct ButtonThresholds {
    pressed: f32,
    released: f32,
}

impl Default for ButtonThresholds {
    fn default() -> Self {
        Self {
            pressed: 0.5,
            released: 0.5,
        }
    }
}

impl ButtonThresholds {
    /// Gets the value at or above which the button is considered to be pressed
    pub fn pressed(&self) -> f32 {
        self.pressed
    }

    /// Gets the value below which the button is considered to be released
    pub fn released(&self) -> f32 {
        self.released
    }

    /// Sets the value of the pressed threshold.
    ///
    /// If the provided `value` is less than the `released` threshold,
    /// it is increased to the `released` threshold and a
    /// `ThresholdError(value_set_to)` error is returned.
    ///
    /// # Panics
    /// Panics if the value provided is not between 0.0 and 1.0 inclusive.
    pub fn set_pressed(&mut self, value: f32) -> Result<(), ThresholdError> {
        assert!(value >= 0.0);
        assert!(value <= 1.0);

        if value >= self.released {
            self.pressed = value;
            Ok(())
        } else {
            self.pressed = self.released;
            Err(ThresholdError(self.released))
        }
    }

    /// Gets the value below which the button is considered to be released
    ///
    /// If the provided `value` is greater than the `pressed` threshold,
    /// it is increased to the `pressed` threshold and a
    /// `ThresholdError(value_set_to)` error is returned.
    ///
    /// # Panics
    /// Panics if the value provided is not between 0.0 and 1.0 inclusive.
    pub fn set_released(&mut self, value: f32) -> Result<(), ThresholdError> {
        assert!(value >= 0.0);
        assert!(value <= 1.0);

        if value <= self.pressed {
            self.pressed = value;
            Ok(())
        } else {
            self.released = self.pressed;
            Err(ThresholdError(self.pressed))
        }
    }
}

/// An error that resulted from inserting an invalid (but within range value) to [`ButtonThresholds`]
#[derive(Debug, Clone)]
pub struct ThresholdError(f32);

mod tests {
    use crate::prelude::*;
    use strum::EnumIter;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn press_lifecycle() {
        use bevy::prelude::*;
        use bevy::utils::Instant;

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::R);

        // Input streams
        let gamepad_input_stream = Input::<GamepadButton>::default();
        let mut keyboard_input_stream = Input::<KeyCode>::default();
        let mouse_input_stream = Input::<MouseButton>::default();

        // Starting state
        action_state.update(
            &input_map,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        );

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(action_state.just_released(Action::Run));

        // Pressing
        keyboard_input_stream.press(KeyCode::R);
        action_state.update(
            &input_map,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        );

        assert!(action_state.pressed(Action::Run));
        assert!(action_state.just_pressed(Action::Run));
        assert!(!action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));

        // Waiting
        action_state.tick(Instant::now());
        action_state.update(
            &input_map,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        );

        assert!(action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(!action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));

        // Releasing
        keyboard_input_stream.release(KeyCode::R);
        action_state.update(
            &input_map,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        );
        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(action_state.just_released(Action::Run));

        // Waiting
        action_state.tick(Instant::now());
        action_state.update(
            &input_map,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        );

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));
    }

    #[test]
    fn durations() {
        use bevy::utils::{Duration, Instant};
        use std::thread::sleep;

        let mut action_state = ActionState::<Action>::default();

        // Virtual buttons start released
        assert!(action_state.state(Action::Jump).released());
        assert_eq!(action_state.state(Action::Jump).instant_started(), None,);
        assert_eq!(
            action_state.state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Pressing a button swaps the state
        action_state.press(Action::Jump);
        assert!(action_state.state(Action::Jump).pressed());
        assert_eq!(action_state.state(Action::Jump).instant_started(), None);
        assert_eq!(
            action_state.state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Ticking time sets the instant for the new state
        let t0 = Instant::now();
        action_state.tick(t0);
        assert_eq!(action_state.state(Action::Jump).instant_started(), Some(t0));
        assert_eq!(
            action_state.state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Time passes
        sleep(Duration::from_micros(1));
        let t1 = Instant::now();

        // The duration is updated
        action_state.tick(t1);
        assert_eq!(action_state.state(Action::Jump).instant_started(), Some(t0));
        assert_eq!(action_state.state(Action::Jump).current_duration(), t1 - t0);
        assert_eq!(
            action_state.state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Releasing again, swapping the current duration to the previous one
        action_state.release(Action::Jump);
        assert_eq!(action_state.state(Action::Jump).instant_started(), None);
        assert_eq!(
            action_state.state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.state(Action::Jump).previous_duration(),
            t1 - t0,
        );
    }
}
