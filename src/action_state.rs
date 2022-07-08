//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::buttonlike::ButtonState;
use crate::user_input::UserInput;
use crate::Actionlike;

use bevy_ecs::{component::Component, entity::Entity};
use bevy_utils::{Duration, Instant};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// Metadata about an [`Actionlike`] action
///
/// If a button is released, its `reasons_pressed` should be empty.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionData {
    /// Is the action pressed or released?
    pub state: ButtonState,
    /// What inputs were responsible for causing this action to be pressed?
    pub reasons_pressed: Vec<UserInput>,
    /// When was the button pressed / released, and how long has it been held for?
    pub timing: Timing,
    /// Was this action consumed by [`ActionState::consume`]?
    ///
    /// Actions that are consumed cannot be pressed again until they are explicitly released.
    /// This ensures that consumed actions are not immediately re-pressed by continued inputs.
    pub consumed: bool,
}

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Can be used as either a resource or as a [`Component`] on entities that you wish to control directly from player input.
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
/// let t0 = Instant::now();
/// let t1 = Instant::now();
///
///  action_state.tick(t1, t0);
/// assert!(action_state.pressed(Action::Jump));
/// assert!(!action_state.just_pressed(Action::Jump));
///
/// action_state.release(Action::Jump);
/// assert!(!action_state.pressed(Action::Jump));
/// assert!(action_state.released(Action::Jump));
/// assert!(action_state.just_released(Action::Jump));
///
/// let t2 = Instant::now();
/// action_state.tick(t2, t1);
/// assert!(action_state.released(Action::Jump));
/// assert!(!action_state.just_released(Action::Jump));
/// ```
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionState<A: Actionlike> {
    /// The [`ActionData`] of each action
    ///
    /// The position in this vector corresponds to [`Actionlike::index`].
    pub action_data: Vec<ActionData>,
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

    /// Advances the time for all actions
    ///
    /// The underlying [`Timing`] and [`ButtonState`] will be advanced according to the `current_instant`.
    /// - if no [`Instant`] is set, the `current_instant` will be set as the initial time at which the button was pressed / released
    /// - the [`Duration`] will advance to reflect elapsed time
    ///
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::ButtonState;
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
    /// // Actions start released
    /// assert!(action_state.released(Action::Jump));
    /// assert!(!action_state.just_released(Action::Run));
    ///
    /// // Ticking time moves causes buttons that were just released to no longer be just released
    /// let t0 = Instant::now();
    /// let t1 = Instant::now();
    ///
    /// action_state.tick(t1, t0);
    /// assert!(action_state.released(Action::Jump));
    /// assert!(!action_state.just_released(Action::Jump));
    ///
    /// action_state.press(Action::Jump);
    /// assert!(action_state.just_pressed(Action::Jump));
    ///
    /// // Ticking time moves causes buttons that were just pressed to no longer be just pressed
    /// let t2 = Instant::now();
    ///
    /// action_state.tick(t2, t1);
    /// assert!(action_state.pressed(Action::Jump));
    /// assert!(!action_state.just_pressed(Action::Jump));
    /// ```
    pub fn tick(&mut self, current_instant: Instant, previous_instant: Instant) {
        // Advanced the ButtonState
        self.action_data.iter_mut().for_each(|ad| ad.state.tick());

        // Advance the Timings
        self.action_data.iter_mut().for_each(|ad| {
            // Durations should not advance while actions are consumed
            if !ad.consumed {
                ad.timing.tick(current_instant, previous_instant)
            }
        });
    }

    /// Gets a copy of the [`ActionData`] of the corresponding `action`
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    /// let mut action_state = ActionState::<Action>::default();
    /// let run_data = action_state.action_data(Action::Run);
    ///
    /// dbg!(run_data);
    /// ```
    #[inline]
    #[must_use]
    pub fn action_data(&self, action: A) -> ActionData {
        self.action_data[action.index()].clone()
    }

    /// Manually sets the [`ActionData`] of the corresponding `action`
    ///
    /// You should almost always use more direct methods, as they are simpler and less error-prone.
    ///
    /// However, this method can be useful for testing,
    /// or when transferring [`ActionData`] between action states.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
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

    /// Press the `action`
    ///
    /// No initial instant or reasons why the button was pressed will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn press(&mut self, action: A) {
        let index = action.index();
        // Consumed actions cannot be pressed until they are released
        if self.action_data[index].consumed {
            return;
        }

        if self.released(action) {
            self.action_data[index].timing.flip();
        }

        self.action_data[index].state.press();
    }

    /// Release the `action`
    ///
    /// No initial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn release(&mut self, action: A) {
        let index = action.index();
        // Once released, consumed actions can be pressed again
        self.action_data[index].consumed = false;

        if self.pressed(action) {
            self.action_data[index].timing.flip();
            self.action_data[index].reasons_pressed = Vec::new();
        }

        self.action_data[index].state.release();
    }

    /// Consumes the `action`
    ///
    /// The action will be released, and will not be able to be pressed again
    /// until it would have otherwise been released by [`ActionState::release`],
    /// [`ActionState::release_all`] or [`ActionState::update`].
    ///
    /// No initial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    /// enum Action {
    ///     Eat,
    ///     Sleep,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// action_state.press(Action::Eat);
    /// assert!(action_state.pressed(Action::Eat));
    ///
    /// // Consuming actions releases them
    /// action_state.consume(Action::Eat);
    /// assert!(action_state.released(Action::Eat));
    ///
    /// // Doesn't work, as the action was consumed
    /// action_state.press(Action::Eat);
    /// assert!(action_state.released(Action::Eat));
    ///
    /// // Releasing consumed actions allows them to be pressed again
    /// action_state.release(Action::Eat);
    /// action_state.press(Action::Eat);
    /// assert!(action_state.pressed(Action::Eat));
    /// ```
    #[inline]
    pub fn consume(&mut self, action: A) {
        let index = action.index();
        // This is the only difference from action_state.release(action)
        self.action_data[index].consumed = true;
        self.action_data[index].state.release();
        self.action_data[index].reasons_pressed = Vec::new();
        self.action_data[index].timing.flip();
    }

    /// Releases all actions
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
    /// use leafwing_input_manager::buttonlike::ButtonState;
    /// use leafwing_input_manager::action_state::ActionData;
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
    ///   ActionData {
    ///         state: ButtonState::JustPressed,
    ///         // Manually setting the reason why this action was pressed
    ///         reasons_pressed: vec![KeyCode::Space.into()],
    ///         // For the sake of this example, we don't care about any other fields
    ///         ..Default::default()
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

    /// The [`Instant`] that the action was last pressed or released
    ///
    /// If the action was pressed or released since the last time [`ActionState::tick`] was called
    /// the value will be [`None`].
    /// This ensures that all of our actions are assigned a timing and duration
    /// that corresponds exactly to the start of a frame, rather than relying on idiosyncratic timing.
    pub fn instant_started(&self, action: A) -> Option<Instant> {
        self.action_data[action.index()].timing.instant_started
    }

    /// The [`Duration`] for which the action has been held or released
    pub fn current_duration(&self, action: A) -> Duration {
        self.action_data[action.index()].timing.current_duration
    }

    /// The [`Duration`] for which the action was last held or released
    ///
    /// This is a snapshot of the [`ActionState::current_duration`] state at the time
    /// the action was last pressed or released.
    pub fn previous_duration(&self, action: A) -> Duration {
        self.action_data[action.index()].timing.previous_duration
    }
}

impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> ActionState<A> {
        ActionState {
            action_data: A::variants().map(|_| ActionData::default()).collect(),
            _phantom: PhantomData::default(),
        }
    }
}

/// A component that allows the attached entity to drive the [`ActionState`] of the associated entity
///
/// # Examples
///
/// By default, [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction) uses this component
/// in order to connect `bevy_ui` buttons to the corresponding `ActionState`.
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// #[derive(Actionlike, Clone, Copy)]
/// enum DanceDance {
///     Left,
///     Right,
///     Up,
///     Down,
/// }
///
/// // Spawn entity to track dance inputs
/// let mut world = World::new();
/// let dance_tracker = world
///     .spawn()
///     .insert(ActionState::<DanceDance>::default())
///     .id();
///
/// // Spawn a button, which is wired up to the dance tracker
/// // When used with InputManagerPlugin<DanceDance>, this button will press the DanceDance::Left action when it is pressed.
/// world
///     .spawn()
///     .insert_bundle(ButtonBundle::default())
///     // This component links the button to the entity with the `ActionState` component
///     .insert(ActionStateDriver {
///         action: DanceDance::Left,
///         entity: dance_tracker,
///     });
///```
///
/// Writing your own systems that use the [`ActionStateDriver`] component is easy,
/// although this should be reserved for cases where the entity whose value you want to check
/// is distinct from the entity whose [`ActionState`] you want to set.
/// Check the source code of [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction) for an example of how this is done.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionStateDriver<A: Actionlike> {
    /// The action triggered by this entity
    pub action: A,
    /// The entity whose action state should be updated
    pub entity: Entity,
}

/// Stores information about when an action was pressed or released
///
/// This struct is principally used as a field on [`ActionData`],
/// which itself lives inside an [`ActionState`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Timing {
    /// The [`Instant`] at which the button was pressed or released
    /// Recorded as the [`Time`](bevy::core::Time) at the start of the tick after the state last changed.
    /// If this is none, [`Timing::tick`] has not been called yet.
    #[serde(skip)]
    pub instant_started: Option<Instant>,
    /// The [`Duration`] for which the button has been pressed or released.
    ///
    /// This begins at [`Duration::ZERO`] when [`ActionState::update`] is called.
    pub current_duration: Duration,
    /// The [`Duration`] for which the button was pressed or released before the state last changed.
    pub previous_duration: Duration,
}

impl PartialOrd for Timing {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.current_duration.partial_cmp(&other.current_duration)
    }
}

