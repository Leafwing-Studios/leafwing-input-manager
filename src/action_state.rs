//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::action_diff::ActionDiff;
#[cfg(feature = "timing")]
use crate::timing::Timing;
use crate::Actionlike;
use crate::{axislike::DualAxisData, buttonlike::ButtonState};

use bevy::ecs::component::Component;
use bevy::prelude::Resource;
use bevy::reflect::Reflect;
#[cfg(feature = "timing")]
use bevy::utils::Duration;
use bevy::utils::{HashMap, Instant};
use serde::{Deserialize, Serialize};

/// Metadata about an [`Actionlike`] action
///
/// If a button is released, its `reasons_pressed` should be empty.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ActionData {
    /// Is the action pressed or released?
    pub state: ButtonState,
    /// The "value" of the binding that triggered the action.
    ///
    /// See [`ActionState::value`] for more details.
    ///
    /// **Warning:** this value may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs.
    pub value: f32,
    /// The [`DualAxisData`] of the binding that triggered the action.
    pub axis_pair: Option<DualAxisData>,
    /// When was the button pressed / released, and how long has it been held for?
    #[cfg(feature = "timing")]
    pub timing: Timing,
    /// Was this action consumed by [`ActionState::consume`]?
    ///
    /// Actions that are consumed cannot be pressed again until they are explicitly released.
    /// This ensures that consumed actions are not immediately re-pressed by continued inputs.
    pub consumed: bool,
    /// Is the action disabled?
    ///
    /// While disabled, an action will always report as released, regardless of its actual state.
    pub disabled: bool,
}

impl ActionData {
    /// Is the action currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self) -> bool {
        !self.disabled && self.state.pressed()
    }

    /// Was the action pressed since the last time it was ticked?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self) -> bool {
        !self.disabled && self.state.just_pressed()
    }

    /// Is the action currently released?
    #[inline]
    #[must_use]
    pub fn released(&self) -> bool {
        self.disabled || self.state.released()
    }

    /// Was the action released since the last time it was ticked?
    #[inline]
    #[must_use]
    pub fn just_released(&self) -> bool {
        !self.disabled && self.state.just_released()
    }
}

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Can be used as either a resource or as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Example
/// ```rust
/// use bevy::reflect::Reflect;
/// use leafwing_input_manager::prelude::*;
/// use bevy::utils::Instant;
///
/// #[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
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
/// action_state.press(&Action::Jump);
///
/// assert!(action_state.pressed(&Action::Jump));
/// assert!(action_state.just_pressed(&Action::Jump));
/// assert!(action_state.released(&Action::Left));
///
/// // Resets just_pressed and just_released
/// let t0 = Instant::now();
/// let t1 = Instant::now();
///
///  action_state.tick(t1, t0);
/// assert!(action_state.pressed(&Action::Jump));
/// assert!(!action_state.just_pressed(&Action::Jump));
///
/// action_state.release(&Action::Jump);
/// assert!(!action_state.pressed(&Action::Jump));
/// assert!(action_state.released(&Action::Jump));
/// assert!(action_state.just_released(&Action::Jump));
///
/// let t2 = Instant::now();
/// action_state.tick(t2, t1);
/// assert!(action_state.released(&Action::Jump));
/// assert!(!action_state.just_released(&Action::Jump));
/// ```
#[derive(Resource, Component, Clone, Debug, PartialEq, Serialize, Deserialize, Reflect)]
pub struct ActionState<A: Actionlike> {
    /// The [`ActionData`] of each action
    action_data: HashMap<A, ActionData>,
}

// The derive does not work unless A: Default,
// so we have to implement it manually
impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            action_data: HashMap::default(),
        }
    }
}

