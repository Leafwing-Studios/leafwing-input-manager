//! Cooldowns tick down until actions are ready to be used.

use crate::Actionlike;

use bevy::ecs::prelude::Component;
use bevy::utils::Duration;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// The time until each action of type `A` can be used again.
///
/// Each action may be associated with a [`Cooldown`]
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
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Default for Cooldowns<A> {
    /// By default, cooldowns are not set.
    fn default() -> Self {
        Cooldowns {
            cooldowns: A::variants().map(|_| None).collect(),
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
    }

    /// Can the corresponding `action` be used?
    ///
    /// This will be `true` if the underlying [`Cooldown::ready`] call is true,
    /// or if no cooldown is stored for this action.
    #[inline]
    #[must_use]
    pub fn ready(&self, action: A) -> bool {
        if let Some(cooldown) = self.cooldown(action) {
            cooldown.ready()
        } else {
            true
        }
    }

    /// Advances each underlying [`Cooldown`] according to the elapsed `delta_time`.
    pub fn tick(&mut self, delta_time: Duration) {
        self.iter_mut().for_each(|cd| cd.tick(delta_time));
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
/// assert_eq!(cooldown.time_remaining(), Duration::ZERO);
///
/// cooldown.trigger();
/// assert_eq!(cooldown.time_remaining(), Duration::from_secs(3));
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
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Cooldown {
    max_time: Duration,
    time_remaining: Duration,
}

impl Cooldown {
    /// Creates a new [`Cooldown`], which will take `max_time` after it is used until it is ready again.
    pub fn new(max_time: Duration) -> Cooldown {
        Cooldown {
            max_time,
            time_remaining: Duration::ZERO,
        }
    }

    /// Creates a new [`Cooldown`] with a [`f32`] number of seconds, which will take `max_time` after it is used until it is ready again.
    pub fn from_secs(max_time: f32) -> Cooldown {
        Cooldown {
            max_time: Duration::from_secs_f32(max_time),
            time_remaining: Duration::ZERO,
        }
    }

    /// Advance the cooldown by `delta_time`.
    pub fn tick(&mut self, delta_time: Duration) {
        self.time_remaining = self.time_remaining.saturating_sub(delta_time);
    }

    /// Is this action ready to be used?
    ///
    /// This will be true if and only if the `time_remaining` is [`Duration::Zero`].
    pub fn ready(&self) -> bool {
        self.time_remaining == Duration::ZERO
    }

    /// Refreshes the cooldown, causing the underlying action to be ready to use immediately.
    pub fn refresh(&mut self) {
        self.time_remaining = Duration::ZERO;
    }

    /// Use the underlying cooldown if and only if it is ready, resetting the cooldown to its maximum value.
    ///
    /// Returns a boolean indicating whether the cooldown was ready.
    pub fn trigger(&mut self) -> bool {
        if self.ready() {
            self.time_remaining = self.max_time;
            true
        } else {
            false
        }
    }

    /// Returns the time that it will take for this action to be ready to use again after being triggered.
    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    /// Sets the time that it will take for this action to be ready to use again after being triggered.
    ///
    /// If the current time remaining is greater than the new max time, it will be clamped to the `max_time`.
    pub fn set_max_time(&mut self, max_time: Duration) {
        self.max_time = max_time;
        self.time_remaining = self.time_remaining.min(max_time);
    }

    /// Returns the time remaining until the action is ready to use again.
    pub fn time_remaining(&self) -> Duration {
        self.time_remaining
    }

    /// Sets the time remaining until the action is ready to use again.
    ///
    /// This will always be clamped between [`Duration::ZERO`] and the `max_time` of this cooldown.
    pub fn set_time_remaining(&mut self, time_remaining: Duration) {
        self.time_remaining = time_remaining.clamp(Duration::ZERO, self.max_time);
    }
}
