//! This module contains [`ChordInput`] and its supporting methods and impls..

use bevy::prelude::{Reflect, Vec2};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate::input_streams::InputStreams;
use crate::user_inputs::UserInput;

/// A combined input that holds multiple [`UserInput`]s to represent simultaneous button presses.
///
/// # Behaviors
///
/// When it treated as a button, it checks if all inner inputs are active simultaneously.
/// When it treated as a single-axis input, it uses the sum of values from all inner single-axis inputs.
/// When it treated as a dual-axis input, it only uses the value of the first inner dual-axis input.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub struct ChordInput(
    // Note: we cannot use a HashSet here because of https://users.rust-lang.org/t/hash-not-implemented-why-cant-it-be-derived/92416/8
    // We cannot use a BTreeSet because the underlying types don't impl Ord
    // We don't want to use a PetitSet here because of memory bloat
    // So a vec it is!
    // RIP your uniqueness guarantees
    Vec<Box<dyn UserInput>>,
);

// #[serde_typetag]
impl UserInput for ChordInput {
    /// Checks if all the inner inputs are active.
    #[inline]
    fn is_active(&self, input_streams: &InputStreams) -> bool {
        self.0.iter().all(|input| input.is_active(input_streams))
    }

    /// Returns a combined value representing the input.
    ///
    /// # Returns
    ///
    ///
    #[inline]
    fn value(&self, input_streams: &InputStreams) -> f32 {
        match self.0.iter().next() {
            Some(input) => input.value(input_streams),
            None => 0.0,
        }
    }

    /// Retrieves the value of the first inner dual-axis input.
    #[inline]
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<Vec2> {
        self.0
            .iter()
            .find_map(|input| input.axis_pair(input_streams))
    }
}
