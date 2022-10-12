//! Cooldowns tick down until actions are ready to be used.

use bevy::utils::Duration;
use serde::{Deserialize, Serialize};

/// A timer-like struct that records the amount of time until an action is available to be used again.
///
/// Cooldowns are typically stored in an [`ActionState`](crate::action_state::ActionState), associated with an action that is to be
/// cooldown-regulated.
///
/// When initialized, cooldowns are always fully available.
///
/// ```rust
/// let cooldown = Cooldown::new(Duration::from_secs(3));
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

    /// Advance the cooldown by `delta_time`.
    pub fn tick(&mut self, delta_time: Duration) {
        self.time_remaining = (self.time_remaining - delta_time).max(Duration::ZERO);
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
    pub fn set_max_time(&mut self, max_time: Duration) {
        self.max_time = max_time;
    }

    /// Returns the time remaining until the action is ready to use again.
    pub fn time_remaining(&self) -> Duration {
        self.time_remaining
    }

    /// Sets the time remaining until the action is ready to use again.
    pub fn set_time_remaining(&mut self, time_remaining: Duration) {
        self.time_remaining = time_remaining.max(Duration::ZERO);
    }
}
