//! Cooldowns tick down until actions are ready to be used.

use crate::Actionlike;

use bevy::ecs::prelude::Component;
use bevy::utils::Duration;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// The time until each action of type `A` can be used again.
///
/// Each action may be associated with a [`Cooldown`].
/// If it is not, it always be treated as being ready.
///
/// This is typically paired with an [`ActionState`](crate::action_state::ActionState):
/// if the action state is just-pressed (or another triggering condition is met),
/// and the cooldown is ready, then perform the action and trigger the cooldown.
///
/// This type is included as part of the [`InputManagerBundle`](crate::InputManagerBundle),
/// but can also be used as a resource for singleton game objects.
///
///     
/// ```rust
/// use leafwing_input_manager::prelude::*;
/// use bevy::utils::Duration;
///
/// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Debug)]
/// enum Action {
///     Run,
///     Jump,
/// }
///
/// let mut action_state = ActionState::<Action>::default();
/// let mut cooldowns = Cooldowns::new([(Cooldown::from_secs(1.), Action::Jump)]);
///
/// action_state.press(Action::Jump);
///
/// if action_state.just_pressed(Action::Jump) && cooldowns.ready(Action::Jump) {
///    // Actually do the jumping thing here
///    // Remember to actually begin the cooldown if you jumped!
///    cooldowns.trigger(Action::Jump);
/// }
///
/// // We just jumped, so the cooldown isn't ready yet
/// assert!(!cooldowns.ready(Action::Jump));
/// ```
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Cooldowns<A: Actionlike> {
    /// The [`Cooldown`] of each action
    ///
    /// The position in this vector corresponds to [`Actionlike::index`].
    /// If [`None`], the action can always be used
    cooldowns: Vec<Option<Cooldown>>,
    /// A shared cooldown between all actions of type `A`.
    ///
    /// No action of type `A` will be ready unless this is ready.
    /// Whenever any cooldown for an action of type `A` is triggered,
    /// this global cooldown is triggered.
    pub global_cooldown: Option<Cooldown>,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Default for Cooldowns<A> {
    /// By default, cooldowns are not set.
    fn default() -> Self {
        Cooldowns {
            cooldowns: A::variants().map(|_| None).collect(),
            global_cooldown: None,
            _phantom: PhantomData::default(),
        }
    }
}

impl<A: Actionlike> Cooldowns<A> {
    /// Creates a new [`Cooldowns`] from an iterator of `(cooldown, action)` pairs
    ///
    /// If a [`Cooldown`] is not provided for an action, that action will be treated as if its cooldown is always available.
    ///
    /// To create an empty [`Cooldowns`] struct, use the [`Default::default`] method instead.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::cooldown::{Cooldown, Cooldowns};
    /// use leafwing_input_manager::Actionlike;
    /// use bevy::input::keyboard::KeyCode;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    ///     Shoot,
    ///     Dash,
    /// }
    ///
    /// let input_map = Cooldowns::new([
    ///     (Cooldown::from_secs(0.1), Action::Shoot),
    ///     (Cooldown::from_secs(1.0), Action::Dash),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(cooldown_action_pairs: impl IntoIterator<Item = (Cooldown, A)>) -> Self {
        let mut cooldowns = Cooldowns::default();
        for (cooldown, action) in cooldown_action_pairs.into_iter() {
            cooldowns.set(cooldown, action);
        }
        cooldowns
    }

    /// Triggers the cooldown of the `action` if it is available to be used.
    ///
    /// This should always be paired with [`Cooldowns::ready`],
    /// to check if the action can be used before triggering its cooldown.
    #[inline]
    pub fn trigger(&mut self, action: A) {
        if let Some(cooldown) = self.cooldown_mut(action) {
            cooldown.trigger();
        }

        if let Some(global_cooldown) = self.global_cooldown.as_mut() {
            global_cooldown.trigger();
        }
    }

    /// Can the corresponding `action` be used?
    ///
    /// This will be `true` if the underlying [`Cooldown::ready`] call is true,
    /// or if no cooldown is stored for this action.
    #[inline]
    #[must_use]
    pub fn ready(&self, action: A) -> bool {
        if !self.gcd_ready() {
            return false;
        }

        if let Some(cooldown) = self.cooldown(action) {
            cooldown.ready()
        } else {
            true
        }
    }

    /// Has the global cooldown for actions of type `A` expired?
    ///
    /// Returns `true` if no GCD is set.
    #[inline]
    #[must_use]
    pub fn gcd_ready(&self) -> bool {
        if let Some(global_cooldown) = self.global_cooldown.as_ref() {
            global_cooldown.ready()
        } else {
            true
        }
    }

    /// Advances each underlying [`Cooldown`] according to the elapsed `delta_time`.
    pub fn tick(&mut self, delta_time: Duration) {
        self.iter_mut().for_each(|cd| cd.tick(delta_time));

        if let Some(global_cooldown) = self.global_cooldown.as_mut() {
            global_cooldown.tick(delta_time);
        }
    }

    /// Returns an iterator of references to the underlying non-[`None`] [`Cooldown`]s
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Cooldown> {
        self.cooldowns.iter().flatten()
    }

    /// Returns an iterator of mutable references to the underlying non-[`None`] [`Cooldown`]s
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Cooldown> {
        self.cooldowns.iter_mut().flatten()
    }

    /// The cooldown associated with the specified `action`, if any.
    #[inline]
    #[must_use]
    pub fn cooldown(&self, action: A) -> &Option<Cooldown> {
        &self.cooldowns[action.index()]
    }

    /// A mutable reference to the cooldown associated with the specified `action`, if any.
    #[inline]
    #[must_use]
    pub fn cooldown_mut(&mut self, action: A) -> &mut Option<Cooldown> {
        &mut self.cooldowns[action.index()]
    }

    /// Set a cooldown for the specified `action`.
    ///
    /// If a cooldown already existed, it will be replaced by a new cooldown with the specified duration.
    #[inline]
    pub fn set(&mut self, cooldown: Cooldown, action: A) {
        *self.cooldown_mut(action) = Some(cooldown);
    }

    /// Remove any cooldown for the specified `action`.
    #[inline]
    pub fn remove(&mut self, action: A) {
        *self.cooldown_mut(action) = None;
    }
}