impl Timing {
    /// Advances the `current_duration` of this timer
    ///
    /// If the `instant_started` is None, it will be set to the current time.
    /// This design allows us to ensure that the timing is always synchronized with the start of each frame.
    pub fn tick(&mut self, current_instant: Instant, previous_instant: Instant) {
        if let Some(instant_started) = self.instant_started {
            self.current_duration = current_instant - instant_started;
        } else {
            self.current_duration = current_instant - previous_instant;
            self.instant_started = Some(previous_instant);
        }
    }

    /// Flips the metaphorical hourglass, storing `current_duration` in `previous_duration` and resetting `instant_started`
    ///
    /// This method is called whenever actions are pressed or released
    ///
    /// FIXME: Ensure that the timing starts on the same frame that the input is flipped.
    pub fn flip(&mut self) {
        self.previous_duration = self.current_duration;
        self.current_duration = Duration::ZERO;
        self.instant_started = None;
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
    /// The action was pressed
    Pressed {
        /// The value of the action
        action: A,
        /// The stable identifier of the entity
        id: ID,
    },
    /// The action was released
    Released {
        /// The value of the action
        action: A,
        /// The stable identifier of the entity
        id: ID,
    },
}

mod tests {
    use crate as leafwing_input_manager;
    use crate::Actionlike;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn press_lifecycle() {
        use crate::action_state::ActionState;
        use crate::clashing_inputs::ClashStrategy;
        use crate::input_map::InputMap;
        use crate::user_input::InputStreams;
        use bevy::prelude::*;
        use bevy_utils::{Duration, Instant};

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(KeyCode::R, Action::Run);

        // Input streams
        let mut keyboard_input_stream = Input::<KeyCode>::default();
        let input_streams = InputStreams::from_keyboard(&keyboard_input_stream);

        // Starting state
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));

        // Pressing
        keyboard_input_stream.press(KeyCode::R);
        let input_streams = InputStreams::from_keyboard(&keyboard_input_stream);

        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(Action::Run));
        assert!(action_state.just_pressed(Action::Run));
        assert!(!action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
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
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.which_pressed(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(Action::Run));
        assert!(!action_state.just_pressed(Action::Run));
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Run));
    }

    #[test]
    fn time_tick_ticks_away() {
        use crate::action_state::ActionState;
        use bevy_utils::{Duration, Instant};

        let mut action_state = ActionState::<Action>::default();

        // Actions start released (but not just released)
        assert!(action_state.released(Action::Run));
        assert!(!action_state.just_released(Action::Jump));

        // Ticking causes buttons that were just released to no longer be just released
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        assert!(action_state.released(Action::Jump));
        assert!(!action_state.just_released(Action::Jump));
        action_state.press(Action::Jump);
        assert!(action_state.just_pressed(Action::Jump));

        // Ticking causes buttons that were just pressed to no longer be just pressed
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        assert!(action_state.pressed(Action::Jump));
        assert!(!action_state.just_pressed(Action::Jump));
    }

    // FIXME: these tests are flaky because floats
    #[test]
    fn durations() {
        use crate::action_state::ActionState;
        use bevy_utils::{Duration, Instant};

        let mut action_state = ActionState::<Action>::default();

        // Actions start released
        assert!(action_state.released(Action::Jump));
        assert_eq!(action_state.instant_started(Action::Jump), None,);
        assert_eq!(action_state.current_duration(Action::Jump), Duration::ZERO);
        assert_eq!(action_state.previous_duration(Action::Jump), Duration::ZERO);

        // Pressing a button swaps the state
        action_state.press(Action::Jump);
        assert!(action_state.pressed(Action::Jump));
        assert_eq!(action_state.instant_started(Action::Jump), None);
        assert_eq!(action_state.current_duration(Action::Jump), Duration::ZERO);
        assert_eq!(action_state.previous_duration(Action::Jump), Duration::ZERO);

        // Ticking time sets the instant for the new state
        let t0 = Instant::now();
        let t1 = t0 + Duration::new(1, 0);

        action_state.tick(t1, t0);
        assert_eq!(action_state.instant_started(Action::Jump), Some(t0));
        assert_eq!(action_state.current_duration(Action::Jump), t1 - t0);
        assert_eq!(action_state.previous_duration(Action::Jump), Duration::ZERO);

        // Time passes
        let t2 = t1 + Duration::new(5, 0);

        // The duration is updated
        action_state.tick(t2, t1);
        assert_eq!(action_state.instant_started(Action::Jump), Some(t0));
        assert_eq!(action_state.current_duration(Action::Jump), t2 - t0);
        assert_eq!(action_state.previous_duration(Action::Jump), Duration::ZERO);

        // Releasing again, swapping the current duration to the previous one
        action_state.release(Action::Jump);
        assert_eq!(action_state.instant_started(Action::Jump), None);
        assert_eq!(action_state.current_duration(Action::Jump), Duration::ZERO);
        assert_eq!(action_state.previous_duration(Action::Jump), t2 - t0);
    }
}
