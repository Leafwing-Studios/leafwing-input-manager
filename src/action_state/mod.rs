//! This module contains [`ActionState`] and its supporting methods and impls.

use crate::input_map::UpdatedValue;
use crate::{action_diff::ActionDiff, input_map::UpdatedActions};
use crate::{Actionlike, InputControlKind};

use bevy::prelude::Resource;
use bevy::reflect::Reflect;
#[cfg(feature = "timing")]
use bevy::utils::Duration;
use bevy::utils::{HashMap, Instant};
use bevy::{ecs::component::Component, prelude::ReflectComponent};
use bevy::{
    math::{Vec2, Vec3},
    prelude::ReflectResource,
};
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

    /// Updates the [`ActionState`] based on the provided [`UpdatedActions`].
    ///
    /// The `action_data` is typically constructed from [`InputMap::process_actions`](crate::input_map::InputMap::process_actions),
    /// which reads from the assorted [`ButtonInput`](bevy::input::ButtonInput) resources.
    ///
    /// Actions that are disabled will still be updated: instead, their values will be read as released / zero.
    /// You can see their underlying values by checking their [`ActionData`] directly.
    pub fn update(&mut self, updated_actions: UpdatedActions<A>) {
        for (action, updated_value) in updated_actions.iter() {
            match updated_value {
                UpdatedValue::Button(pressed) => {
                    if *pressed {
                        self.press(action);
                    } else {
                        self.release(action);
                    }
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
                ActionKindData::Button(ref mut button_data) => Some(button_data),
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
                ActionKindData::Axis(ref mut axis_data) => Some(axis_data),
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
                ActionKindData::DualAxis(ref mut dual_axis_data) => Some(dual_axis_data),
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
                ActionKindData::TripleAxis(ref mut triple_axis_data) => Some(triple_axis_data),
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

    /// Get the value associated with the corresponding `action` if present.
    ///
    /// Different kinds of bindings have different ways of calculating the value:
    ///
    /// - Binary buttons will have a value of `0.0` when the button is not pressed, and a value of `1.0` when the button is pressed.
    /// - Some axes, such as an analog stick, will have a value in the range `[-1.0, 1.0]`.
    /// - Some axes, such as a variable trigger, will have a value in the range `[0.0, 1.0]`.
    /// - Some buttons will also return a value in the range `[0.0, 1.0]`, such as analog gamepad triggers which may be tracked as buttons or axes. Examples of these include the Xbox LT/Rtriggers and the Playstation L2/R2 triggers. See also the `axis_inputs` example in the repository.
    /// - Dual axis inputs will return the magnitude of its [`Vec2`] and will be in the range `0.0..=1.0`.
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
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn value(&self, action: &A) -> f32 {
        debug_assert_eq!(action.input_control_kind(), InputControlKind::Axis);

        if self.action_disabled(action) {
            return 0.0;
        }

        match self.axis_data(action) {
            Some(axis_data) => axis_data.value,
            None => 0.0,
        }
    }

    /// Sets the value of the `action` to the provided `value`.
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
    /// Only events that represent dual-axis control provide a [`Vec2`],
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
    /// Only events that represent triple-axis control provide a [`Vec3`],
    /// and this will return [`None`] for other events.
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
        if action_data.state.released() {
            action_data.timing.flip();
        }

        action_data.state.press();
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
        if action_data.state.pressed() {
            action_data.timing.flip();
        }

        action_data.state.release();
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
            ActionDiff::Pressed { action } => {
                self.press(action);
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
    use crate::action_state::ActionState;
    use bevy::prelude::*;
    use leafwing_input_manager_macros::Actionlike;

    #[cfg(feature = "keyboard")]
    #[test]
    fn press_lifecycle() {
        use std::time::{Duration, Instant};

        use crate::input_map::InputMap;
        use crate::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
        use crate::prelude::updating::CentralInputStore;
        use crate::prelude::ClashStrategy;
        use crate::user_input::Buttonlike;
        use bevy::input::InputPlugin;

        let mut app = App::new();
        app.add_plugins((InputPlugin, AccumulatorPlugin, CentralInputStorePlugin));

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
        let input_store = app.world().resource::<CentralInputStore>();
        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PressAll,
        ));

        println!(
            "Initialized button data: {:?}",
            action_state.button_data(&Action::Run)
        );

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Pressing
        KeyCode::KeyR.press(app.world_mut());
        // Process the input events into Input<KeyCode> data
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PressAll,
        ));

        assert!(action_state.pressed(&Action::Run));
        assert!(action_state.just_pressed(&Action::Run));
        assert!(!action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PressAll,
        ));

        assert!(action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(!action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));

        // Releasing
        KeyCode::KeyR.release(app.world_mut());
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PressAll,
        ));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(action_state.just_released(&Action::Run));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PressAll,
        ));

        assert!(!action_state.pressed(&Action::Run));
        assert!(!action_state.just_pressed(&Action::Run));
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Run));
    }

    #[test]
    fn synthetic_press() {
        #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
        enum Action {
            One,
            Two,
        }

        let mut action_state = ActionState::<Action>::default();
        action_state.press(&Action::One);
        dbg!(&action_state);

        assert!(action_state.pressed(&Action::One));
        assert!(action_state.just_pressed(&Action::One));
        assert!(!action_state.released(&Action::One));
        assert!(!action_state.just_released(&Action::One));

        assert!(!action_state.pressed(&Action::Two));
        assert!(!action_state.just_pressed(&Action::Two));
        assert!(action_state.released(&Action::Two));
        assert!(!action_state.just_released(&Action::Two));
    }

    #[cfg(feature = "keyboard")]
    #[test]
    #[ignore = "Clashing inputs for non-buttonlike inputs is broken."]
    fn update_with_clashes_prioritizing_longest() {
        use std::time::{Duration, Instant};

        use crate::input_map::InputMap;
        use crate::plugin::{AccumulatorPlugin, CentralInputStorePlugin};
        use crate::prelude::updating::CentralInputStore;
        use crate::prelude::ClashStrategy;
        use crate::user_input::chord::ButtonlikeChord;
        use crate::user_input::Buttonlike;
        use bevy::input::InputPlugin;
        use bevy::prelude::KeyCode::*;
        use bevy::prelude::*;

        #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
        enum Action {
            One,
            Two,
            OneAndTwo,
        }

        // Input map
        let mut input_map = InputMap::default();
        input_map.insert(Action::One, Digit1);
        input_map.insert(Action::Two, Digit2);
        input_map.insert(Action::OneAndTwo, ButtonlikeChord::new([Digit1, Digit2]));

        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .add_plugins((AccumulatorPlugin, CentralInputStorePlugin));

        // Action state
        let mut action_state = ActionState::<Action>::default();

        // Starting state
        let input_store = app.world().resource::<CentralInputStore>();
        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));
        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Pressing One
        Digit1.press(app.world_mut());
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        assert!(action_state.pressed(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        assert!(action_state.pressed(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.released(&Action::OneAndTwo));

        // Pressing Two
        Digit2.press(app.world_mut());
        app.update();
        let input_store = app.world().resource::<CentralInputStore>();

        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        // Now only the longest OneAndTwo has been pressed,
        // while both One and Two have been released
        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.pressed(&Action::OneAndTwo));

        // Waiting
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        action_state.update(input_map.process_actions(
            &Gamepads::default(),
            input_store,
            ClashStrategy::PrioritizeLongest,
        ));

        assert!(action_state.released(&Action::One));
        assert!(action_state.released(&Action::Two));
        assert!(action_state.pressed(&Action::OneAndTwo));
    }
}