/// A timer-like struct that records the amount of time until an action is available to be used again.
///
/// Cooldowns are typically stored in an [`ActionState`](crate::action_state::ActionState), associated with an action that is to be
/// cooldown-regulated.
///
/// When initialized, cooldowns are always fully available.
///
/// ```rust
/// use bevy::utils::Duration;
/// use leafwing_input_manager::cooldown::Cooldown;
///
/// let mut cooldown = Cooldown::new(Duration::from_secs(3));
/// assert_eq!(cooldown.remaining(), Duration::ZERO);
///
/// cooldown.trigger();
/// assert_eq!(cooldown.remaining(), Duration::from_secs(3));
///
/// cooldown.tick(Duration::from_secs(1));
/// assert!(!cooldown.ready());
///
/// cooldown.tick(Duration::from_secs(5));
/// let triggered = cooldown.trigger();
/// assert!(triggered);
///
/// cooldown.refresh();
/// assert!(cooldown.ready());
/// ```
///
/// ## Charges
///
/// Cooldowns can have more than one **charge**.
/// When a cooldown is triggered, one charge will be consumed.
/// When the cooldown expires, one charge will be added.
/// Cooldowns are only ready if at least one charge is available.
///
/// ```rust
/// use leafwing_input_manager::cooldown::Cooldown;
/// use bevy::utils::Duration;
///
/// let mut cooldown = Cooldown::from_secs(1.).with_charges(3);
/// assert_eq!(cooldown.charges(), 3);
/// assert_eq!(cooldown.max_charges(), 3);
/// assert!(cooldown.ready());
///
/// cooldown.trigger();
/// assert_eq!(cooldown.charges(), 2);
///
/// cooldown.trigger();
/// cooldown.trigger();
/// assert_eq!(cooldown.charges(), 0);
/// assert!(!cooldown.ready());
///
/// cooldown.tick(Duration::from_secs(1));
/// assert_eq!(cooldown.charges(), 1);
/// assert!(cooldown.ready());
///
/// cooldown.tick(Duration::from_secs(9001));
/// assert_eq!(cooldown.charges(), cooldown.max_charges());
/// ```
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Cooldown {
    max_time: Duration,
    elapsed_time: Duration,
    charges: u8,
    max_charges: u8,
}

