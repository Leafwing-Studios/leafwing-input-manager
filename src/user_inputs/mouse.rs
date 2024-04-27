//! Mouse inputs

use bevy::prelude::{Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate::input_processing::*;
use crate::input_streams::InputStreams;
use crate::user_inputs::UserInput;

/// Represents mouse motion input, combining individual axis movements into a single value.
///
/// It uses an internal [`DualAxisProcessor`] to process raw mouse movement data
/// from multiple mouse motion events into a combined representation.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct MouseMotionInput(DualAxisProcessor);

#[serde_typetag]
impl UserInput for MouseMotionInput {
    /// Checks if there is active mouse motion.
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        self.axis_pair(input_streams).unwrap() != Vec2::ZERO
    }

    /// Retrieves the magnitude of the mouse motion, representing the overall amount of movement.
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        self.axis_pair(input_streams).unwrap().length()
    }

    /// Retrieves the accumulated mouse displacement.
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<Vec2> {
        let value = input_streams
            .mouse_motion
            .iter()
            .map(|event| event.delta)
            .sum();
        let value = self.0.process(value);
        Some(value)
    }
}