impl<A: Actionlike> ActionState<A> {
    /// Updates the [`ActionState`] based on a vector of [`ActionData`], ordered by [`Actionlike::id`](Actionlike).
    ///
    /// The `action_data` is typically constructed from [`InputMap::which_pressed`](crate::input_map::InputMap),
    /// which reads from the assorted [`ButtonInput`](bevy::input::ButtonInput) resources.
    pub fn update(&mut self, action_data: HashMap<A, ActionData>) {
        for (action, action_datum) in self.action_data.iter_mut() {
            if !action_data.contains_key(action) {
                action_datum.state.release();
            }
        }
        for (action, action_datum) in action_data {
            // Avoid multiple mut borrows, make the compiler happy
            if self.action_data.contains_key(&action) {
                match action_datum.state {
                    ButtonState::JustPressed => self.press(&action),
                    ButtonState::Pressed => self.press(&action),
                    ButtonState::JustReleased => self.release(&action),
                    ButtonState::Released => self.release(&action),
                }

                let current_data = self.action_data.get_mut(&action).unwrap();
                current_data.axis_pair = action_datum.axis_pair;
                current_data.value = action_datum.value;
            } else {
                self.action_data.insert(action, action_datum.clone());
            }
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
    /// use bevy::prelude::Reflect;
    /// use leafwing_input_manager::prelude::*;
    /// use leafwing_input_manager::buttonlike::ButtonState;
    /// use bevy::utils::Instant;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// // Actions start released
    /// assert!(action_state.released(&Action::Jump));
    /// assert!(!action_state.just_released(&Action::Run));
    ///
    /// // Ticking time moves causes buttons just released to no longer be just released
    /// let t0 = Instant::now();
    /// let t1 = Instant::now();
    ///
    /// action_state.tick(t1, t0);
    /// assert!(action_state.released(&Action::Jump));
    /// assert!(!action_state.just_released(&Action::Jump));
    ///
    /// action_state.press(&Action::Jump);
    /// assert!(action_state.just_pressed(&Action::Jump));
    ///
    /// // Ticking time moves causes buttons just pressed to no longer be just pressed
    /// let t2 = Instant::now();
    ///
    /// action_state.tick(t2, t1);
    /// assert!(action_state.pressed(&Action::Jump));
    /// assert!(!action_state.just_pressed(&Action::Jump));
    /// ```
    pub fn tick(&mut self, _current_instant: Instant, _previous_instant: Instant) {
        // Advanced the ButtonState
        self.action_data.values_mut().for_each(|ad| ad.state.tick());

        // Advance the Timings if the feature is enabled
        #[cfg(feature = "timing")]
        self.action_data.values_mut().for_each(|ad| {
            // Durations should not advance while actions are consumed
            if !ad.consumed {
                ad.timing.tick(_current_instant, _previous_instant);
            }
        });
    }

    /// A reference of the [`ActionData`] corresponding to the `action` if triggered.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Caution
    ///
    /// To access the [`ActionData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// # Returns
    ///
    /// - `Some(ActionData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn action_data(&self, action: &A) -> Option<&ActionData> {
        self.action_data.get(action)
    }

    /// A mutable reference of the [`ActionData`] corresponding to the `action` if triggered.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Caution
    ///
    /// - To access the [`ActionData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// - To insert a default [`ActionData`] if it doesn't exist,
    /// use [`action_data_mut_or_default`](Self::action_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(ActionData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn action_data_mut(&mut self, action: &A) -> Option<&mut ActionData> {
        self.action_data.get_mut(action)
    }