impl Cooldown {
    /// Creates a new [`Cooldown`], which will take `max_time` after it is used until it is ready again.
    ///
    /// # Panics
    ///
    /// The provided max time cannot be [`Duration::ZERO`].
    /// Instead, use [`None`] in the [`Cooldowns`] struct for an action without a cooldown.
    pub fn new(max_time: Duration) -> Cooldown {
        assert!(max_time != Duration::ZERO);

        Cooldown {
            max_time,
            elapsed_time: Duration::ZERO,
            charges: 1,
            max_charges: 1,
        }
    }

    /// Creates a new [`Cooldown`] with a [`f32`] number of seconds, which will take `max_time` after it is used until it is ready again.
    ///
    /// # Panics
    ///
    /// The provided max time must be greater than 0.
    /// Instead, use [`None`] in the [`Cooldowns`] struct for an action without a cooldown.
    pub fn from_secs(max_time: f32) -> Cooldown {
        assert!(max_time > 0.);
        let max_time = Duration::from_secs_f32(max_time);

        Cooldown::new(max_time)
    }

    /// Advance the cooldown by `delta_time`.
    ///
    /// If the elapsed time is enough to reset the cooldown, the number of available charges.
    pub fn tick(&mut self, delta_time: Duration) {
        assert!(self.max_time != Duration::ZERO);

        // Ticking a cooldown at max charges has no effect
        if self.charges == self.max_charges {
            // Elapsed time is always zero when we're at max charges
            self.elapsed_time = Duration::ZERO;
            return;
        }

        let total_time = self.elapsed_time + delta_time;

        let total_nanos: u64 = total_time.as_nanos().try_into().unwrap_or(u64::MAX);
        let max_nanos: u64 = self.max_time.as_nanos().try_into().unwrap_or(u64::MAX);

        let n_completed = (total_nanos / max_nanos).try_into().unwrap_or(u8::MAX);
        let extra_time = Duration::from_nanos(total_nanos % max_nanos);

        let excess_completions = self.add_charges(n_completed);
        if excess_completions == 0 {
            self.elapsed_time = (self.elapsed_time + extra_time).min(self.max_time);
        } else {
            self.elapsed_time = self.max_time;
        }
    }

    /// Is this action ready to be used?
    ///
    /// This will be true if and only if at least one charge is available.
    /// For cooldowns without charges, this will be true if `time_remaining` is [`Duration::Zero`].
    pub fn ready(&self) -> bool {
        self.charges > 0
    }

    /// Refreshes the cooldown, causing the underlying action to be ready to use immediately.
    ///
    /// If this cooldown has charges, the number of available charges is increased by one (but the point within the cycle is unchanged).
    #[inline]
    pub fn refresh(&mut self) {
        self.add_charges(1);
    }

    /// Use the underlying cooldown if and only if it is ready, resetting the cooldown to its maximum value.
    ///
    /// If this cooldown has multiple charges, only one will be consumed.
    ///
    /// Returns a boolean indicating whether the cooldown was ready.
    /// If the cooldown was not ready, has no effect.
    #[inline]
    pub fn trigger(&mut self) -> bool {
        if self.ready() {
            self.charges = self.charges.saturating_sub(1);
            true
        } else {
            false
        }
    }

