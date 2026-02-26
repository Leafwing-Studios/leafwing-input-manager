//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::buttonlike::ButtonValue;
use crate::input_map::UpdatedValue;
use crate::{Actionlike, InputControlKind};
use crate::{action_diff::ActionDiff, input_map::UpdatedActions};

use bevy::platform::{collections::HashMap, time::Instant};
use bevy::prelude::Resource;
use bevy::reflect::Reflect;
use bevy::{ecs::component::Component, prelude::ReflectComponent};
use bevy::{
    math::{Vec2, Vec3},
    prelude::ReflectResource,
};
#[cfg(feature = "timing")]
use core::time::Duration;
use serde::{Deserialize, Serialize};

mod action_data;
pub use action_data::*;

/// Stores the canonical input-method-agnostic representation of the inputs received
///
/// Can be used as either a resource or as a [`Component`] on entities that you wish to control directly from player input.
///
/// # Disabling actions
///
/// Actions can be disabled in four different ways, with increasing granularity:
///
/// 1. By disabling updates to all actions using a run condition on [`InputManagerSystem::Update`](crate::plugin::InputManagerSystem::Update).
/// 2. By disabling updates to all actions of type `A` using a run condition on [`TickActionStateSystem::<A>`](crate::plugin::TickActionStateSystem).
/// 3. By setting a specific action state to disabled using [`ActionState::disable`].
/// 4. By disabling a specific action using [`ActionState::disable_action`].
///
/// More general mechanisms of disabling actions will cause specific mechanisms to be ignored.
/// For example, if an entire action state is disabled, then enabling or disabling individual actions will have no effect.
///
/// Actions that are disabled will report as released (but not just released), and their values will be zero.
/// Under the hood, their values are still updated to avoid surprising behavior when re-enabled,
/// but they are not reported to the user using standard methods like [`ActionState::pressed`].
/// To check the underlying values, access their [`ActionData`] directly.
///
/// # Example
///
/// ```rust
/// use bevy::reflect::Reflect;
/// use leafwing_input_manager::prelude::*;
/// use bevy::platform::time::Instant;
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
#[reflect(Resource, Component)]
pub struct ActionState<A: Actionlike> {
    /// Whether or not all of the actions are disabled.
    disabled: bool,
    /// The shared action data for each action
    action_data: HashMap<A, ActionData>,
}

// The derive does not work unless A: Default,
// so we have to implement it manually
impl<A: Actionlike> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            disabled: false,
            action_data: HashMap::default(),
        }
    }
}

impl<A: Actionlike> ActionState<A> {
    /// Returns a reference to the complete [`ActionData`] for all actions.
    #[inline]
    #[must_use]
    pub fn all_action_data(&self) -> &HashMap<A, ActionData> {
        &self.action_data
    }

    /// We are about to enter the `Main` schedule, so we:
    /// - save all the changes applied to `state` into the `fixed_update_state`
    /// - switch to loading the `update_state`
    pub(crate) fn swap_to_update_state(&mut self) {
        for action_datum in self.action_data.values_mut() {
            action_datum.kind_data.swap_to_update_state();
        }
    }

    /// We are about to enter the `FixedMain` schedule, so we:
    /// - save all the changes applied to `state` into the `update_state`
    /// - switch to loading the `fixed_update_state`
    pub(crate) fn swap_to_fixed_update_state(&mut self) {
        for action_datum in self.action_data.values_mut() {
            action_datum.kind_data.swap_to_fixed_update_state();
        }
    }

    /// Function for advanced users to override the `state` from the `update_state`
    pub fn set_update_state_from_state(&mut self) {
        for action_datum in self.action_data.values_mut() {
            action_datum.kind_data.set_update_state_from_state();
        }
    }

    /// Function for advanced users to override the `state` from the `fixed_update_state`
    pub fn set_fixed_update_state_from_state(&mut self) {
        for action_datum in self.action_data.values_mut() {
            action_datum.kind_data.set_fixed_update_state_from_state();
        }
    }

