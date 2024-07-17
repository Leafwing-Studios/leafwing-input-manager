//! Information about when an action was pressed or released.

use bevy::{
    reflect::Reflect,
    utils::{Duration, Instant},
};
use serde::{Deserialize, Serialize};

/// Stores information about when an action was pressed or released
///
/// This struct is principally used as a field on [`ButtonData`](crate::action_state::ButtonData),
/// which itself lives inside an [`ActionState`](crate::action_state::ActionState).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, Reflect)]
pub struct Timing {
    /// The [`Instant`] at which the button was pressed or released
    /// Recorded as the [`Time`](bevy::time::Time) at the start of the tick after the state last changed.
    /// If this is none, [`Timing::tick`] has not been called yet.
    #[serde(skip)]
    pub instant_started: Option<Instant>,
    /// The [`Duration`] for which the button has been pressed or released.
    ///
    /// This begins at [`Duration::ZERO`] when [`ActionState::update`](crate::action_state::ActionState::update) is called.
    pub current_duration: Duration,
    /// The [`Duration`] for which the button was pressed or released before the state last changed.
    pub previous_duration: Duration,
}

impl Timing {
    /// The default timing for a button that has not been pressed or released
    pub const NEW: Timing = Timing {
        instant_started: None,
        current_duration: Duration::ZERO,
        previous_duration: Duration::ZERO,
    };
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
    pub fn flip(&mut self) {
        self.previous_duration = self.current_duration;
        self.current_duration = Duration::ZERO;
        self.instant_started = None;
    }
}

#[cfg(test)]
mod tests {
    use crate as leafwing_input_manager;
    use bevy::prelude::Reflect;
    use leafwing_input_manager_macros::Actionlike;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn time_tick_ticks_away() {
        use crate::action_state::ActionState;
        use bevy::utils::{Duration, Instant};

        let mut action_state = ActionState::<Action>::default();

        // Actions start released (but not just released)
        assert!(action_state.released(&Action::Run));
        assert!(!action_state.just_released(&Action::Jump));

        // Ticking causes buttons just released to no longer be just released
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        assert!(action_state.released(&Action::Jump));
        assert!(!action_state.just_released(&Action::Jump));
        action_state.press(&Action::Jump);
        assert!(action_state.just_pressed(&Action::Jump));

        // Ticking causes buttons just pressed to no longer be just pressed
        action_state.tick(Instant::now(), Instant::now() - Duration::from_micros(1));
        assert!(action_state.pressed(&Action::Jump));
        assert!(!action_state.just_pressed(&Action::Jump));
    }

    #[test]
    fn durations() {
        use crate::action_state::ActionState;
        use bevy::utils::{Duration, Instant};

        let mut action_state = ActionState::<Action>::default();

        // Actions start released
        assert!(action_state.released(&Action::Jump));
        assert_eq!(action_state.instant_started(&Action::Jump), None,);
        assert_eq!(action_state.current_duration(&Action::Jump), Duration::ZERO);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Pressing a button swaps the state
        action_state.press(&Action::Jump);
        assert!(action_state.pressed(&Action::Jump));
        assert_eq!(action_state.instant_started(&Action::Jump), None);
        assert_eq!(action_state.current_duration(&Action::Jump), Duration::ZERO);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Ticking time sets the instant for the new state
        let t0 = Instant::now();
        let t1 = t0 + Duration::new(1, 0);

        action_state.tick(t1, t0);
        assert_eq!(action_state.instant_started(&Action::Jump), Some(t0));
        assert_eq!(action_state.current_duration(&Action::Jump), t1 - t0);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Time passes
        let t2 = t1 + Duration::new(5, 0);

        // The duration is updated
        action_state.tick(t2, t1);
        assert_eq!(action_state.instant_started(&Action::Jump), Some(t0));
        assert_eq!(action_state.current_duration(&Action::Jump), t2 - t0);
        assert_eq!(
            action_state.previous_duration(&Action::Jump),
            Duration::ZERO
        );

        // Releasing again, swapping the current duration to the previous one
        action_state.release(&Action::Jump);
        assert_eq!(action_state.instant_started(&Action::Jump), None);
        assert_eq!(action_state.current_duration(&Action::Jump), Duration::ZERO);
        assert_eq!(action_state.previous_duration(&Action::Jump), t2 - t0);
    }
}
