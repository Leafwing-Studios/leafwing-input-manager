//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::buttonlike::{Timing, VirtualButtonState};
use crate::user_input::UserInput;
use crate::Actionlike;

use bevy_ecs::{component::Component, entity::Entity};
use bevy_utils::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Intended to be used as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Example
/// ```rust
/// use leafwing_input_manager::prelude::*;
/// use bevy_utils::Instant;
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
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionState<A: Actionlike> {
    button_states: Vec<VirtualButtonState>,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> ActionState<A> {
    /// Updates the [`ActionState`] based on a [`Vec<VirtualButtonState>`] of pressed virtual buttons, ordered by [`Actionlike::id`](Actionlike).
    ///
    /// The `pressed_list` is typically constructed from [`InputMap::which_pressed`](crate::input_map::InputMap),
    /// which reads from the assorted [`Input`](bevy::input::Input) resources.
    pub fn update(&mut self, pressed_list: Vec<VirtualButtonState>) {
        for (i, button_state) in pressed_list.iter().enumerate() {
            match button_state {
                VirtualButtonState::Pressed {
                    timing: _,
                    reasons_pressed: new_inputs,
                } => {
                    if self.button_states[i].released() {
                        self.press(A::get_at(i).unwrap())
                    }
                    if let VirtualButtonState::Pressed {
                        timing: _,
                        reasons_pressed: ref mut inputs,
                    } = self.button_states[i]
                    {
                        *inputs = new_inputs.clone();
                    }
                }
                VirtualButtonState::Released { timing: _ } => {
                    if self.button_states[i].pressed() {
                        self.release(A::get_at(i).unwrap())
                    }
                }
            }
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
    /// assert!(action_state.button_state(Action::Run).just_released());
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
        for state in self.button_states.iter_mut() {
            state.tick(current_instant);
        }
    }

    /// Gets the [`VirtualButtonState`] of the corresponding `action`
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the state directly allows you to examine the detailed [`Timing`] information.
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
    /// let run_state = action_state.button_state(Action::Run);
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
    pub fn button_state(&self, action: A) -> VirtualButtonState {
        self.button_states[action.index()].clone()
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
    /// let slot_1_state = ability_slot_state.button_state(AbilitySlot::Slot1);
    ///
    /// // And transfer it to the actual ability that we care about
    /// // without losing timing information
    /// action_state.set_button_state(Action::Run, slot_1_state);
    /// ```
    #[inline]
    pub fn set_button_state(&mut self, action: A, state: VirtualButtonState) {
        self.button_states[action.index()] = state;
    }

    /// Press the `action` virtual button
    ///
    /// No inititial instant or reasons why the button was pressed will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn press(&mut self, action: A) {
        self.button_states[action.index()].press(None, Vec::new());
    }

    /// Release the `action` virtual button
    ///
    /// No inititial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn release(&mut self, action: A) {
        self.button_states[action.index()].release(None);
    }

    /// Releases all action virtual buttons
    pub fn release_all(&mut self) {
        for action in A::variants() {
            self.release(action);
        }
    }

    /// Fully resets the state of the `action`
    ///
    /// The action will be released, but will no longer be `just_pressed` or `just_released`,
    /// all timing information will be lost and all `reasons_pressed` will be wiped.
    ///
    /// This is occsaionally useful to avoid triggering just-pressed and just-released events
    /// during various transitions.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// use leafwing_input_manager::Actionlike;
    /// use leafwing_input_manager::action_state::ActionState;
    /// use bevy_utils::Instant;
    ///
    /// #[derive(Actionlike, Clone)]
    /// enum MenuAction {
    ///     Open,
    ///     Close,
    /// }
    ///
    /// let mut action_state = ActionState::<MenuAction>::default();
    /// action_state.press(MenuAction::Open);
    ///
    /// assert!(action_state.pressed(MenuAction::Open));
    /// assert!(action_state.just_pressed(MenuAction::Open));
    ///
    /// // Go directly to Released, do not pass Just Released,
    /// // do not collect $200.
    /// action_state.reset(MenuAction::Open, Instant::now());
    ///
    /// assert!(!action_state.just_pressed(MenuAction::Open));
    /// assert!(!action_state.just_released(MenuAction::Open));
    /// assert!(action_state.released(MenuAction::Open));
    /// ```
    ///
    /// This system demonstrates how you might use this API in practice:
    ///
    /// ```rust
    /// use bevy_ecs::prelude::*;
    /// use bevy_core::Time;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone)]
    /// enum MenuAction {
    ///     Open,
    ///     Close,
    /// }
    ///
    /// fn reset_menu(mut action_state: ResMut<ActionState<MenuAction>>, time: Res<Time>){
    ///    let current_time = time.last_update().expect("Time has been updated at least once.");
    ///    for action in MenuAction::variants(){
    ///         // This causes the action to look as if it had been released on
    ///         // the schedule tick before the current tick,
    ///         // completely skipping over just_released
    ///         action_state.reset(action, current_time);
    ///    }
    /// }
    /// ```
    #[inline]
    pub fn reset(&mut self, action: A, current_instant: Instant) {
        self.button_states[action.index()] = VirtualButtonState::Released {
            timing: Timing {
                instant_started: Some(current_instant),
                current_duration: Duration::ZERO,
                previous_duration: Duration::ZERO,
            },
        };
    }

    /// Is this `action` currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self, action: A) -> bool {
        self.button_state(action).pressed()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self, action: A) -> bool {
        self.button_state(action).just_pressed()
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[inline]
    #[must_use]
    pub fn released(&self, action: A) -> bool {
        self.button_state(action).released()
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_released(&self, action: A) -> bool {
        self.button_state(action).just_released()
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
    /// action_state.set_button_state(PlatformerAction::Jump,
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
        self.button_states[action.index()].reasons_pressed()
    }
}

impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> ActionState<A> {
        ActionState {
            button_states: A::variants()
                .map(|_| VirtualButtonState::JUST_RELEASED)
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
        use bevy_utils::Instant;

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
        action_state.tick(Instant::now());
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
        action_state.tick(Instant::now());
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));
    }

    #[test]
    fn durations() {
        use bevy_utils::{Duration, Instant};
        use std::thread::sleep;

        let mut action_state = ActionState::<Action>::default();

        // Virtual buttons start released
        assert!(action_state.button_state(Action::Jump).released());
        assert_eq!(
            action_state.button_state(Action::Jump).instant_started(),
            None,
        );
        assert_eq!(
            action_state.button_state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.button_state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Pressing a button swaps the state
        action_state.press(Action::Jump);
        assert!(action_state.button_state(Action::Jump).pressed());
        assert_eq!(
            action_state.button_state(Action::Jump).instant_started(),
            None
        );
        assert_eq!(
            action_state.button_state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.button_state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Ticking time sets the instant for the new state
        let t0 = Instant::now();
        action_state.tick(t0);
        assert_eq!(
            action_state.button_state(Action::Jump).instant_started(),
            Some(t0)
        );
        assert_eq!(
            action_state.button_state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.button_state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Time passes
        sleep(Duration::from_micros(1));
        let t1 = Instant::now();

        // The duration is updated
        action_state.tick(t1);
        assert_eq!(
            action_state.button_state(Action::Jump).instant_started(),
            Some(t0)
        );
        assert_eq!(
            action_state.button_state(Action::Jump).current_duration(),
            t1 - t0
        );
        assert_eq!(
            action_state.button_state(Action::Jump).previous_duration(),
            Duration::ZERO
        );

        // Releasing again, swapping the current duration to the previous one
        action_state.release(Action::Jump);
        assert_eq!(
            action_state.button_state(Action::Jump).instant_started(),
            None
        );
        assert_eq!(
            action_state.button_state(Action::Jump).current_duration(),
            Duration::ZERO
        );
        assert_eq!(
            action_state.button_state(Action::Jump).previous_duration(),
            t1 - t0,
        );
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
        use std::time::Instant;

        let mut action_state = ActionState::<Action>::default();

        // Action states start fully released
        dbg!(action_state.get_released());
        dbg!(action_state.clone());

        // Virtual buttons start released
        assert!(action_state.button_state(Action::Run).just_released());
        assert!(action_state.just_released(Action::Jump));

        // Ticking time moves causes buttons that were just released to no longer be just released
        action_state.tick(Instant::now());
        assert!(action_state.released(Action::Jump));
        assert!(!action_state.just_released(Action::Jump));
        action_state.press(Action::Jump);
        assert!(action_state.just_pressed(Action::Jump));

        // Ticking time moves causes buttons that were just pressed to no longer be just pressed
        action_state.tick(Instant::now());
        assert!(action_state.pressed(Action::Jump));
        assert!(!action_state.just_pressed(Action::Jump));
    }
}