    /// Updates the [`ActionState`] based on the provided [`UpdatedActions`],
    /// typically constructed from [`InputMap::process_actions`](crate::input_map::InputMap::process_actions).
    ///
    /// Actions absent from `updated_actions` but with existing data are reset to zero/released.
    /// Actions that are disabled will still be updated: instead, their values will be read as released / zero.
    /// You can see their underlying values by checking their [`ActionData`] directly.
    pub fn update(&mut self, updated_actions: UpdatedActions<A>) {
        // Reset existing action data absent from this frame's update (e.g. after rollback restore).
        for (action, action_data) in self.action_data.iter_mut() {
            if updated_actions.contains_key(action) {
                continue;
            }
            match action_data.kind_data {
                ActionKindData::Button(_) => {}
                ActionKindData::Axis(ref mut data) => {
                    data.value = 0.0;
                }
                ActionKindData::DualAxis(ref mut data) => {
                    data.pair = Vec2::ZERO;
                }
                ActionKindData::TripleAxis(ref mut data) => {
                    data.triple = Vec3::ZERO;
                }
            }
        }

        for (action, updated_value) in updated_actions.iter() {
            match updated_value {
                UpdatedValue::Button(ButtonValue { pressed, value }) => {
                    if *pressed {
                        self.press(action);
                    } else {
                        self.release(action);
                    }
                    self.set_button_value(action, *value);
                }
                UpdatedValue::Axis(value) => {
                    self.set_value(action, *value);
                }
                UpdatedValue::DualAxis(pair) => {
                    self.set_axis_pair(action, *pair);
                }
                UpdatedValue::TripleAxis(triple) => {
                    self.set_axis_triple(action, *triple);
                }
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
    /// use bevy::platform::time::Instant;
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
        // Advanced the action states
        self.action_data
            .values_mut()
            .for_each(|action_datum| action_datum.tick(_current_instant, _previous_instant));
    }

    /// A reference to the [`ActionData`] corresponding to the `action`.
    #[inline]
    #[must_use]
    pub fn action_data(&self, action: &A) -> Option<&ActionData> {
        self.action_data.get(action)
    }

    /// A mutable reference to the [`ActionData`] corresponding to the `action`.
    ///
    /// To initialize the [`ActionData`] if it has not yet been triggered,
    /// use [`action_data_mut_or_default`](Self::action_data_mut_or_default) method.
    #[inline]
    #[must_use]
    pub fn action_data_mut(&mut self, action: &A) -> Option<&mut ActionData> {
        self.action_data.get_mut(action)
    }

    /// A mutable reference to the [`ActionData`] corresponding to the `action`, initializing it if needed.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`ActionData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    pub fn action_data_mut_or_default(&mut self, action: &A) -> &mut ActionData {
        if self.action_data.contains_key(action) {
            // Safe to unwrap because we just checked
            self.action_data.get_mut(action).unwrap()
        } else {
            self.action_data.insert(
                action.clone(),
                ActionData::from_kind(action.input_control_kind()),
            );
            // Safe to unwrap because we just inserted
            self.action_data_mut(action).unwrap()
        }
    }

    /// A reference of the [`ButtonData`] corresponding to the `action`.
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
        match self.action_data(action) {
            Some(action_data) => match action_data.kind_data {
                ActionKindData::Button(ref button_data) => Some(button_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`ButtonData`] corresponding to the `action`.
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    ///
    /// # Caution
    ///
    /// To access the [`ButtonData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// To insert a default [`ButtonData`] if it doesn't exist,
    /// use [`button_data_mut_or_default`](Self::button_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(ButtonData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn button_data_mut(&mut self, action: &A) -> Option<&mut ButtonData> {
        match self.action_data_mut(action) {
            Some(action_data) => match &mut action_data.kind_data {
                ActionKindData::Button(button_data) => Some(button_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`ButtonData`] corresponding to the `action`, initializing it if needed.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`ButtonData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn button_data_mut_or_default(&mut self, action: &A) -> &mut ButtonData {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let action_data = self.action_data_mut_or_default(action);
        let ActionKindData::Button(ref mut button_data) = action_data.kind_data else {
            panic!("{action:?} is not a Button");
        };
        button_data
    }

    /// A reference of the [`AxisData`] corresponding to the `action`.
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
    #[track_caller]
    pub fn axis_data(&self, action: &A) -> Option<&AxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        match self.action_data(action) {
            Some(action_data) => match action_data.kind_data {
                ActionKindData::Axis(ref axis_data) => Some(axis_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`AxisData`] corresponding to the `action`.
    ///
    /// # Caution
    ///
    /// To insert a default [`AxisData`] if it doesn't exist,
    /// use [`axis_data_mut_or_default`](Self::axis_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(AxisData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    pub fn axis_data_mut(&mut self, action: &A) -> Option<&mut AxisData> {
        match self.action_data_mut(action) {
            Some(action_data) => match &mut action_data.kind_data {
                ActionKindData::Axis(axis_data) => Some(axis_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`AxisData`] corresponding to the `action`, initializing it if needed..
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`AxisData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn axis_data_mut_or_default(&mut self, action: &A) -> &mut AxisData {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        let action_data = self.action_data_mut_or_default(action);
        let ActionKindData::Axis(ref mut axis_data) = action_data.kind_data else {
            panic!("{action:?} is not an Axis");
        };
        axis_data
    }

    /// A reference of the [`DualAxisData`] corresponding to the `action`.
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
    #[track_caller]
    pub fn dual_axis_data(&self, action: &A) -> Option<&DualAxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        match self.action_data(action) {
            Some(action_data) => match action_data.kind_data {
                ActionKindData::DualAxis(ref dual_axis_data) => Some(dual_axis_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`DualAxisData`] corresponding to the `action`.
    ///
    /// # Caution
    ///
    /// To insert a default [`DualAxisData`] if it doesn't exist,
    /// use [`dual_axis_data_mut_or_default`](Self::dual_axis_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(DualAxisData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn dual_axis_data_mut(&mut self, action: &A) -> Option<&mut DualAxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        match self.action_data_mut(action) {
            Some(action_data) => match &mut action_data.kind_data {
                ActionKindData::DualAxis(dual_axis_data) => Some(dual_axis_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`DualAxisData`] corresponding to the `action` initializing it if needed.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`DualAxisData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn dual_axis_data_mut_or_default(&mut self, action: &A) -> &mut DualAxisData {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        let action_data = self.action_data_mut_or_default(action);
        let ActionKindData::DualAxis(ref mut dual_axis_data) = action_data.kind_data else {
            panic!("{action:?} is not a DualAxis");
        };
        dual_axis_data
    }

    /// A reference of the [`TripleAxisData`] corresponding to the `action`.
    ///
    /// # Caution
    ///
    /// To access the [`TripleAxisData`] regardless of whether the `action` has been triggered,
    /// use [`unwrap_or_default`](Option::unwrap_or_default) on the returned [`Option`].
    ///
    /// # Returns
    ///
    /// - `Some(TripleAxisData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn triple_axis_data(&self, action: &A) -> Option<&TripleAxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::TripleAxis);

        match self.action_data(action) {
            Some(action_data) => match action_data.kind_data {
                ActionKindData::TripleAxis(ref triple_axis_data) => Some(triple_axis_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`TripleAxisData`] corresponding to the `action`.
    ///
    /// # Caution
    ///
    /// To insert a default [`TripleAxisData`] if it doesn't exist,
    /// use [`triple_axis_data_mut_or_default`](Self::dual_axis_data_mut_or_default) method.
    ///
    /// # Returns
    ///
    /// - `Some(ButtonData)` if it exists.
    /// - `None` if the `action` has never been triggered (pressed, clicked, etc.).
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn triple_axis_data_mut(&mut self, action: &A) -> Option<&mut TripleAxisData> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::TripleAxis);

        match self.action_data_mut(action) {
            Some(action_data) => match &mut action_data.kind_data {
                ActionKindData::TripleAxis(triple_axis_data) => Some(triple_axis_data),
                _ => None,
            },
            None => None,
        }
    }

    /// A mutable reference of the [`TripleAxisData`] corresponding to the `action` initializing it if needed.
    ///
    /// If the `action` has no data yet (because the `action` has not been triggered),
    /// this method will create and insert a default [`TripleAxisData`] for you,
    /// avoiding potential errors from unwrapping [`None`].
    ///
    /// Generally, it'll be clearer to call `pressed` or so on directly on the [`ActionState`].
    /// However, accessing the raw data directly allows you to examine detailed metadata holistically.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn triple_axis_data_mut_or_default(&mut self, action: &A) -> &mut TripleAxisData {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::TripleAxis);

        let action_data = self.action_data_mut_or_default(action);
        let ActionKindData::TripleAxis(ref mut triple_axis_data) = action_data.kind_data else {
            panic!("{action:?} is not a TripleAxis");
        };
        triple_axis_data
    }

    /// Get the value associated with the corresponding buttonlike `action` if present.
    ///
    /// # Warnings
    ///
    /// This value may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_button_value`](Self::clamped_button_value) method instead.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn button_value(&self, action: &A) -> f32 {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        if self.action_disabled(action) {
            return 0.0;
        }

        let action_data = self.button_data(action);
        action_data.map_or(0.0, |action_data| action_data.value)
    }

    /// Sets the value of the buttonlike `action` to the provided `value`.
    /// A threshold of `0.02` must be overcome for the button to count as "pressed"
    /// This is used to account for a semi-frequent issue with analog inputs
    /// (e.g., gamepad triggers) reporting very small non-zero values when
    /// not physically pressed, due to sensor imprecision.
    ///
    /// Also updates the state of the button based on the `value`:
    /// - If `value > 0.02`, the button will be pressed.
    /// - If `value <= 0.0`, the button will be released.
    #[track_caller]
    pub fn set_button_value(&mut self, action: &A, value: f32) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);
        const BUTTON_PRESS_THRESHOLD: f32 = 0.02;

        let button_data = self.button_data_mut_or_default(action);
        button_data.value = value;

        if value > BUTTON_PRESS_THRESHOLD {
            #[cfg(feature = "timing")]
            if button_data.state.released() {
                button_data.timing.flip();
            }

            button_data.state.press();
        } else {
            #[cfg(feature = "timing")]
            if button_data.state.pressed() {
                button_data.timing.flip();
            }

            button_data.state.release();
        }
    }

    /// Get the value associated with the corresponding `action`, clamped to `[0.0, 1.0]`.
    ///
    /// # Warning
    ///
    /// This value will be 0. by default,
    /// even if the action is not a buttonlike action.
    pub fn clamped_button_value(&self, action: &A) -> f32 {
        self.button_value(action).clamp(0., 1.)
    }

    /// Get the value associated with the corresponding axislike `action` if present.
    ///
    /// # Warnings
    ///
    /// This value may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_value`](Self::clamped_value) method instead.
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn value(&self, action: &A) -> f32 {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        if self.action_disabled(action) {
            return 0.0;
        }

        let action_data = self.axis_data(action);
        action_data.map_or(0.0, |action_data| action_data.value)
    }

    /// Sets the value of the axislike `action` to the provided `value`.
    #[track_caller]
    pub fn set_value(&mut self, action: &A, value: f32) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        let axis_data = self.axis_data_mut_or_default(action);
        axis_data.value = value;
    }

    /// Get the value associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    ///
    /// # Warning
    ///
    /// This value will be 0. by default,
    /// even if the action is not an axislike action.
    pub fn clamped_value(&self, action: &A) -> f32 {
        self.value(action).clamp(-1., 1.)
    }

    /// Get the [`Vec2`] from the binding that triggered the corresponding `action`.
    ///
    /// Only messages that represent dual-axis control provide a [`Vec2`],
    /// and this will return [`None`] for other messages.
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
    #[must_use]
    #[track_caller]
    pub fn axis_pair(&self, action: &A) -> Vec2 {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        if self.action_disabled(action) {
            return Vec2::ZERO;
        }

        let action_data = self.dual_axis_data(action);
        action_data.map_or(Vec2::ZERO, |action_data| action_data.pair)
    }

    /// Sets the [`Vec2`] of the `action` to the provided `pair`.
    #[track_caller]
    pub fn set_axis_pair(&mut self, action: &A, pair: Vec2) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::DualAxis);

        let dual_axis_data = self.dual_axis_data_mut_or_default(action);
        dual_axis_data.pair = pair;
    }

