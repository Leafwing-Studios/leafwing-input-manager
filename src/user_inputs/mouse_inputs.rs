//! Utilities for handling keyboard inputs.

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::input_streams::InputStreams;
use crate::user_inputs::axislike_settings::SingleAxisSettings;
use crate::user_inputs::UserInput;

/// Vertical mouse wheel input with settings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct MouseWheelVertical(pub SingleAxisSettings);

impl UserInput<'_> for MouseWheelVertical {
    /// Retrieves the magnitude of vertical mouse wheel movements.
    ///
    /// Returns `None` if mouse input tracking is unavailable.
    /// Returns `Some(0.0)` if the tracked mouse wheel isn't scrolling.
    /// Returns `Some(magnitude)` of the tracked mouse wheel along the y-axis.
    fn value(&self, input_query: InputStreams) -> Option<f32> {
        let mouse_wheel_input = input_query.mouse_wheel;
        mouse_wheel_input.map(|mouse_wheel| {
            let movements = mouse_wheel.iter().map(|wheel| wheel.y).sum();
            self.0.apply_settings(movements)
        })
    }

    /// Checks if the mouse wheel started scrolling vertically during the current tick.
    fn started(&self, _input_query: InputStreams) -> bool {
        // Unable to accurately determine this here;
        // it should be checked during the update of the `ActionState`.
        false
    }

    /// Checks if the mouse wheel stopped scrolling vertically during the current tick.
    fn finished(&self, _input_query: InputStreams) -> bool {
        // Unable to accurately determine this here;
        // it should be checked during the update of the `ActionState`.
        false
    }
}
