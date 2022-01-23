//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::{Actionlike, InputMap};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::utils::{Duration, Instant};

/// The current state of a particular virtual button,
/// corresponding to a single [`Actionlike`] action.
///
/// If the [`Duration`] of the [`VirtualButtonState::Pressed`] or [`VirtualButtonState::Released`] state is
/// [`Duration::ZERO`], the button is considered to be "just pressed" / "just released".
/// If the [`Option<Instant>`](std::time::Instant) stored is `None`, the virtual button was pressed / released
/// since the last time [`ActionState::tick`] was called.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum VirtualButtonState {
    /// This button is currently pressed
    ///
    /// The [`Instant`] records the app's [`Time`](bevy::core::Time) at the start of the tick it was first pressed.
    /// The [`Duration`] stores how long the button has been pressed for.
    Pressed(Option<Instant>, Duration),
    /// This button is currently released
    ///
    /// The [`Instant`] records the app's [`Time`](bevy::core::Time) at the start of the tick after it was first released.
    /// The [`Duration`] stores how long the button has been released for.
    Released(Option<Instant>, Duration),
}

impl VirtualButtonState {
    /// The state that buttons are put into when pressed
    pub const JUST_PRESSED: VirtualButtonState = VirtualButtonState::Pressed(None, Duration::ZERO);
    /// The state that buttons are put into when released
    pub const JUST_RELEASED: VirtualButtonState =
        VirtualButtonState::Released(None, Duration::ZERO);
}

impl Default for VirtualButtonState {
    fn default() -> Self {
        VirtualButtonState::JUST_RELEASED
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
    /// assert_eq!(action_state.state(Action::Run), VirtualButtonState::JUST_RELEASED);
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
    pub fn tick(&mut self, current_time: Instant) {
        use VirtualButtonState::*;

        for state in self.map.values_mut() {
            *state = match state {
                Pressed(maybe_instant, _duration) => match maybe_instant {
                    Some(instant) => Pressed(Some(*instant), current_time - *instant),
                    None => Pressed(Some(current_time), Duration::ZERO),
                },
                Released(maybe_instant, _duration) => match maybe_instant {
                    Some(instant) => Released(Some(*instant), current_time - *instant),
                    None => Released(Some(current_time), Duration::ZERO),
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
        if let VirtualButtonState::Released(_, _) = self.state(action) {
            self.map.insert(action, VirtualButtonState::JUST_PRESSED);
        }
    }

    /// Release the `action` virtual button
    pub fn release(&mut self, action: A) {
        if let VirtualButtonState::Pressed(_, _) = self.state(action) {
            self.map.insert(action, VirtualButtonState::JUST_RELEASED);
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
        match self.state(action) {
            VirtualButtonState::Pressed(_, _) => true,
            VirtualButtonState::Released(_, _) => false,
        }
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[must_use]
    pub fn just_pressed(&self, action: A) -> bool {
        match self.state(action) {
            // We cannot check that the duration is zero, or events will be double-counted
            VirtualButtonState::Pressed(maybe_instant, _) => maybe_instant.is_none(),
            VirtualButtonState::Released(_, _) => false,
        }
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[must_use]
    pub fn released(&self, action: A) -> bool {
        match self.state(action) {
            VirtualButtonState::Pressed(_, _) => false,
            VirtualButtonState::Released(_, _) => true,
        }
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[must_use]
    pub fn just_released(&self, action: A) -> bool {
        match self.state(action) {
            VirtualButtonState::Pressed(_, _) => false,
            // We cannot check that the duration is zero, or events will be double-counted
            VirtualButtonState::Released(maybe_instant, _) => maybe_instant.is_none(),
        }
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
}
