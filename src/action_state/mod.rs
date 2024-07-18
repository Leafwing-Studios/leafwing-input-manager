//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::{action_diff::ActionDiff, input_map::UpdatedActions};
use crate::{Actionlike, InputControlKind};

use bevy::ecs::component::Component;
use bevy::math::Vec2;
use bevy::prelude::Resource;
use bevy::reflect::Reflect;
#[cfg(feature = "timing")]
use bevy::utils::Duration;
use bevy::utils::{HashMap, Instant};
use serde::{Deserialize, Serialize};

mod action_data;
pub use action_data::*;

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
    /// The [`ButtonData`] of each action
    button_data: HashMap<A, ButtonData>,
    /// The [`AxisData`] of each action
    axis_data: HashMap<A, AxisData>,
    /// The [`Vec2`] of each action
    dual_axis_data: HashMap<A, DualAxisData>,
}

// The derive does not work unless A: Default,
// so we have to implement it manually
impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            button_data: HashMap::default(),
            axis_data: HashMap::default(),
            dual_axis_data: HashMap::default(),
        }
    }
}

impl<A: Actionlike> ActionState<A> {
    /// Returns a reference to the complete [`ButtonData`] for all actions.
    #[inline]
    #[must_use]
    pub fn all_button_data(&self) -> &HashMap<A, ButtonData> {
        &self.button_data
    }

    /// Returns a reference to the complete [`AxisData`] for all actions.
    #[inline]
    #[must_use]
    pub fn all_axis_data(&self) -> &HashMap<A, AxisData> {
        &self.axis_data
    }

    /// Returns a reference to the complete [`DualAxisData`] for all actions.
    #[inline]
    #[must_use]
    pub fn all_dual_axis_data(&self) -> &HashMap<A, DualAxisData> {
        &self.dual_axis_data
    }

    /// We are about to enter the `Main` schedule, so we:
    /// - save all the changes applied to `state` into the `fixed_update_state`
    /// - switch to loading the `update_state`
    pub(crate) fn swap_to_update_state(&mut self) {
        for (_action, action_datum) in self.button_data.iter_mut() {
            // save the changes applied to `state` into `fixed_update_state`
            action_datum.fixed_update_state = action_datum.state;
            // switch to loading the `update_state` into `state`
            action_datum.state = action_datum.update_state;
        }

        for (_action, action_datum) in self.axis_data.iter_mut() {
            // save the changes applied to `state` into `fixed_update_state`
            action_datum.fixed_update_value = action_datum.value;
            // switch to loading the `update_state` into `state`
            action_datum.value = action_datum.update_value;
        }

        for (_action, action_datum) in self.dual_axis_data.iter_mut() {
            // save the changes applied to `state` into `fixed_update_state`
            action_datum.fixed_update_pair = action_datum.pair;
            // switch to loading the `update_state` into `state`
            action_datum.pair = action_datum.update_pair;
        }
    }

    /// We are about to enter the `FixedMain` schedule, so we:
    /// - save all the changes applied to `state` into the `update_state`
    /// - switch to loading the `fixed_update_state`
    pub(crate) fn swap_to_fixed_update_state(&mut self) {
        for (_action, action_datum) in self.button_data.iter_mut() {
            // save the changes applied to `state` into `update_state`
            action_datum.update_state = action_datum.state;
            // switch to loading the `fixed_update_state` into `state`
            action_datum.state = action_datum.fixed_update_state;
        }

        for (_action, action_datum) in self.axis_data.iter_mut() {
            // save the changes applied to `state` into `update_state`
            action_datum.update_value = action_datum.value;
            // switch to loading the `fixed_update_state` into `state`
            action_datum.value = action_datum.fixed_update_value;
        }

        for (_action, action_datum) in self.dual_axis_data.iter_mut() {
            // save the changes applied to `state` into `update_state`
            action_datum.update_pair = action_datum.pair;
            // switch to loading the `fixed_update_state` into `state`
            action_datum.pair = action_datum.fixed_update_pair;
        }
    }