    /// Returns the time that it will take for this action to be ready to use again after being triggered.
    #[inline]
    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    /// Sets the time that it will take for this action to be ready to use again after being triggered.
    ///
    /// If the current time remaining is greater than the new max time, it will be clamped to the `max_time`.
    ///
    /// # Panics
    ///
    /// The provided max time cannot be [`Duration::ZERO`].
    /// Instead, use [`None`] in the [`Cooldowns`] struct for an action without a cooldown.
    #[inline]
    pub fn set_max_time(&mut self, max_time: Duration) {
        assert!(max_time != Duration::ZERO);

        self.max_time = max_time;
        self.elapsed_time = self.elapsed_time.min(max_time);
    }

    /// Returns the time that has passed since the cooldown was triggered.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed_time
    }

    /// Sets the time that has passed since the cooldown was triggered.
    ///
    /// This will always be clamped between [`Duration::ZERO`] and the `max_time` of this cooldown.
    #[inline]
    pub fn set_elapsed(&mut self, elapsed_time: Duration) {
        self.elapsed_time = elapsed_time.clamp(Duration::ZERO, self.max_time);
    }

    /// Returns the time remaining until the next charge is ready.
    ///
    /// When a cooldown is fully charged, this will return [`Duration::ZERO`].
    #[inline]
    pub fn remaining(&self) -> Duration {
        if self.elapsed_time == Duration::ZERO && self.charges == self.max_charges {
            // Users expect cooldowns that are ready to have zero time remaining,
            // so we wrap over the math to account for this
            Duration::ZERO
        } else {
            self.max_time.saturating_sub(self.elapsed_time)
        }
    }

    /// Sets the time remaining until the next charge is ready.
    ///
    /// This will always be clamped between [`Duration::ZERO`] and the `max_time` of this cooldown.
    #[inline]
    pub fn set_remaining(&mut self, time_remaining: Duration) {
        self.elapsed_time = self.max_time - time_remaining.clamp(Duration::ZERO, self.max_time);
    }
}

// Charge-related methods
impl Cooldown {
    /// Transforms the [`Cooldown`] to set the `max_charges` to the provided value.
    ///
    /// All charges will be available when first created.
    #[inline]
    pub fn with_charges(self, max_charges: u8) -> Cooldown {
        Cooldown {
            charges: max_charges,
            max_charges,
            ..self
        }
    }

    /// The current number of available charges
    #[inline]
    pub fn charges(&self) -> u8 {
        self.charges
    }

    /// The maximum number of available charges
    #[inline]
    pub fn max_charges(&self) -> u8 {
        self.max_charges
    }

    /// Adds `charges` to the current number of available charges
    ///
    /// This will never exceed the maximum number of charges.
    /// Returns the number of excess charges.
    #[inline]
    pub fn add_charges(&mut self, charges: u8) -> u8 {
        let excess = (self.charges + charges).saturating_sub(self.max_charges);
        self.charges = (self.charges + charges).min(self.max_charges);
        excess
    }

    /// Set the current number of available charges
    ///
    /// This will never exceed the maximum number of charges.
    /// Returns the number of excess charges.
    #[inline]
    pub fn set_charges(&mut self, charges: u8) -> u8 {
        let excess = charges.saturating_sub(self.max_charges);
        self.charges = charges.min(self.max_charges);
        excess
    }

    /// Set the maximmum number of available charges
    ///
    /// If the number of charges available is greater than this number, it will be reduced to the new cap.
    #[inline]
    pub fn set_max_charges(&mut self, max_charges: u8) {
        self.max_charges = max_charges;
        self.charges = self.charges.min(self.max_charges);
    }
}