    /// A mutable reference of the [`ActionData`] corresponding to the `action`.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`ActionData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    pub fn action_data_mut_or_default(&mut self, action: &A) -> &mut ActionData {
        self.action_data
            .raw_entry_mut()
            .from_key(action)
            .or_insert_with(|| (action.clone(), ActionData::default()))
            .1
    }

    /// Get the value associated with the corresponding `action` if present.
    ///
    /// Different kinds of bindings have different ways of calculating the value:
    ///
    /// - Binary buttons will have a value of `0.0` when the button is not pressed, and a value of
    /// `1.0` when the button is pressed.
    /// - Some axes, such as an analog stick, will have a value in the range `-1.0..=1.0`.
    /// - Some axes, such as a variable trigger, will have a value in the range `0.0..=1.0`.
    /// - Some buttons will also return a value in the range `0.0..=1.0`, such as analog gamepad
    /// triggers which may be tracked as buttons or axes. Examples of these include the Xbox LT/RT
    /// triggers and the Playstation L2/R2 triggers. See also the `axis_inputs` example in the
    /// repository.
    /// - Dual axis inputs will return the magnitude of its [`DualAxisData`] and will be in the range
    /// `0.0..=1.0`.
    /// - Chord inputs will return the value of its first input.
    ///
    /// If multiple inputs trigger the same game action at the same time, the value of each
    /// triggering input will be added together.
    ///
    /// # Warnings
    ///
    /// This value will be 0. if the action has never been pressed or released.
    ///
    /// This value may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_value`](Self::clamped_value) method instead.
    pub fn value(&self, action: &A) -> f32 {
        match self.action_data(action) {
            Some(action_data) => action_data.value,
            None => 0.0,
        }
    }

    /// Get the value associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    ///
    /// # Warning
    ///
    /// This value will be 0. if the action has never been pressed or released.
    pub fn clamped_value(&self, action: &A) -> f32 {
        self.value(action).clamp(-1., 1.)
    }

    /// Get the [`DualAxisData`] from the binding that triggered the corresponding `action`.
    ///
    /// Only certain events such as [`VirtualDPad`][crate::axislike::VirtualDPad] and
    /// [`DualAxis`][crate::axislike::DualAxis] provide an [`DualAxisData`], and this
    /// will return [`None`] for other events.
    ///
    /// Chord inputs will return the [`DualAxisData`] of it's first input.
    ///
    /// If multiple inputs with an axis pair trigger the same game action at the same time, the
    /// value of each axis pair will be added together.
    ///
    /// # Warning
    ///
    /// These values may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_axis_pair`](Self::clamped_axis_pair) method instead.
    pub fn axis_pair(&self, action: &A) -> Option<DualAxisData> {
        let action_data = self.action_data(action)?;
        action_data.axis_pair
    }

    /// Get the [`DualAxisData`] associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    pub fn clamped_axis_pair(&self, action: &A) -> Option<DualAxisData> {
        self.axis_pair(action)
            .map(|pair| DualAxisData::new(pair.x().clamp(-1.0, 1.0), pair.y().clamp(-1.0, 1.0)))
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
    /// use bevy::prelude::Reflect;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum AbilitySlot {
    ///     Slot1,
    ///     Slot2,
    /// }
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let mut ability_slot_state = ActionState::<AbilitySlot>::default();
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// // Extract the state from the ability slot
    /// let slot_1_state = ability_slot_state.action_data(&AbilitySlot::Slot1);
    ///
    /// // And transfer it to the actual ability that we care about
    /// // without losing timing information
    /// if let Some(state) = slot_1_state {
    ///    action_state.set_action_data(Action::Run, state.clone());
    /// }
    /// ```
    #[inline]
    pub fn set_action_data(&mut self, action: A, data: ActionData) {
        self.action_data.insert(action, data);
    }

    /// Press the `action`
    ///
    /// No initial instant or reasons why the button was pressed will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn press(&mut self, action: &A) {
        let action_data = self.action_data_mut_or_default(action);

        // Consumed actions cannot be pressed until they are released
        if action_data.consumed {
            return;
        }

        #[cfg(feature = "timing")]
        if action_data.state.released() {
            action_data.timing.flip();
        }

        action_data.state.press();
    }

    /// Release the `action`
    ///
    /// No initial instant will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn release(&mut self, action: &A) {
        let action_data = self.action_data_mut_or_default(action);

        // Once released, consumed actions can be pressed again
        action_data.consumed = false;

        #[cfg(feature = "timing")]
        if action_data.state.pressed() {
            action_data.timing.flip();
        }

        action_data.state.release();
    }

    /// Releases all actions
    pub fn release_all(&mut self) {
        for action in self.keys() {
            self.release(&action);
        }
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
    /// use bevy::prelude::Reflect;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    /// enum Action {
    ///     Eat,
    ///     Sleep,
    /// }
    ///
    /// let mut action_state = ActionState::<Action>::default();
    ///
    /// action_state.press(&Action::Eat);
    /// assert!(action_state.pressed(&Action::Eat));
    ///
    /// // Consuming actions releases them
    /// action_state.consume(&Action::Eat);
    /// assert!(action_state.released(&Action::Eat));
    ///
    /// // Doesn't work, as the action was consumed
    /// action_state.press(&Action::Eat);
    /// assert!(action_state.released(&Action::Eat));
    ///
    /// // Releasing consumed actions allows them to be pressed again
    /// action_state.release(&Action::Eat);
    /// action_state.press(&Action::Eat);
    /// assert!(action_state.pressed(&Action::Eat));
    /// ```
    #[inline]
    pub fn consume(&mut self, action: &A) {
        let action_data = self.action_data_mut_or_default(action);

        // This is the only difference from action_state.release(&action)
        action_data.consumed = true;
        action_data.state.release();
        #[cfg(feature = "timing")]
        action_data.timing.flip();
    }

    /// Consumes all actions
    #[inline]
    pub fn consume_all(&mut self) {
        for action in self.keys() {
            self.consume(&action);
        }
    }

    /// Is this `action` currently consumed?
    #[inline]
    #[must_use]
    pub fn consumed(&self, action: &A) -> bool {
        matches!(self.action_data(action), Some(action_data) if action_data.consumed)
    }

    /// Disables the `action`
    #[inline]
    pub fn disable(&mut self, action: &A) {
        let action_data = match self.action_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_action_data(action.clone(), ActionData::default());
                self.action_data_mut(action).unwrap()
            }
        };

        action_data.disabled = true;
    }

    /// Disables all actions
    #[inline]
    pub fn disable_all(&mut self) {
        for action in self.keys() {
            self.disable(&action);
        }
    }

    /// Is this `action` currently disabled?
    #[inline]
    #[must_use]
    pub fn disabled(&mut self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.disabled,
            None => false,
        }
    }

    /// Enables the `action`
    #[inline]
    pub fn enable(&mut self, action: &A) {
        let action_data = match self.action_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_action_data(action.clone(), ActionData::default());
                self.action_data_mut(action).unwrap()
            }
        };

        action_data.disabled = false;
    }

    /// Enables all actions
    #[inline]
    pub fn enable_all(&mut self) {
        for action in self.keys() {
            self.enable(&action);
        }
    }

    /// Is this `action` currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self, action: &A) -> bool {
        matches!(self.action_data(action), Some(action_data) if action_data.pressed())
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self, action: &A) -> bool {
        matches!(self.action_data(action), Some(action_data) if action_data.just_pressed())
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    #[inline]
    #[must_use]
    pub fn released(&self, action: &A) -> bool {
        match self.action_data(action) {
            Some(action_data) => action_data.released(),
            None => true,
        }
    }

    /// Was this `action` released since the last time [tick](ActionState::tick) was called?
    #[inline]
    #[must_use]
    pub fn just_released(&self, action: &A) -> bool {
        matches!(self.action_data(action), Some(action_data) if action_data.just_released())
    }

    #[must_use]
    /// Which actions are currently pressed?
    pub fn get_pressed(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.pressed())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions were just pressed?
    pub fn get_just_pressed(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.just_pressed())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions are currently released?
    pub fn get_released(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.released())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions were just released?
    pub fn get_just_released(&self) -> Vec<A> {
        self.action_data
            .iter()
            .filter(|(_action, data)| data.just_released())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    /// The [`Instant`] that the action was last pressed or released
    ///
    ///
    ///
    /// If the action was pressed or released since the last time [`ActionState::tick`] was called
    /// the value will be [`None`].
    /// This ensures that all our actions are assigned a timing and duration
    /// that corresponds exactly to the start of a frame, rather than relying on idiosyncratic timing.
    ///
    /// This will also be [`None`] if the action was never pressed or released.
    #[cfg(feature = "timing")]
    pub fn instant_started(&self, action: &A) -> Option<Instant> {
        let action_data = self.action_data(action)?;
        action_data.timing.instant_started
    }

    /// The [`Duration`] for which the action has been held or released
    ///
    /// This will be [`Duration::ZERO`] if the action was never pressed or released.
    #[cfg(feature = "timing")]
    pub fn current_duration(&self, action: &A) -> Duration {
        self.action_data(action)
            .map(|data| data.timing.current_duration)
            .unwrap_or_default()
    }

    /// The [`Duration`] for which the action was last held or released
    ///
    /// This is a snapshot of the [`ActionState::current_duration`] state at the time
    /// the action was last pressed or released.
    ///
    /// This will be [`Duration::ZERO`] if the action was never pressed or released.
    #[cfg(feature = "timing")]
    pub fn previous_duration(&self, action: &A) -> Duration {
        self.action_data(action)
            .map(|data| data.timing.previous_duration)
            .unwrap_or_default()
    }

    /// Applies an [`ActionDiff`] (usually received over the network) to the [`ActionState`].
    ///
    /// This lets you reconstruct an [`ActionState`] from a stream of [`ActionDiff`]s
    pub fn apply_diff(&mut self, action_diff: &ActionDiff<A>) {
        match action_diff {
            ActionDiff::Pressed { action } => {
                self.press(action);
                // Pressing will initialize the ActionData if it doesn't exist
                self.action_data_mut(action).unwrap().value = 1.;
            }
            ActionDiff::Released { action } => {
                self.release(action);
                // Releasing will initialize the ActionData if it doesn't exist
                let action_data = self.action_data_mut(action).unwrap();
                action_data.value = 0.;
                action_data.axis_pair = None;
            }
            ActionDiff::ValueChanged { action, value } => {
                self.press(action);
                // Pressing will initialize the ActionData if it doesn't exist
                self.action_data_mut(action).unwrap().value = *value;
            }
            ActionDiff::AxisPairChanged { action, axis_pair } => {
                self.press(action);
                let action_data = self.action_data_mut(action).unwrap();
                // Pressing will initialize the ActionData if it doesn't exist
                action_data.axis_pair = Some(DualAxisData::from_xy(*axis_pair));
                action_data.value = axis_pair.length();
            }
        };
    }

    /// Returns an owned list of the [`Actionlike`] keys in this [`ActionState`].
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Vec<A> {
        self.action_data.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate as leafwing_input_manager;
    use crate::action_state::ActionState;
    use crate::clashing_inputs::ClashStrategy;
    use crate::input_map::InputMap;
    use crate::input_mocking::MockInput;
    use crate::input_streams::InputStreams;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;
    use bevy::utils::{Duration, Instant};
    use leafwing_input_manager_macros::Actionlike;

    #[test]
    fn press_lifecycle() {
        let mut app = App::new();
        app.add_plugins(InputPlugin);

        #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, bevy::prelude::Reflect)]
        enum Action {
            Run,
            Jump,
            Hide,
        }

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::KeyR);

        // Starting state
        let input_streams = InputStreams::from_world(&app.world, None);
        action_state.update(input_map.process_actions(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Pressing
        app.press_input(KeyCode::KeyR);
        // Process the input events into Input<KeyCode> data
        app.update();
        let input_streams = InputStreams::from_world(&app.world, None);

        action_state.update(input_map.process_actions(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(&Action::Run));
        assert!(action_state.just_pressed(&Action::Run));
        assert!(!action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(&input_streams, ClashStrategy::PressAll));

        assert!(action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(!action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Releasing
        app.release_input(KeyCode::KeyR);
        app.update();
        let input_streams = InputStreams::from_world(&app.world, None);

        action_state.update(input_map.process_actions(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(action_state.just_released(&Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(&input_streams, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));
    }

    #[test]
    fn update_with_clashes_prioritizing_longest() {
        #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
        enum Action {
            One,
            Two,
            OneAndTwo,
        }

        // Input map
        use bevy::prelude::KeyCode::*;
        let mut input_map = InputMap::default();
        input_map.insert(Action::One, Digit1);
        input_map.insert(Action::Two, Digit2);
        input_map.insert_chord(Action::OneAndTwo, [Digit1, Digit2]);

        let mut app = App::new();
        app.add_plugins(InputPlugin);

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Starting state
        let input_streams = InputStreams::from_world(&app.world, None);
        action_state
            .update(input_map.process_actions(&input_streams, ClashStrategy::PrioritizeLongest));
        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Pressing One
        app.press_input(Digit1);
        app.update();
        let input_streams = InputStreams::from_world(&app.world, None);

        action_state
            .update(input_map.process_actions(&input_streams, ClashStrategy::PrioritizeLongest));

        assert!(action_state.pressed(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state
            .update(input_map.process_actions(&input_streams, ClashStrategy::PrioritizeLongest));

        assert!(action_state.pressed(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Pressing Two
        app.press_input(Digit2);
        app.update();
        let input_streams = InputStreams::from_world(&app.world, None);

        action_state
            .update(input_map.process_actions(&input_streams, ClashStrategy::PrioritizeLongest));

        // Now only the longest OneAndTwo has been pressed,
        // while both One and Two have been released
        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.pressed(&Action::OneAndTwo));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state
            .update(input_map.process_actions(&input_streams, ClashStrategy::PrioritizeLongest));

        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.pressed(&Action::OneAndTwo));
    }
}