    /// Updates the [`ActionState`] based on the provided [`UpdatedActions`].
    ///
    /// The `action_data` is typically constructed from [`InputMap::process_actions`](crate::input_map::InputMap::process_actions),
    /// which reads from the assorted [`ButtonInput`](bevy::input::ButtonInput) resources.
    pub fn update(&mut self, updated_actions: UpdatedActions<A>) {
        for (action, button_datum) in updated_actions.button_actions {
            if self.button_data.contains_key(&action) {
                match button_datum {
                    true => self.press(&action),
                    false => self.release(&action),
                }
            } else {
                match button_datum {
                    true => self.button_data.insert(action, ButtonData::JUST_PRESSED),
                    // Buttons should start in a released state,
                    // and should not be just pressed or just released.
                    // This behavior helps avoid unexpected behavior with on-key-release actions
                    // at the start of the game.
                    false => self.button_data.insert(action, ButtonData::RELEASED),
                };
            }
        }

        for (action, axis_datum) in updated_actions.axis_actions.into_iter() {
            if self.axis_data.contains_key(&action) {
                self.axis_data.get_mut(&action).unwrap().value = axis_datum;
            } else {
                self.axis_data.insert(
                    action,
                    AxisData {
                        value: axis_datum,
                        ..Default::default()
                    },
                );
            }
        }

        for (action, dual_axis_datum) in updated_actions.dual_axis_actions.into_iter() {
            if self.dual_axis_data.contains_key(&action) {
                self.dual_axis_data.get_mut(&action).unwrap().pair = dual_axis_datum;
            } else {
                self.dual_axis_data.insert(
                    action,
                    DualAxisData {
                        pair: dual_axis_datum,
                        ..Default::default()
                    },
                );
            }
        }
    }

    /// Advances the time for all actions,
    /// transitioning them from `just_pressed` to `pressed`, and `just_released` to `released`.
    ///
    /// If the `timing` feature flag is enabled, the underlying timing and action data will be advanced according to the `current_instant`.
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
        self.button_data.values_mut().for_each(|ad| ad.state.tick());

