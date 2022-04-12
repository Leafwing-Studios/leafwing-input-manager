//! Tools for working with button-like user inputs (mouse clicks, gamepad button, keyboard inputs and so on)
//!
use bevy_utils::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::user_input::UserInput;

/// The current state of a particular virtual button,
/// corresponding to a single [`Actionlike`] action.
///
/// Detailed timing information for the button can be accessed through the stored [`Timing`] value.
/// When the button is pressed, you can inspect *why* it was pressed,
/// allowing you to access information about e.g. the degree to which a trigger was depressed or the exact inputs used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ButtonState {
    /// This button is currently pressed
    Pressed {
        /// How long has this button been pressed for, and when was it first pressed?
        timing: Timing,
        /// What [`UserInput`]s (including their values) were responsible for this button being pressed?
        reasons_pressed: Vec<UserInput>,
    },
    /// This button is currently released
    Released {
        /// How long has this button been released for, and when was it first pressed?
        timing: Timing,
    },
}

/// Stores the timing information for a [`VirtualButtonState`]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Timing {
    /// The [`Instant`] at which the button was pressed or released
    ///
    /// Recorded as the [`Time`](bevy::core::Time) at the start of the tick after the state last changed.
    /// If this is none, [`ActionState::update`] has not been called yet.
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

impl PartialOrd for ButtonState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self {
            ButtonState::Pressed {
                timing,
                reasons_pressed: _,
            } => match other {
                ButtonState::Pressed {
                    timing: other_timing,
                    reasons_pressed: _,
                } => timing.partial_cmp(other_timing),
                ButtonState::Released {
                    timing: other_timing,
                } => timing.partial_cmp(other_timing),
            },
            ButtonState::Released { timing } => match other {
                ButtonState::Pressed {
                    timing: other_timing,
                    reasons_pressed: _,
                } => timing.partial_cmp(other_timing),
                ButtonState::Released {
                    timing: other_timing,
                } => timing.partial_cmp(other_timing),
            },
        }
    }
}

impl ButtonState {
    /// A [`VirtualButtonState`] that is just pressed, with no history
    pub const JUST_PRESSED: ButtonState = ButtonState::Pressed {
        timing: Timing {
            instant_started: None,
            current_duration: Duration::ZERO,
            previous_duration: Duration::ZERO,
        },
        reasons_pressed: Vec::new(),
    };

    /// A [`VirtualButtonState`] that is just released, with no history
    pub const JUST_RELEASED: ButtonState = ButtonState::Released {
        timing: Timing {
            instant_started: None,
            current_duration: Duration::ZERO,
            previous_duration: Duration::ZERO,
        },
    };

    /// Presses the virtual button
    ///
    /// Records:
    ///
    /// * the [`Duration`] for which the button was previously held
    /// * the [`Instant`] that this button was pressed
    /// * the [`UserInput`]s responsible for this button being pressed
    #[inline]
    pub fn press(&mut self, instant_started: Option<Instant>, reasons_pressed: Vec<UserInput>) {
        if let ButtonState::Released {
            timing: previous_timing,
        } = self
        {
            *self = ButtonState::Pressed {
                timing: Timing {
                    instant_started,
                    current_duration: Duration::ZERO,
                    previous_duration: previous_timing.current_duration,
                },
                reasons_pressed,
            }
        }
    }

    /// Releases the virtual button
    ///
    /// Records:
    ///
    /// * the [`Duration`] for which the button was previously held
    /// * the [`Instant`] that this button was pressed
    /// * the [`UserInput`]s responsible for this button being pressed
    #[inline]
    pub fn release(&mut self, instant_started: Option<Instant>) {
        if let ButtonState::Pressed {
            timing: previous_timing,
            reasons_pressed: _,
        } = self
        {
            *self = ButtonState::Released {
                timing: Timing {
                    instant_started,
                    current_duration: Duration::ZERO,
                    previous_duration: previous_timing.current_duration,
                },
            }
        }
    }

