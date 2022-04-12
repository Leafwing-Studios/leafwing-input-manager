//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::buttonlike::ButtonState;
use crate::user_input::UserInput;
use crate::Actionlike;

use bevy_ecs::{component::Component, entity::Entity};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Metadata about an [`Actionlike`] action
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionData {
    /// Is the action pressed or released?
    pub state: ButtonState,
    /// What inputs were responsible for causing this action to be pressed?
    pub reasons_pressed: Vec<UserInput>,
}

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Intended to be used as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Example
/// ```rust
/// use leafwing_input_manager::prelude::*;
///
/// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Debug)]
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
/// action_state.tick();
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
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionState<A: Actionlike> {
    action_data: Vec<ActionData>,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> ActionState<A> {
    /// Updates the [`ActionState`] based on a vector of [`ActionData`], ordered by [`Actionlike::id`](Actionlike).
    ///
    /// The `action_data` is typically constructed from [`InputMap::which_pressed`](crate::input_map::InputMap),
    /// which reads from the assorted [`Input`](bevy::input::Input) resources.
    pub fn update(&mut self, action_data: Vec<ActionData>) {
        assert_eq!(action_data.len(), A::N_VARIANTS);

        for (i, action) in A::variants().enumerate() {
            match action_data[i].state {
                ButtonState::JustPressed => self.press(action),
                ButtonState::Pressed => self.press(action),
                ButtonState::JustReleased => self.release(action),
                ButtonState::Released => self.release(action),
            }

            self.action_data[i].reasons_pressed = action_data[i].reasons_pressed.clone();
        }
    }

    /// Advances the time for all virtual buttons
    ///
    /// The underlying [`VirtualButtonState`] state will be advanced according to the `current_instant`.
    /// - if no [`Instant`] is set, the `current_instant` will be set as the initial time at which the button was pressed / released
    /// - the [`Duration`] will advance to reflect elapsed time
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::VirtualButtonState;
    /// use bevy_utils::Instant;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// // Virtual buttons start released
    /// assert!(action_state.action_data(Action::Run).just_released());
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
    pub fn tick(&mut self) {
        self.action_data.iter_mut().for_each(|ad| ad.state.tick());
    }

    /// Gets a copy of the [`ActionData`] of the corresponding `action`
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::VirtualButtonState;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    /// let mut action_state = ActionState::<Action>::default();
    /// let run_state = action_state.action_data(Action::Run);
    ///
    /// // States can either be pressed or released,
    /// // and store an internal `Timing`
    /// if let VirtualButtonState::Pressed{timing, reasons_pressed: _} = run_state {
    ///     let pressed_duration = timing.current_duration;
    ///     let last_released_duration = timing.previous_duration;
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn action_data(&self, action: A) -> ActionData {
        self.action_data[action.index()].clone()
    }

    /// Manually sets the [`VirtualButtonState`] of the corresponding `action`
    ///
    /// You should almost always be using the [`ActionState::press`] and [`ActionState::release`] methods instead,
    /// as they will ensure that the duration is correct.
    ///
    /// However, this method can be useful for testing,
    /// or when transferring [`VirtualButtonState`] between action maps.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::VirtualButtonState;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    /// enum AbilitySlot {
    ///     Slot1,
    ///     Slot2,
    /// }
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut ability_slot_state = ActionState::<AbilitySlot>::default();
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// // Extract the state from the ability slot
    /// let slot_1_state = ability_slot_state.action_data(AbilitySlot::Slot1);
    ///
    /// // And transfer it to the actual ability that we care about
    /// // without losing timing information
    /// action_state.set_action_data(Action::Run, slot_1_state);
    /// ```
    #[inline]
    pub fn set_action_data(&mut self, action: A, data: ActionData) {
        self.action_data[action.index()] = data;
    }

    /// Press the `action` virtual button
    ///
    /// No inititial instant or reasons why the button was pressed will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn press(&mut self, action: A) {
        self.action_data[action.index()].state.press();
    }

    /// Release the `action` virtual button
    ///
    /// No inititial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn release(&mut self, action: A) {
        self.action_data[action.index()].state.release();
        self.action_data[action.index()].reasons_pressed = Vec::new();
    }

    /// Releases all action virtual buttons
    pub fn release_all(&mut self) {
        for action in A::variants() {
            self.release(action);
        }
    }

    /// Is this `action` currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self, action: A) -> bool {
        self.action_data[action.index()].state.pressed()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self, action: A) -> bool {
        self.action_data[action.index()].state.just_pressed()
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[inline]
    #[must_use]
    pub fn released(&self, action: A) -> bool {
        self.action_data[action.index()].state.released()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_released(&self, action: A) -> bool {
        self.action_data[action.index()].state.just_released()
    }

    #[must_use]
    /// Which actions are currently pressed?
    pub fn get_pressed(&self) -> Vec<A> {
        A::variants().filter(|a| self.pressed(a.clone())).collect()
    }

    #[must_use]
    /// Which actions were just pressed?
    pub fn get_just_pressed(&self) -> Vec<A> {
        A::variants()
            .filter(|a| self.just_pressed(a.clone()))
            .collect()
    }

    #[must_use]
    /// Which actions are currently released?
    pub fn get_released(&self) -> Vec<A> {
        A::variants().filter(|a| self.released(a.clone())).collect()
    }

    #[must_use]
    /// Which actions were just released?
    pub fn get_just_released(&self) -> Vec<A> {
        A::variants()
            .filter(|a| self.just_released(a.clone()))
            .collect()
    }

    /// The reasons (in terms of [`UserInput`]) that the button was pressed
    ///
    /// If the button is currently released, the `Vec<UserInput`> returned will be empty
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::{VirtualButtonState, Timing};
    /// use bevy_input::keyboard::KeyCode;
    ///
    /// #[derive(Actionlike, Clone)]
    /// enum PlatformerAction{
    ///     Move,
    ///     Jump,
    /// }
    ///
    /// let mut action_state = ActionState::<PlatformerAction>::default();
    ///
    /// // Usually this will be done automatically for you, via [`ActionState::update`]
    /// action_state.set_action_data(PlatformerAction::Jump,
    ///     VirtualButtonState::Pressed {
    ///         // For the sake of this example, we don't care about the timing information
    ///         timing: Timing::default(),
    ///         reasons_pressed: vec![KeyCode::Space.into()],
    ///     }
    /// );
    ///
    /// let reasons_jumped = action_state.reasons_pressed(PlatformerAction::Jump);
    /// assert_eq!(reasons_jumped[0], KeyCode::Space.into());
    /// ```
    #[inline]
    #[must_use]
    pub fn reasons_pressed(&self, action: A) -> Vec<UserInput> {
        self.action_data[action.index()].reasons_pressed.clone()
    }
}

impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> ActionState<A> {
        ActionState {
            action_data: A::variants()
                .map(|_| ActionData {
                    state: ButtonState::Released,
                    reasons_pressed: Vec::new(),
                })
                .collect(),
            _phantom: PhantomData::default(),
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

mod tests {
    use crate as leafwing_input_manager;
    use crate::prelude::*;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn press_lifecycle() {
        use crate::user_input::InputStreams;
        use bevy::prelude::*;

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::R);

        // Input streams
        let mut keyboard_input_stream = Input::<KeyCode>::default();
        let input_streams = InputStreams::from_keyboard(&keyboard_input_stream);

        // Starting state
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(action_state.just_released(Action::Run));

        // Pressing
        keyboard_input_stream.press(KeyCode::R);
        let input_streams = InputStreams::from_keyboard(&keyboard_input_stream);

        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(Action::Run));
        assert!(action_state.just_pressed(Action::Run));
        assert!(!action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));

        // Waiting
        action_state.tick();
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(!action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));

        // Releasing
        keyboard_input_stream.release(KeyCode::R);
        let input_streams = InputStreams::from_keyboard(&keyboard_input_stream);

        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(action_state.just_released(Action::Run));

        // Waiting
        action_state.tick();
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));
    }
}