        // Advance the Timings if the feature is enabled
        #[cfg(feature = "timing")]
        self.button_data.values_mut().for_each(|ad| {
            // Durations should not advance while actions are consumed
            if !ad.consumed {
                ad.timing.tick(_current_instant, _previous_instant);
            }
        });
    }

    /// A reference of the [`ButtonData`] corresponding to the `action` if triggered.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Caution
    ///
    /// To access the [`ButtonData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// # Returns
    ///
    /// - `Some(ButtonData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn button_data(&self, action: &A) -> Option<&ButtonData> {
        self.button_data.get(action)
    }

    /// A mutable reference of the [`ButtonData`] corresponding to the `action` if triggered.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Caution
    ///
    /// - To access the [`ButtonData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// - To insert a default [`ButtonData`] if it doesn't exist,
    /// use [`button_data_mut_or_default`](Self::button_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(ButtonData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn button_data_mut(&mut self, action: &A) -> Option<&mut ButtonData> {
        self.button_data.get_mut(action)
    }

    /// A mutable reference of the [`ButtonData`] corresponding to the `action`.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`ButtonData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    pub fn button_data_mut_or_default(&mut self, action: &A) -> &mut ButtonData {
        self.button_data
            .raw_entry_mut()
            .from_key(action)
            .or_insert_with(|| (action.clone(), ButtonData::default()))
            .1
    }

    /// A reference of the [`AxisData`] corresponding to the `action` if triggered.
    ///
    /// # Caution
    ///
    /// To access the [`AxisData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// # Returns
    ///
    /// - `Some(AxisData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn axis_data(&self, action: &A) -> Option<&AxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        self.axis_data.get(action)
    }

    /// A mutable reference of the [`AxisData`] corresponding to the `action` if triggered.
    ///
    /// # Caution
    ///
    /// - To access the [`AxisData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// - To insert a default [`AxisData`] if it doesn't exist,
    /// use [`axis_data_mut_or_default`](Self::axis_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(AxisData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn axis_data_mut(&mut self, action: &A) -> Option<&mut AxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        self.axis_data.get_mut(action)
    }

    /// A mutable reference of the [`AxisData`] corresponding to the `action`.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`AxisData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    pub fn axis_data_mut_or_default(&mut self, action: &A) -> &mut AxisData {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        self.axis_data
            .raw_entry_mut()
            .from_key(action)
            .or_insert_with(|| (action.clone(), AxisData::default()))
            .1
    }

    /// A reference of the [`DualAxisData`] corresponding to the `action` if triggered.
    ///
    /// # Caution
    ///
    /// To access the [`DualAxisData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// # Returns
    ///
    /// - `Some(DualAxisData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn dual_axis_data(&self, action: &A) -> Option<&DualAxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        self.dual_axis_data.get(action)
    }

    /// A mutable reference of the [`DualAxisData`] corresponding to the `action` if triggered.
    ///
    /// # Caution
    ///
    /// - To access the [`DualAxisData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// - To insert a default [`ButtonData`] if it doesn't exist,
    /// use [`dual_axis_data_mut_or_default`](Self::dual_axis_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(ButtonData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn dual_axis_data_mut(&mut self, action: &A) -> Option<&mut DualAxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        self.dual_axis_data.get_mut(action)
    }

    /// A mutable reference of the [`ButtonData`] corresponding to the `action`.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`DualAxisData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    pub fn dual_axis_data_mut_or_default(&mut self, action: &A) -> &mut DualAxisData {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        self.dual_axis_data
            .raw_entry_mut()
            .from_key(action)
            .or_insert_with(|| (action.clone(), DualAxisData::default()))
            .1
    }

    /// Get the value associated with the corresponding `action` if present.
    ///
    /// Different kinds of bindings have different ways of calculating the value:
    ///
    /// - Binary buttons will have a value of `0.0` when the button is not pressed, and a value of
    /// `1.0` when the button is pressed.
    /// - Some axes, such as an analog stick, will have a value in the range `[-1.0, 1.0]`.
    /// - Some axes, such as a variable trigger, will have a value in the range `[0.0, 1.0]`.
    /// - Some buttons will also return a value in the range `[0.0, 1.0]`, such as analog gamepad
    /// triggers which may be tracked as buttons or axes. Examples of these include the Xbox LT/RT
    /// triggers and the Playstation L2/R2 triggers. See also the `axis_inputs` example in the
    /// repository.
    /// - Dual axis inputs will return the magnitude of its [`Vec2`] and will be in the range
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
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        match self.axis_data(action) {
            Some(axis_data) => axis_data.value,
            None => 0.0,
        }
    }

    /// Get the value associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    ///
    /// # Warning
    ///
    /// This value will be 0. by default,
    /// even if the action is not a axislike action.
    pub fn clamped_value(&self, action: &A) -> f32 {
        self.value(action).clamp(-1., 1.)
    }

    /// Get the [`Vec2`] from the binding that triggered the corresponding `action`.
    ///
    /// Only events that represent dual-axis control provide an [`Vec2`],
    /// and this will return [`None`] for other events.
    ///
    /// If multiple inputs with an axis pair trigger the same game action at the same time, the
    /// value of each axis pair will be added together.
    ///
    /// # Warning
    ///
    /// This value will be [`Vec2::ZERO`] by default,
    /// even if the action is not a dual-axislike action.
    ///
    /// These values may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_axis_pair`](Self::clamped_axis_pair) method instead.
    pub fn axis_pair(&self, action: &A) -> Vec2 {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        let action_data = self.dual_axis_data(action);
        action_data.map_or(Vec2::ZERO, |action_data| action_data.pair)
    }

    /// Get the [`Vec2`] associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    ///  
    /// # Warning
    ///
    /// This value will be [`Vec2::ZERO`] by default,
    /// even if the action is not a dual-axislike action.
    pub fn clamped_axis_pair(&self, action: &A) -> Vec2 {
        let pair = self.axis_pair(action);
        Vec2::new(pair.x.clamp(-1.0, 1.0), pair.y.clamp(-1.0, 1.0))
    }

    /// Manually sets the [`ButtonData`] of the corresponding `action`
    ///
    /// You should almost always use more direct methods, as they are simpler and less error-prone.
    ///
    /// However, this method can be useful for testing,
    /// or when transferring [`ButtonData`] between action states.
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
    /// let slot_1_state = ability_slot_state.button_data(&AbilitySlot::Slot1);
    ///
    /// // And transfer it to the actual ability that we care about
    /// // without losing timing information
    /// if let Some(state) = slot_1_state {
    ///    action_state.set_button_data(Action::Run, state.clone());
    /// }
    /// ```
    #[inline]
    pub fn set_button_data(&mut self, action: A, data: ButtonData) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        self.button_data.insert(action, data);
    }

    /// Press the `action`
    ///
    /// No initial instant or reasons why the button was pressed will be recorded
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn press(&mut self, action: &A) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let action_data = self.button_data_mut_or_default(action);

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
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let action_data = self.button_data_mut_or_default(action);

        // Once released, consumed actions can be pressed again
        action_data.consumed = false;

        #[cfg(feature = "timing")]
        if action_data.state.pressed() {
            action_data.timing.flip();
        }

        action_data.state.release();
    }

    /// Releases all [`Buttonlike`](crate::user_input::Buttonlike) actions
    pub fn release_all(&mut self) {
        // Collect out to avoid angering the borrow checker
        let buttonlike_actions = self.button_data.keys().cloned().collect::<Vec<A>>();
        for action in buttonlike_actions {
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
        let action_data = self.button_data_mut_or_default(action);

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
        matches!(self.button_data(action), Some(action_data) if action_data.consumed)
    }

    /// Disables the `action`
    #[inline]
    pub fn disable(&mut self, action: &A) {
        let action_data = match self.button_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_button_data(action.clone(), ButtonData::default());
                self.button_data_mut(action).unwrap()
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
        match self.button_data(action) {
            Some(action_data) => action_data.disabled,
            None => false,
        }
    }

    /// Enables the `action`
    #[inline]
    pub fn enable(&mut self, action: &A) {
        let action_data = match self.button_data_mut(action) {
            Some(action_data) => action_data,
            None => {
                self.set_button_data(action.clone(), ButtonData::default());
                self.button_data_mut(action).unwrap()
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
    ///
    /// # Warning
    ///
    /// This value will be `false` by default,
    /// even if the action is not a buttonlike action.
    #[inline]
    #[must_use]
    pub fn pressed(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        match self.button_data(action) {
            Some(button_data) => button_data.pressed(),
            None => true,
        }
    }

    /// Was this `action` pressed since the last time [tick](ActionState::tick) was called?
    ///
    /// # Warning
    ///
    /// This value will be `false` by default,
    /// even if the action is not a buttonlike action.
    #[inline]
    #[must_use]
    pub fn just_pressed(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        match self.button_data(action) {
            Some(button_data) => button_data.just_pressed(),
            None => true,
        }
    }

    /// Is this `action` currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    ///
    /// # Warning
    ///
    /// This value will be `true` by default,
    /// even if the action is not a buttonlike action.
    #[inline]
    #[must_use]
    pub fn released(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        match self.button_data(action) {
            Some(button_data) => button_data.released(),
            None => true,
        }
    }

    /// Was this `action` released since the last time [tick](ActionState::tick) was called?
    ///
    /// # Warning
    ///
    /// This value will be `false` by default,
    /// even if the action is not a buttonlike action.
    #[inline]
    #[must_use]
    pub fn just_released(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        match self.button_data(action) {
            Some(button_data) => button_data.just_released(),
            None => false,
        }
    }

    #[must_use]
    /// Which actions are currently pressed?
    pub fn get_pressed(&self) -> Vec<A> {
        self.button_data
            .iter()
            .filter(|(_action, data)| data.pressed())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions were just pressed?
    pub fn get_just_pressed(&self) -> Vec<A> {
        self.button_data
            .iter()
            .filter(|(_action, data)| data.just_pressed())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions are currently released?
    pub fn get_released(&self) -> Vec<A> {
        self.button_data
            .iter()
            .filter(|(_action, data)| data.released())
            .map(|(action, _data)| action.clone())
            .collect()
    }

    #[must_use]
    /// Which actions were just released?
    pub fn get_just_released(&self) -> Vec<A> {
        self.button_data
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
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let button_data = self.button_data(action)?;
        button_data.timing.instant_started
    }

    /// The [`Duration`] for which the action has been held or released
    ///
    /// This will be [`Duration::ZERO`] if the action was never pressed or released.
    #[cfg(feature = "timing")]
    pub fn current_duration(&self, action: &A) -> Duration {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        self.button_data(action)
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
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        self.button_data(action)
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
            }
            ActionDiff::Released { action } => {
                self.release(action);
            }
            ActionDiff::AxisChanged { action, value } => {
                let axis_data = self.axis_data_mut(action).unwrap();
                // Pressing will initialize the ActionData if it doesn't exist
                axis_data.value = *value;
            }
            ActionDiff::DualAxisChanged { action, axis_pair } => {
                let axis_data = self.dual_axis_data_mut(action).unwrap();
                // Pressing will initialize the ActionData if it doesn't exist
                axis_data.pair = *axis_pair;
            }
        };
    }

    /// Returns an owned list of the [`Actionlike`] keys in this [`ActionState`].
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Vec<A> {
        self.button_data.keys().cloned().collect()
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
    use crate::plugin::AccumulatorPlugin;
    use crate::prelude::ButtonlikeChord;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;
    use bevy::utils::{Duration, Instant};
    use leafwing_input_manager_macros::Actionlike;

    #[test]
    fn press_lifecycle() {
        let mut app = App::new();
        app.add_plugins(InputPlugin).add_plugins(AccumulatorPlugin);

        #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, bevy::prelude::Reflect)]
        enum Action {
            Run,
            Jump,
            Hide,
        }

        // Action state
        let mut action_state = ActionState::<Action>::default();
        println!(
            "Default button data: {:?}",
            action_state.button_data(&Action::Run)
        );

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::KeyR);

        // Starting state
        let input_streams = InputStreams::from_world(app.world(), None);
        action_state.update(input_map.process_actions(&input_streams, ClashStrategy::PressAll));

        println!(
            "Initialized button data: {:?}",
            action_state.button_data(&Action::Run)
        );

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Pressing
        app.press_input(KeyCode::KeyR);
        // Process the input events into Input<KeyCode> data
        app.update();
        let input_streams = InputStreams::from_world(app.world(), None);

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
        let input_streams = InputStreams::from_world(app.world(), None);

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
    #[ignore = "Clashing inputs for non-buttonlike inputs is broken."]
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
        input_map.insert(Action::OneAndTwo, ButtonlikeChord::new([Digit1, Digit2]));

        let mut app = App::new();
        app.add_plugins(InputPlugin).add_plugins(AccumulatorPlugin);

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Starting state
        let input_streams = InputStreams::from_world(app.world(), None);
        action_state
            .update(input_map.process_actions(&input_streams, ClashStrategy::PrioritizeLongest));
        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Pressing One
        app.press_input(Digit1);
        app.update();
        let input_streams = InputStreams::from_world(app.world(), None);

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
        let input_streams = InputStreams::from_world(app.world(), None);

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