#[cfg(test)]
mod tick_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn zero_duration_cooldown_cannot_be_constructed() {
        Cooldown::new(Duration::ZERO);
    }

    #[test]
    fn tick_has_no_effect_on_fresh_cooldown() {
        let cooldown = Cooldown::from_secs(1.);
        let mut cloned_cooldown = cooldown.clone();
        cloned_cooldown.tick(Duration::from_secs_f32(1.234));
        assert_eq!(cooldown, cloned_cooldown);
    }

    #[test]
    fn cooldowns_are_ready_when_refreshed() {
        let mut cooldown = Cooldown::from_secs(1.);
        assert!(cooldown.ready());
        cooldown.trigger();
        assert!(!cooldown.ready());
        cooldown.refresh();
        assert!(cooldown.ready());
    }

    #[test]
    fn ticking_changes_cooldown() {
        let cooldown = Cooldown::new(Duration::from_millis(1000));
        let mut cloned_cooldown = cooldown.clone();
        cloned_cooldown.trigger();
        assert!(cooldown != cloned_cooldown);

        cloned_cooldown.tick(Duration::from_millis(123));
        assert!(cooldown != cloned_cooldown);
    }

    #[test]
    fn cooldowns_reset_after_being_ticked() {
        let mut cooldown = Cooldown::from_secs(1.);
        cooldown.trigger();
        assert!(!cooldown.ready());

        cooldown.tick(Duration::from_secs(3));
        assert!(cooldown.ready());
    }

    #[test]
    fn time_remaining_on_fresh_cooldown_is_zero() {
        let cooldown = Cooldown::from_secs(1.);
        assert_eq!(cooldown.remaining(), Duration::ZERO);
    }

    #[test]
    fn time_elapsed_on_fresh_cooldown_is_zero() {
        let cooldown = Cooldown::from_secs(1.);
        assert_eq!(cooldown.elapsed(), Duration::ZERO);
    }
}

#[cfg(test)]
mod charge_tests {
    use super::*;

    #[test]
    fn cooldowns_start_with_one_charge() {
        let cooldown = Cooldown::from_secs(1.);
        assert_eq!(cooldown.charges(), 1);
        assert_eq!(cooldown.max_charges(), 1);
    }

    #[test]
    fn cooldowns_have_no_charges_after_being_triggered() {
        let mut cooldown = Cooldown::from_secs(1.);
        cooldown.trigger();
        assert_eq!(cooldown.charges(), 0);
        assert!(!cooldown.ready())
    }

    #[test]
    fn cooldowns_gain_a_charge_when_refreshed() {
        let mut cooldown = Cooldown::from_secs(1.);
        cooldown.trigger();
        cooldown.refresh();
        assert_eq!(cooldown.charges(), 1);
    }

    #[test]
    fn charges_are_depleted() {
        let mut cooldown = Cooldown::from_secs(1.).with_charges(3);
        assert_eq!(cooldown.charges(), 3);
        cooldown.trigger();
        assert_eq!(cooldown.charges(), 2);
        cooldown.trigger();
        assert_eq!(cooldown.charges(), 1);
        cooldown.trigger();
        assert_eq!(cooldown.charges(), 0);
        cooldown.trigger();
        assert_eq!(cooldown.charges(), 0);
    }

    #[test]
    fn cooldown_time_is_preserved_when_charge_depleted() {
        let mut cooldown = Cooldown::new(Duration::from_millis(1000)).with_charges(3);
        cooldown.trigger();
        cooldown.tick(Duration::from_millis(300));
        assert_eq!(cooldown.remaining(), Duration::from_millis(700));
        cooldown.trigger();
        assert_eq!(cooldown.remaining(), Duration::from_millis(700));
    }

    #[test]
    fn charges_regenerate_one_at_a_time() {
        let mut cooldown = Cooldown::new(Duration::from_secs(1)).with_charges(3);
        cooldown.set_charges(0);
        assert_eq!(cooldown.charges(), 0);

        cooldown.tick(Duration::from_secs(1));
        assert_eq!(cooldown.charges(), 1);
        cooldown.tick(Duration::from_secs(1));
        assert_eq!(cooldown.charges(), 2);
        cooldown.tick(Duration::from_secs(1));
        assert_eq!(cooldown.charges(), 3);
        cooldown.tick(Duration::from_secs(1));
        assert_eq!(cooldown.charges(), 3);
    }

    #[test]
    fn multiple_charges_regenerate_in_long_tick() {
        let mut cooldown = Cooldown::from_secs(1.).with_charges(3);
        cooldown.set_charges(0);

        cooldown.tick(Duration::from_secs(2));
        assert_eq!(cooldown.charges(), 2);
    }
}