    /// Get the [`Vec2`] associated with the corresponding `action`, clamped to `[-1.0, 1.0]`.
    ///  
    /// # Warning
    ///
    /// This value will be [`Vec2::ZERO`] by default,
    /// even if the action is not a dual-axislike action.
    pub fn clamped_axis_pair(&self, action: &A) -> Vec2 {
        let pair = self.axis_pair(action);
        pair.clamp(Vec2::NEG_ONE, Vec2::ONE)
    }

    /// Get the [`Vec3`] from the binding that triggered the corresponding `action`.
    ///
    /// Only messages that represent triple-axis control provide a [`Vec3`],
    /// and this will return [`None`] for other messages.
    ///
    /// If multiple inputs with an axis triple trigger the same game action at the same time, the
    /// value of each axis triple will be added together.
    ///
    /// # Warning
    ///
    /// This value will be [`Vec3::ZERO`] by default,
    /// even if the action is not a triple-axislike action.
    ///
    /// These values may not be bounded as you might expect.
    /// Consider clamping this to account for multiple triggering inputs,
    /// typically using the [`clamped_axis_triple`](Self::clamped_axis_triple) method instead.
    #[must_use]
    #[track_caller]
    pub fn axis_triple(&self, action: &A) -> Vec3 {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::TripleAxis);

        if self.action_disabled(action) {
            return Vec3::ZERO;
        }