/// Stores presses and releases of buttons without timing information
///
/// These are typically accessed using the `Events<ActionDiff>` resource.
/// Uses a minimal storage format, in order to facilitate transport over the network.
///
/// `ID` should be a component type that stores a unique stable identifier for the entity
/// that stores the corresponding [`ActionState`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionDiff<A: Actionlike, ID: Eq + Clone + Component> {
    /// The virtual button was pressed
    Pressed {
        /// The value of the action
        action: A,
        /// The stable identifier of the entity
        id: ID,
    },
    /// The virtual button was released
    Released {
        /// The value of the action
        action: A,
        /// The stable identifier of the entity
        id: ID,
    },
}

mod test {
    use crate as leafwing_input_manager;
    use crate::Actionlike;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    enum Action {
        Run,
        Jump,
    }

    #[test]
    fn time_tick_ticks_away() {
        use crate::action_state::ActionState;

        let mut action_state = ActionState::<Action>::default();

        // Action states start fully released
        dbg!(action_state.get_released());
        dbg!(action_state.clone());

        // Virtual buttons start released (but not just released)
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Jump));

        // Ticking causes buttons that were just released to no longer be just released
        action_state.tick();
        assert!(action_state.released(Action::Jump));
        assert!(!action_state.just_released(Action::Jump));
        action_state.press(Action::Jump);
        assert!(action_state.just_pressed(Action::Jump));

        // Ticking causes buttons that were just pressed to no longer be just pressed
        action_state.tick();
        assert!(action_state.pressed(Action::Jump));
        assert!(!action_state.just_pressed(Action::Jump));
    }
}
