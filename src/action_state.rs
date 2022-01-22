//! This module contains [ActionState] and its supporting methods and impls.

use crate::{Actionlike, InputMap};
use bevy::prelude::*;
use bevy::utils::HashMap;

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Intended to be used as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Example
/// ```rust
/// use leafwing_input_manager::prelude::*;
/// use strum_macros::EnumIter;
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
/// action_state.press(Action::Jump);
///
/// assert!(action_state.pressed(Action::Jump));
/// assert!(action_state.just_pressed(Action::Jump));
/// assert!(action_state.released(Action::Left));
///
/// // Resets just_pressed and just_released
/// action_state.tick();
/// assert!(action_state.pressed(Action::Jump));
/// assert!(!action_state.just_pressed(Action::Jump));
///
/// action_state.release(Action::Jump);
/// assert!(!action_state.pressed(Action::Jump));
/// assert!(action_state.released(Action::Jump));
/// assert!(action_state.just_released(Action::Jump));
///
/// action_state.tick();
/// assert!(action_state.released(Action::Jump));
/// assert!(!action_state.just_released(Action::Jump));
/// ```
#[derive(Component)]
pub struct ActionState<A: Actionlike> {
    pressed: HashMap<A, bool>,
    just_pressed: HashMap<A, bool>,
    just_released: HashMap<A, bool>,
}

impl<A: Actionlike> ActionState<A> {
    /// Updates the [`ActionState`] based on the [`InputMap`] and the provided [`Input`]s
    ///
    /// Presses and releases buttons according to the current state of the inputs.
    /// Combine with [ActionState::tick] to update `just_pressed` and `just_released`.
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

    /// Clears all `just_pressed` and `just_released` state
    pub fn tick(&mut self) {
        self.just_pressed = Self::default_map();
        self.just_released = Self::default_map();
    }

    /// Press the `action` virtual button
    pub fn press(&mut self, action: A) {
        if !self.pressed(action) {
            self.just_pressed.insert(action, true);
        }
        self.pressed.insert(action, true);
    }

    /// Release the `action` virtual button
    pub fn release(&mut self, action: A) {
        if !self.released(action) {
            self.just_released.insert(action, true);
        }
        self.pressed.insert(action, false);
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
        *self.pressed.get(&action).unwrap()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[must_use]
    pub fn just_pressed(&self, action: A) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[must_use]
    pub fn released(&self, action: A) -> bool {
        !*self.pressed.get(&action).unwrap()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[must_use]
    pub fn just_released(&self, action: A) -> bool {
        *self.just_released.get(&action).unwrap()
    }

    /// Creates a Hashmap with all of the possible A variants as keys, and false as the values
    #[must_use]
    pub fn default_map() -> HashMap<A, bool> {
        // PERF: optimize construction through pre-allocation or constification
        let mut map = HashMap::default();

        for action in A::iter() {
            map.insert(action, false);
        }
        map
    }
}

impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            pressed: Self::default_map(),
            just_pressed: Self::default_map(),
            just_released: Self::default_map(),
        }
    }
}

/// A component that allows the attached entity to drive the [ActionState] of the associated entity
///
/// Used in [update_action_state_from_interaction](crate::systems::update_action_state_from_interaction).
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionStateDriver<A: Actionlike> {
    /// The action triggered by this entity
    pub action: A,
    /// The entity whose action state should be updated
    pub entity: Entity,
}

mod tests {
    use crate::prelude::*;
    use strum_macros::EnumIter;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn press_lifecycle() {
        use bevy::prelude::*;

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
        assert!(!action_state.just_released(Action::Run));

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
        action_state.tick();
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
        action_state.tick();
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