        let action_data = self.triple_axis_data(action);
        action_data.map_or(Vec3::ZERO, |action_data| action_data.triple)
    }

    /// Sets the [`Vec2`] of the `action` to the provided `pair`.
    #[track_caller]
    pub fn set_axis_triple(&mut self, action: &A, triple: Vec3) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::TripleAxis);

        let triple_axis_data = self.triple_axis_data_mut_or_default(action);
        triple_axis_data.triple = triple;
    }

    /// Get the [`Vec3`] associated with the corresponding `action`, clamped to the cube of values bounded by -1 and 1 on all axes.
    ///
    /// # Warning
    ///
    /// This value will be [`Vec3::ZERO`] by default,
    /// even if the action is not a dual-axislike action.
    pub fn clamped_axis_triple(&self, action: &A) -> Vec3 {
        let triple = self.axis_triple(action);
        triple.clamp(Vec3::NEG_ONE, Vec3::ONE)
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
    #[track_caller]
    pub fn set_button_data(&mut self, action: A, data: ButtonData) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let button_data = self.button_data_mut_or_default(&action);
        *button_data = data;
    }

    /// Press the `action`
    ///
    /// No initial instant or reasons why the button was pressed will be recorded.
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    #[track_caller]
    pub fn press(&mut self, action: &A) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let action_data = self.button_data_mut_or_default(action);

        #[cfg(feature = "timing")]
        if action_data.update_state.released() {
            action_data.timing.flip();
        }

        action_data.state.press();
        action_data.value = 1.0;
    }

    /// Release the `action`
    ///
    /// No initial instant will be recorded.
    /// Instead, this is set through [`ActionState::tick()`]
    #[inline]
    pub fn release(&mut self, action: &A) {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let action_data = self.button_data_mut_or_default(action);

        #[cfg(feature = "timing")]
        if action_data.update_state.pressed() {
            action_data.timing.flip();
        }

        action_data.state.release();
        action_data.value = 0.0;
    }

    /// Resets an action to its default state.
    ///
    /// Buttons will be released, and axes will be set to 0.
    pub fn reset(&mut self, action: &A) {
        match action.input_control_kind() {
            InputControlKind::Button => self.release(action),
            InputControlKind::Axis => {
                self.set_value(action, 0.0);
            }
            InputControlKind::DualAxis => {
                self.set_axis_pair(action, Vec2::ZERO);
            }
            InputControlKind::TripleAxis => {
                self.set_axis_triple(action, Vec3::ZERO);
            }
        }
    }

    /// Releases all [`Buttonlike`](crate::user_input::Buttonlike) actions,
    /// sets all [`Axislike`](crate::user_input::Axislike) actions to 0,
    /// sets all [`DualAxislike`](crate::user_input::DualAxislike) actions to [`Vec2::ZERO`],
    /// and sets all [`TripleAxislike`](crate::user_input::TripleAxislike) actions to [`Vec3::ZERO`].
    pub fn reset_all(&mut self) {
        // Collect out to avoid angering the borrow checker
        let all_actions = self.action_data.keys().cloned().collect::<Vec<A>>();
        for action in all_actions.into_iter() {
            self.reset(&action);
        }
    }

    /// Is the entire [`ActionState`] currently disabled?
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// Is this `action` currently disabled?
    #[inline]
    #[must_use]
    pub fn action_disabled(&self, action: &A) -> bool {
        if self.disabled {
            return true;
        }

        match self.action_data(action) {
            Some(action_data) => action_data.disabled,
            None => false,
        }
    }

    /// Disables the entire [`ActionState`].
    ///
    /// All values will be reset to their default state.
    #[inline]
    pub fn disable(&mut self) {
        self.disabled = true;
        self.reset_all();
    }

    /// Disables the `action`.
    ///
    /// The action's value will be reset to its default state.
    #[inline]
    pub fn disable_action(&mut self, action: &A) {
        let action_data = self.action_data_mut_or_default(action);

        action_data.disabled = true;
        self.reset(action);
    }

    /// Disables all actions
    #[inline]
    pub fn disable_all_actions(&mut self) {
        for action in self.keys() {
            self.disable_action(&action);
        }
    }

    /// Enables the entire [`ActionState`]
    #[inline]
    pub fn enable(&mut self) {
        self.disabled = false;
    }

    /// Enables the `action`
    #[inline]
    pub fn enable_action(&mut self, action: &A) {
        let action_data = self.action_data_mut_or_default(action);

        action_data.disabled = false;
    }

    /// Enables all actions
    #[inline]
    pub fn enable_all_actions(&mut self) {
        for action in self.keys() {
            self.enable_action(&action);
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
    #[track_caller]
    pub fn pressed(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        if self.action_disabled(action) {
            return false;
        }

        match self.button_data(action) {
            Some(button_data) => button_data.pressed(),
            None => false,
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
    #[track_caller]
    pub fn just_pressed(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        if self.action_disabled(action) {
            return false;
        }

        match self.button_data(action) {
            Some(button_data) => button_data.just_pressed(),
            None => false,
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
    #[track_caller]
    pub fn released(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        if self.action_disabled(action) {
            return true;
        }

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
    #[track_caller]
    pub fn just_released(&self, action: &A) -> bool {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        if self.action_disabled(action) {
            return false;
        }

        match self.button_data(action) {
            Some(button_data) => button_data.just_released(),
            None => false,
        }
    }

    #[must_use]
    /// Which actions are currently pressed?
    pub fn get_pressed(&self) -> Vec<A> {
        let all_actions = self.action_data.keys().cloned();

        all_actions
            .into_iter()
            .filter(|action| action.input_control_kind() == InputControlKind::Button)
            .filter(|action| self.pressed(action))
            .collect()
    }

    #[must_use]
    /// Which actions were just pressed?
    pub fn get_just_pressed(&self) -> Vec<A> {
        let all_actions = self.action_data.keys().cloned();

        all_actions
            .into_iter()
            .filter(|action| action.input_control_kind() == InputControlKind::Button)
            .filter(|action| self.just_pressed(action))
            .collect()
    }

    #[must_use]
    /// Which actions are currently released?
    pub fn get_released(&self) -> Vec<A> {
        let all_actions = self.action_data.keys().cloned();

        all_actions
            .into_iter()
            .filter(|action| action.input_control_kind() == InputControlKind::Button)
            .filter(|action| self.released(action))
            .collect()
    }

    #[must_use]
    /// Which actions were just released?
    pub fn get_just_released(&self) -> Vec<A> {
        let all_actions = self.action_data.keys().cloned();

        all_actions
            .into_iter()
            .filter(|action| action.input_control_kind() == InputControlKind::Button)
            .filter(|action| self.just_released(action))
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
    #[must_use]
    #[track_caller]
    pub fn instant_started(&self, action: &A) -> Option<Instant> {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Button);

        let button_data = self.button_data(action)?;
        button_data.timing.instant_started
    }

    /// The [`Duration`] for which the action has been held or released
    ///
    /// This will be [`Duration::ZERO`] if the action was never pressed or released.
    #[cfg(feature = "timing")]
    #[must_use]
    #[track_caller]
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
    #[must_use]
    #[track_caller]
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
            ActionDiff::Pressed { action, value } => {
                self.set_button_value(action, *value);
            }
            ActionDiff::Released { action } => {
                self.release(action);
            }
            ActionDiff::AxisChanged { action, value } => {
                self.set_value(action, *value);
            }
            ActionDiff::DualAxisChanged { action, axis_pair } => {
                self.set_axis_pair(action, *axis_pair);
            }
            ActionDiff::TripleAxisChanged {
                action,
                axis_triple,
            } => {
                self.set_axis_triple(action, *axis_triple);
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
    use crate::action_diff::ActionDiff;
    use crate::action_state::{
        ActionData, ActionKindData, ActionState, AxisData, ButtonData, DualAxisData,
    };
    use crate::buttonlike::{ButtonState, ButtonValue};
    use crate::input_map::UpdatedActions;
    #[cfg(any(feature = "gamepad", feature = "keyboard"))]
    use crate::prelude::{ButtonlikeChord, InputMap};
    #[cfg(feature = "gamepad")]
    use bevy::input::gamepad::GamepadButton;
    #[cfg(feature = "keyboard")]
    use bevy::input::keyboard::KeyCode;
    use bevy::platform::collections::HashMap;
    use bevy::prelude::*;
    use leafwing_input_manager_macros::Actionlike;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    enum TestAction {
        Trigger,
        Run,
        Jump,
        Hide,
        One,
        Two,
        OneAndTwo,
        #[actionlike(Axis)]
        Axis,
        #[actionlike(DualAxis)]
        DualAxis,
        #[actionlike(TripleAxis)]
        TripleAxis,
    }

    #[cfg(any(feature = "keyboard", feature = "gamepad"))]
    struct TestContext {
        pub app: App,
        pub input_map: InputMap<TestAction>,
    }

    #[cfg(any(feature = "keyboard", feature = "gamepad"))]
    impl TestContext {
        pub fn new() -> Self {
            use bevy::input::InputPlugin;

            use crate::plugin::InputManagerPlugin;

            let mut app = App::new();
            app.add_plugins((
                MinimalPlugins,
                InputPlugin,
                InputManagerPlugin::<TestAction>::default(),
            ));

            let mut input_map = InputMap::default();
            #[cfg(feature = "gamepad")]
            input_map.insert(TestAction::Trigger, GamepadButton::RightTrigger);

            #[cfg(feature = "keyboard")]
            {
                input_map.insert(TestAction::One, KeyCode::Digit1);
                input_map.insert(TestAction::Two, KeyCode::Digit2);
                input_map.insert(
                    TestAction::OneAndTwo,
                    ButtonlikeChord::new([KeyCode::Digit1, KeyCode::Digit2]),
                );
                input_map.insert(TestAction::Run, KeyCode::KeyR);
            }

            app.insert_resource(input_map.clone())
                .init_resource::<ActionState<TestAction>>();

            app.update();

            Self { app, input_map }
        }

        #[cfg(feature = "gamepad")]
        pub fn send_gamepad_connection_event(&mut self, gamepad: Option<Entity>) -> Entity {
            use bevy::input::gamepad::{GamepadConnection, GamepadConnectionEvent};

            let gamepad = gamepad.unwrap_or_else(|| self.app.world_mut().spawn_empty().id());
            self.app
                .world_mut()
                .resource_mut::<Messages<GamepadConnectionEvent>>()
                .write(GamepadConnectionEvent::new(
                    gamepad,
                    GamepadConnection::Connected {
                        name: "TestController".to_string(),
                        vendor_id: None,
                        product_id: None,
                    },
                ));
            gamepad
        }

        pub fn update(&mut self) {
            self.app.update();
        }
    }

    #[test]
    fn action_state_default_state() {
        let action_state = ActionState::<TestAction>::default();

        assert!(!action_state.disabled);
        assert_eq!(action_state.action_data, HashMap::default());
    }

    #[test]
    fn action_state_all_action_data() {
        let action_state = ActionState::<TestAction>::default();

        assert_eq!(action_state.all_action_data(), &action_state.action_data);
    }

    #[test]
    fn action_state_button() {
        let mut action_state = ActionState::<TestAction>::default();

        assert!(!action_state.pressed(&TestAction::Jump));
        assert_eq!(action_state.button_value(&TestAction::Jump), 0.0);

        let mut updated_actions = UpdatedActions::<TestAction>::default();
        updated_actions.0.insert(
            TestAction::Jump,
            crate::input_map::UpdatedValue::Button(ButtonValue {
                pressed: true,
                value: 0.5,
            }),
        );

        action_state.update(updated_actions);

        assert!(action_state.pressed(&TestAction::Jump));
        assert_eq!(action_state.button_value(&TestAction::Jump), 0.5);
    }

    #[test]
    fn action_state_axis() {
        let mut action_state = ActionState::<TestAction>::default();

        assert_eq!(action_state.value(&TestAction::Axis), 0.0);

        let mut updated_actions = UpdatedActions::<TestAction>::default();
        updated_actions
            .0
            .insert(TestAction::Axis, crate::input_map::UpdatedValue::Axis(0.5));

        action_state.update(updated_actions);

        assert_eq!(action_state.value(&TestAction::Axis), 0.5);
    }

    #[test]
    fn action_state_dual_axis() {
        let mut action_state = ActionState::<TestAction>::default();

        assert_eq!(
            action_state.axis_pair(&TestAction::DualAxis),
            Vec2::new(0.0, 0.0)
        );

        let mut updated_actions = UpdatedActions::<TestAction>::default();
        updated_actions.0.insert(
            TestAction::DualAxis,
            crate::input_map::UpdatedValue::DualAxis(Vec2::new(0.5, 0.5)),
        );

        action_state.update(updated_actions);

        assert_eq!(
            action_state.axis_pair(&TestAction::DualAxis),
            Vec2::new(0.5, 0.5)
        );
    }

    #[test]
    fn action_state_triple_axis() {
        let mut action_state = ActionState::<TestAction>::default();

        assert_eq!(
            action_state.axis_triple(&TestAction::TripleAxis),
            Vec3::new(0.0, 0.0, 0.0)
        );

        let mut updated_actions = UpdatedActions::<TestAction>::default();
        updated_actions.0.insert(
            TestAction::TripleAxis,
            crate::input_map::UpdatedValue::TripleAxis(Vec3::new(0.5, 0.5, 0.5)),
        );

        action_state.update(updated_actions);

        assert_eq!(
            action_state.axis_triple(&TestAction::TripleAxis),
            Vec3::new(0.5, 0.5, 0.5)
        );
    }

    #[cfg(feature = "keyboard")]
    #[test]
    fn press_lifecycle() {
        use std::time::{Duration, Instant};

        use crate::prelude::Buttonlike;
        use crate::prelude::ClashStrategy;
        use crate::prelude::updating::CentralInputStore;

        let ctx = TestContext::new();
        let mut app = ctx.app;
        let input_map = ctx.input_map;

        // Action state
        let mut action_state = ActionState::<TestAction>::default();
        println!(
            "Default button data: {:?}",
            action_state.button_data(&TestAction::Run)
        );

        // Starting state
        let input_store = app.world().resource::<CentralInputStore>();
        action_state.update(input_map.process_actions(None, input_store, ClashStrategy::PressAll));

        println!(
            "Initialized button data: {:?}",
            action_state.button_data(&TestAction::Run)
        );

        assert!(!action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));
        assert!(action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));

        // Pressing
        KeyCode::KeyR.press(app.world_mut());
        // Process the input messages into Input<KeyCode> data
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(None, input_store, ClashStrategy::PressAll));

        assert!(action_state.pressed(&TestAction::Run));
        assert!(action_state.just_pressed(&TestAction::Run));
        assert!(!action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(None, input_store, ClashStrategy::PressAll));

        assert!(action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));
        assert!(!action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));

        // Releasing
        KeyCode::KeyR.release(app.world_mut());
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(None, input_store, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));
        assert!(action_state.released(&TestAction::Run));
        assert!(action_state.just_released(&TestAction::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(None, input_store, ClashStrategy::PressAll));

        assert!(!action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));
        assert!(action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));
    }

    #[test]
    fn synthetic_press() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.press(&TestAction::One);
        dbg!(&action_state);

        assert!(action_state.pressed(&TestAction::One));
        assert!(action_state.just_pressed(&TestAction::One));
        assert!(!action_state.released(&TestAction::One));
        assert!(!action_state.just_released(&TestAction::One));

        assert!(!action_state.pressed(&TestAction::Two));
        assert!(!action_state.just_pressed(&TestAction::Two));
        assert!(action_state.released(&TestAction::Two));
        assert!(!action_state.just_released(&TestAction::Two));
    }

    #[cfg(feature = "keyboard")]
    #[test]
    #[ignore = "Clashing inputs for non-buttonlike inputs is broken."]
    fn update_with_clashes_prioritizing_longest() {
        use std::time::{Duration, Instant};

        use crate::prelude::ClashStrategy;
        use crate::prelude::updating::CentralInputStore;
        use crate::user_input::Buttonlike;
        use bevy::prelude::KeyCode::*;

        let ctx = TestContext::new();
        let mut app = ctx.app;
        let input_map = ctx.input_map;

        // Action state
        let mut action_state = ActionState::<TestAction>::default();

        // Starting state
        let input_store = app.world().resource::<CentralInputStore>();
        action_state.update(input_map.process_actions(
            None,
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));
        assert!(action_state.released(&TestAction::One));
        assert!(action_state.released(&TestAction::Two));
        assert!(action_state.released(&TestAction::OneAndTwo));

        // Pressing One
        Digit1.press(app.world_mut());
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(
            None,
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        assert!(action_state.pressed(&TestAction::One));
        assert!(action_state.released(&TestAction::Two));
        assert!(action_state.released(&TestAction::OneAndTwo));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(
            None,
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        assert!(action_state.pressed(&TestAction::One));
        assert!(action_state.released(&TestAction::Two));
        assert!(action_state.released(&TestAction::OneAndTwo));

        // Pressing Two
        Digit2.press(app.world_mut());
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(
            None,
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        // Now only the longest OneAndTwo has been pressed,
        // while both One and Two have been released
        assert!(action_state.released(&TestAction::One));
        assert!(action_state.released(&TestAction::Two));
        assert!(action_state.pressed(&TestAction::OneAndTwo));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(
            None,
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        assert!(action_state.released(&TestAction::One));
        assert!(action_state.released(&TestAction::Two));
        assert!(action_state.pressed(&TestAction::OneAndTwo));
    }

    #[test]
    fn test_set_update_state_from_state() {
        let mut action_state = ActionState::<TestAction>::default();

        // Initial state
        assert!(action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));
        assert!(!action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));

        // Update the state manually
        action_state.action_data.insert(
            TestAction::Run,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Button(ButtonData {
                    state: ButtonState::Pressed,
                    update_state: ButtonState::Pressed,
                    fixed_update_state: ButtonState::Pressed,
                    value: 1.0,
                    update_value: 1.0,
                    fixed_update_value: 1.0,
                    #[cfg(feature = "timing")]
                    timing: Default::default(),
                }),
            },
        );
        action_state.set_update_state_from_state();

        // Check the state
        assert!(action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));
        assert!(!action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));
    }

    #[test]
    fn test_set_fixed_update_state_from_state() {
        let mut action_state = ActionState::<TestAction>::default();

        // Initial state
        assert!(action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));
        assert!(!action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));

        // Update the state manually
        action_state.action_data.insert(
            TestAction::Run,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Button(ButtonData {
                    state: ButtonState::Pressed,
                    update_state: ButtonState::Pressed,
                    fixed_update_state: ButtonState::Pressed,
                    value: 1.0,
                    update_value: 1.0,
                    fixed_update_value: 1.0,
                    #[cfg(feature = "timing")]
                    timing: Default::default(),
                }),
            },
        );
        action_state.set_fixed_update_state_from_state();

        assert!(action_state.pressed(&TestAction::Run));
        assert!(!action_state.just_pressed(&TestAction::Run));
        assert!(!action_state.released(&TestAction::Run));
        assert!(!action_state.just_released(&TestAction::Run));
    }

    #[test]
    fn test_button_data_for_button_action_without_data() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.remove(&TestAction::Run);
        assert!(action_state.button_data(&TestAction::Run).is_none());
    }

    #[test]
    fn test_button_data_for_non_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Axis,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Axis(AxisData {
                    value: 0.5,
                    update_value: 0.5,
                    fixed_update_value: 0.5,
                }),
            },
        );
        assert!(action_state.button_data(&TestAction::Axis).is_none());
    }

    #[test]
    fn test_button_data_mut_for_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Run,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Button(ButtonData {
                    state: ButtonState::Released,
                    update_state: ButtonState::Released,
                    fixed_update_state: ButtonState::Released,
                    value: 0.0,
                    update_value: 0.0,
                    fixed_update_value: 0.0,
                    #[cfg(feature = "timing")]
                    timing: Default::default(),
                }),
            },
        );

        assert!(action_state.button_data_mut(&TestAction::Run).is_some());
    }

    #[test]
    fn test_button_data_mut_for_button_action_without_data() {
        let mut action_state = ActionState::<TestAction>::default();

        assert!(action_state.button_data_mut(&TestAction::Run).is_none());
    }

    #[test]
    fn test_button_data_mut_for_non_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Axis,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Axis(AxisData {
                    value: 0.5,
                    update_value: 0.5,
                    fixed_update_value: 0.5,
                }),
            },
        );
        assert!(action_state.button_data_mut(&TestAction::Axis).is_none());
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: Axis\n right: Button")]
    fn test_button_data_mut_or_default_for_non_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        let _ = action_state.button_data_mut_or_default(&TestAction::Axis);
    }

    #[test]
    fn test_axis_data_for_axis_action_without_data() {
        let action_state = ActionState::<TestAction>::default();
        assert!(action_state.axis_data(&TestAction::Axis).is_none());
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: Button\n right: Axis")]
    fn test_axis_data_for_non_axis_action() {
        let action_state = ActionState::<TestAction>::default();
        assert!(action_state.axis_data(&TestAction::Run).is_none());
    }

    #[test]
    fn test_axis_data_mut_for_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Axis,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Axis(AxisData {
                    value: 0.5,
                    update_value: 0.5,
                    fixed_update_value: 0.5,
                }),
            },
        );
        assert!(action_state.axis_data_mut(&TestAction::Axis).is_some());
    }

    #[test]
    fn test_axis_data_mut_for_axis_action_without_data() {
        let mut action_state = ActionState::<TestAction>::default();
        assert!(action_state.axis_data_mut(&TestAction::Axis).is_none());
    }

    #[test]
    fn test_axis_data_mut_for_non_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Run,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::Button(ButtonData {
                    state: ButtonState::Released,
                    update_state: ButtonState::Released,
                    fixed_update_state: ButtonState::Released,
                    value: 0.0,
                    update_value: 0.0,
                    fixed_update_value: 0.0,
                    #[cfg(feature = "timing")]
                    timing: Default::default(),
                }),
            },
        );
        assert!(action_state.axis_data_mut(&TestAction::Run).is_none());
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: Button\n right: Axis")]
    fn test_axis_data_mut_or_default_for_non_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        let _ = action_state.axis_data_mut_or_default(&TestAction::Run);
    }

    #[test]
    fn test_dual_axis_data_for_dual_axis_action_without_data() {
        let action_state = ActionState::<TestAction>::default();
        assert!(action_state.dual_axis_data(&TestAction::DualAxis).is_none());
    }

    #[test]
    fn test_dual_axis_data_mut_for_dual_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::DualAxis,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::DualAxis(DualAxisData {
                    pair: Vec2::new(0.5, 0.5),
                    update_pair: Vec2::new(0.5, 0.5),
                    fixed_update_pair: Vec2::new(0.5, 0.5),
                }),
            },
        );
        assert!(
            action_state
                .dual_axis_data_mut(&TestAction::DualAxis)
                .is_some()
        );
    }

    #[test]
    fn test_dual_axis_data_mut_for_dual_axis_action_without_data() {
        let mut action_state = ActionState::<TestAction>::default();
        assert!(
            action_state
                .dual_axis_data_mut(&TestAction::DualAxis)
                .is_none()
        );
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: Button\n right: DualAxis")]
    fn test_dual_axis_data_mut_for_non_dual_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        assert!(action_state.dual_axis_data_mut(&TestAction::Run).is_none());
    }

    #[test]
    fn test_triple_axis_data_for_triple_axis_action_without_data() {
        let action_state = ActionState::<TestAction>::default();
        assert!(
            action_state
                .triple_axis_data(&TestAction::TripleAxis)
                .is_none()
        );
    }

    #[test]
    fn test_triple_axis_data_mut_for_triple_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::TripleAxis,
            ActionData {
                disabled: false,
                kind_data: ActionKindData::TripleAxis(crate::action_state::TripleAxisData {
                    triple: Vec3::new(0.5, 0.5, 0.5),
                    update_triple: Vec3::new(0.5, 0.5, 0.5),
                    fixed_update_triple: Vec3::new(0.5, 0.5, 0.5),
                }),
            },
        );
        assert!(
            action_state
                .triple_axis_data_mut(&TestAction::TripleAxis)
                .is_some()
        );
    }

    #[test]
    fn test_button_value_for_disabled_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Run,
            ActionData {
                disabled: true,
                kind_data: ActionKindData::Button(ButtonData {
                    state: ButtonState::Pressed,
                    update_state: ButtonState::Pressed,
                    fixed_update_state: ButtonState::Pressed,
                    value: 1.0,
                    update_value: 1.0,
                    fixed_update_value: 1.0,
                    #[cfg(feature = "timing")]
                    timing: Default::default(),
                }),
            },
        );
        action_state.disable_action(&TestAction::Run);
        assert_eq!(action_state.button_value(&TestAction::Run), 0.0);
    }

    #[test]
    fn test_clamped_button_value_less_than_zero() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, -0.5);
        assert_eq!(action_state.clamped_button_value(&TestAction::Run), 0.0);
    }

    #[test]
    fn test_clamped_button_value_greater_than_zero() {
        let mut action_state: ActionState<TestAction> = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, 1.5);
        assert_eq!(action_state.clamped_button_value(&TestAction::Run), 1.0);
    }

    #[test]
    fn test_value_for_disabled_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.action_data.insert(
            TestAction::Axis,
            ActionData {
                disabled: true,
                kind_data: ActionKindData::Axis(AxisData {
                    value: 1.0,
                    update_value: 1.0,
                    fixed_update_value: 1.0,
                }),
            },
        );
        action_state.disable_action(&TestAction::Axis);
        assert_eq!(action_state.value(&TestAction::Axis), 0.0);
    }

    #[test]
    fn test_clamped_value_less_than_negative_one() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_value(&TestAction::Axis, -2.0);
        assert_eq!(action_state.clamped_value(&TestAction::Axis), -1.0);
    }

    #[test]
    fn test_clamped_value_greater_than_one() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_value(&TestAction::Axis, 2.0);
        assert_eq!(action_state.clamped_value(&TestAction::Axis), 1.0);
    }

    #[test]
    fn test_axis_pair_for_disabled_dual_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable_action(&TestAction::DualAxis);
        assert_eq!(action_state.axis_pair(&TestAction::DualAxis), Vec2::ZERO);
    }

    #[test]
    fn test_clamped_axis_pair_greater_than_vec2_one() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_pair(&TestAction::DualAxis, Vec2::new(2.0, 2.0));
        assert_eq!(
            action_state.clamped_axis_pair(&TestAction::DualAxis),
            Vec2::ONE
        );
    }

    #[test]
    fn test_clamped_axis_pair_less_than_vec2_negative_one() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_pair(&TestAction::DualAxis, Vec2::new(-2.0, -2.0));
        assert_eq!(
            action_state.clamped_axis_pair(&TestAction::DualAxis),
            Vec2::NEG_ONE
        );
    }

    #[test]
    fn test_axis_triple_for_disabled_triple_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable_action(&TestAction::TripleAxis);
        assert_eq!(
            action_state.axis_triple(&TestAction::TripleAxis),
            Vec3::ZERO
        );
    }

    #[test]
    fn test_clamped_axis_triple_greater_than_vec3_one() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_triple(&TestAction::TripleAxis, Vec3::new(2.0, 2.0, 2.0));
        assert_eq!(
            action_state.clamped_axis_triple(&TestAction::TripleAxis),
            Vec3::ONE
        );
    }

    #[test]
    fn test_clamped_axis_triple_less_than_vec3_negative_one() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_triple(&TestAction::TripleAxis, Vec3::new(-2.0, -2.0, -2.0));
        assert_eq!(
            action_state.clamped_axis_triple(&TestAction::TripleAxis),
            Vec3::NEG_ONE
        );
    }

    #[test]
    fn test_set_button_data_for_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        let button_data = ButtonData {
            state: ButtonState::Pressed,
            update_state: ButtonState::Pressed,
            fixed_update_state: ButtonState::Pressed,
            value: 1.0,
            update_value: 1.0,
            fixed_update_value: 1.0,
            #[cfg(feature = "timing")]
            timing: Default::default(),
        };
        action_state.set_button_data(TestAction::Run, button_data);

        let returned_data = action_state.button_data(&TestAction::Run).unwrap();
        assert_eq!(returned_data.state, ButtonState::Pressed);
        assert_eq!(returned_data.value, 1.0);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed\n  left: Axis\n right: Button")]
    fn test_set_button_data_for_non_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        let button_data = ButtonData {
            state: ButtonState::Pressed,
            update_state: ButtonState::Pressed,
            fixed_update_state: ButtonState::Pressed,
            value: 1.0,
            update_value: 1.0,
            fixed_update_value: 1.0,
            #[cfg(feature = "timing")]
            timing: Default::default(),
        };
        action_state.set_button_data(TestAction::Axis, button_data);
    }

    #[test]
    fn test_reset_button_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, 1.0);
        action_state.reset(&TestAction::Run);

        assert!(!action_state.pressed(&TestAction::Run));
        assert_eq!(action_state.button_value(&TestAction::Run), 0.0);
    }

    #[test]
    fn test_reset_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_value(&TestAction::Axis, 1.0);
        action_state.reset(&TestAction::Axis);

        assert_eq!(action_state.value(&TestAction::Axis), 0.0);
    }

    #[test]
    fn test_reset_dual_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_pair(&TestAction::DualAxis, Vec2::ONE);
        action_state.reset(&TestAction::DualAxis);

        assert_eq!(action_state.axis_pair(&TestAction::DualAxis), Vec2::ZERO);
    }

    #[test]
    fn test_reset_triple_axis_action() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_triple(&TestAction::TripleAxis, Vec3::ONE);
        action_state.reset(&TestAction::TripleAxis);

        assert_eq!(
            action_state.axis_triple(&TestAction::TripleAxis),
            Vec3::ZERO
        );
    }

    #[test]
    fn test_action_disabled_when_action_state_disabled() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable();

        assert!(action_state.action_disabled(&TestAction::Run));
    }

    #[test]
    fn test_action_disabled_when_action_state_not_disabled() {
        let mut action_state = ActionState::<TestAction>::default();
        assert!(!action_state.action_disabled(&TestAction::Run));

        action_state.disable_action(&TestAction::Run);
        assert!(action_state.action_disabled(&TestAction::Run));
    }

    #[test]
    fn test_disable() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable();

        assert!(action_state.disabled);
    }

    #[test]
    fn test_enable() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable();
        action_state.enable();
        assert!(!action_state.disabled);
    }

    #[test]
    fn test_just_pressed_when_action_disabled() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable_action(&TestAction::Run);

        assert!(!action_state.just_pressed(&TestAction::Run));
    }

    #[test]
    fn test_released_when_action_disabled() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable_action(&TestAction::Run);

        assert!(action_state.released(&TestAction::Run));
    }

    #[test]
    fn test_just_released_when_action_disabled() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable_action(&TestAction::Run);

        assert!(!action_state.just_released(&TestAction::Run));
    }

    #[test]
    fn test_pressed_when_action_disabled() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.disable();

        assert!(!action_state.pressed(&TestAction::Run));
    }

    #[test]
    fn apply_diff_triple_axis() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_axis_triple(&TestAction::TripleAxis, Vec3::new(1.0, 1.0, 1.0));

        let diff = ActionDiff::TripleAxisChanged {
            action: TestAction::TripleAxis,
            axis_triple: Vec3::new(0.5, 1.0, 1.5),
        };
        action_state.apply_diff(&diff);

        assert_eq!(
            action_state.axis_triple(&TestAction::TripleAxis),
            Vec3::new(0.5, 1.0, 1.5)
        );
    }

    #[test]
    fn test_get_pressed() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, 1.0);

        let pressed_actions: Vec<TestAction> = action_state.get_pressed();
        assert_eq!(pressed_actions.len(), 1);
        assert!(pressed_actions.contains(&TestAction::Run));
    }

    #[test]
    fn test_get_just_pressed() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, 1.0);

        let just_pressed_actions: Vec<TestAction> = action_state.get_just_pressed();
        assert_eq!(just_pressed_actions.len(), 1);
        assert!(just_pressed_actions.contains(&TestAction::Run));
    }

    #[test]
    fn test_get_released() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, 0.0);

        let released_actions: Vec<TestAction> = action_state.get_released();
        assert_eq!(released_actions.len(), 1);
        assert!(released_actions.contains(&TestAction::Run));
    }

    #[test]
    fn test_get_just_released() {
        let mut action_state = ActionState::<TestAction>::default();
        action_state.set_button_value(&TestAction::Run, 1.0);
        action_state.set_button_value(&TestAction::Run, 0.0);

        let just_released_actions: Vec<TestAction> = action_state.get_just_released();
        assert_eq!(just_released_actions.len(), 1);
        assert!(just_released_actions.contains(&TestAction::Run));
    }

    #[cfg(feature = "gamepad")]
    #[test]
    fn test_triggerlikes() {
        use crate::prelude::Buttonlike;

        let mut ctx = TestContext::new();
        let _gamepad = ctx.send_gamepad_connection_event(None);
        ctx.update();

        let action_state = ctx.app.world().resource::<ActionState<TestAction>>();
        assert!(action_state.released(&TestAction::Trigger));

        // App Context
        let mut ctx = TestContext::new();
        let gamepad = ctx.send_gamepad_connection_event(None);
        ctx.update();

        GamepadButton::RightTrigger.set_value_as_gamepad(ctx.app.world_mut(), 0.8, Some(gamepad));
        ctx.update();

        let action_state = ctx.app.world().resource::<ActionState<TestAction>>();
        assert!(action_state.pressed(&TestAction::Trigger));
        assert!(action_state.just_pressed(&TestAction::Trigger));
        assert_eq!(action_state.button_value(&TestAction::Trigger), 0.8);
    }
}