    /// Is the button currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self) -> bool {
        match self {
            ButtonState::Pressed {
                timing: _,
                reasons_pressed: _,
            } => true,
            ButtonState::Released { timing: _ } => false,
        }
    }

    /// Advances the time for the virtual button
    ///
    /// The [`VirtualButtonState`] state will be advanced according to the `current_instant`.
    /// - if no [`Instant`] is set, the `current_time` will be set as the initial time at which the button was pressed / released
    /// - the [`Duration`] will advance to reflect elapsed time
    #[inline]
    pub fn tick(&mut self, current_instant: Instant) {
        match self {
            ButtonState::Pressed {
                timing,
                reasons_pressed: _,
            } => match timing.instant_started {
                Some(instant) => {
                    timing.current_duration = current_instant - instant;
                }
                None => {
                    timing.instant_started = Some(current_instant);
                    timing.current_duration = Duration::ZERO;
                }
            },
            ButtonState::Released { timing } => match timing.instant_started {
                Some(instant) => {
                    timing.current_duration = current_instant - instant;
                }
                None => {
                    timing.instant_started = Some(current_instant);
                    timing.current_duration = Duration::ZERO;
                }
            },
        };
    }

    /// Is the button currently released?
    #[inline]
    #[must_use]
    pub fn released(&self) -> bool {
        match self {
            ButtonState::Pressed {
                timing: _,
                reasons_pressed: _,
            } => false,
            ButtonState::Released { timing: _ } => true,
        }
    }

    /// Was the button pressed since the last time [`ActionState::update`] was called?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self) -> bool {
        match self {
            ButtonState::Pressed {
                timing,
                reasons_pressed: _,
            } => timing.instant_started.is_none(),
            ButtonState::Released { timing: _ } => false,
        }
    }

    /// Was the button released since the last time [`ActionState::update`] was called?
    #[inline]
    #[must_use]
    pub fn just_released(&self) -> bool {
        match self {
            ButtonState::Pressed {
                timing: _,
                reasons_pressed: _,
            } => false,
            ButtonState::Released { timing } => timing.instant_started.is_none(),
        }
    }

    /// The [`Instant`] at which the button was pressed or released
    ///
    /// Recorded as the [`Time`](bevy::core::Time) at the start of the tick after the state last changed.
    /// If this is none, [`ActionState::update`] has not been called yet.
    #[inline]
    #[must_use]
    pub fn instant_started(&self) -> Option<Instant> {
        match self {
            ButtonState::Pressed {
                timing,
                reasons_pressed: _,
            } => timing.instant_started,
            ButtonState::Released { timing } => timing.instant_started,
        }
    }

    /// The [`Duration`] for which the button has been pressed or released.
    ///
    /// This begins at [`Duration::ZERO`] when [`ActionState::update`] is called.
    #[inline]
    #[must_use]
    pub fn current_duration(&self) -> Duration {
        match self {
            ButtonState::Pressed {
                timing,
                reasons_pressed: _,
            } => timing.current_duration,
            ButtonState::Released { timing } => timing.current_duration,
        }
    }

    /// The [`Duration`] for which the button was pressed or released before the state last changed.
    #[inline]
    #[must_use]
    pub fn previous_duration(&self) -> Duration {
        match self {
            ButtonState::Pressed {
                timing,
                reasons_pressed: _,
            } => timing.previous_duration,
            ButtonState::Released { timing } => timing.previous_duration,
        }
    }

    /// The reasons (in terms of [`UserInput`]) that the button was pressed
    ///
    /// If the button is currently released, the `Vec<UserInput`> returned will be empty
    ///
    /// # Example
    ///
    /// ```rust
    /// use leafwing_input_manager::buttonlike::VirtualButtonState;
    /// use bevy_input::keyboard::KeyCode;
    ///
    /// let mut state = VirtualButtonState::JUST_RELEASED;
    ///
    /// assert_eq!(state.reasons_pressed(), Vec::new());
    ///
    /// state.press(None, vec![KeyCode::Space.into()]);
    /// assert!(state.pressed());
    /// assert_eq!(state.reasons_pressed(), vec![KeyCode::Space.into()]);
    /// ```
    #[inline]
    #[must_use]
    pub fn reasons_pressed(&self) -> Vec<UserInput> {
        match self {
            ButtonState::Pressed {
                timing: _,
                reasons_pressed,
            } => reasons_pressed.clone(),
            ButtonState::Released { timing: _ } => Vec::new(),
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Released {
            timing: Timing::default(),
        }
    }
}
