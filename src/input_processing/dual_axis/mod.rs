//! Processors for dual-axis input values

use std::hash::Hash;
use std::sync::Arc;

use bevy::prelude::{Reflect, Vec2};
use serde::{Deserialize, Serialize};

pub use self::circle::*;
pub use self::custom::*;
pub use self::modifier::*;
pub use self::range::*;

mod circle;
mod custom;
mod modifier;
mod range;

/// A processor for dual-axis input values,
/// accepting a [`Vec2`] input and producing a [`Vec2`] output.
#[must_use]
#[non_exhaustive]
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DualAxisProcessor {
    /// No processor is applied.
    #[default]
    None,

    /// A wrapper around [`DualAxisInverted`] to represent inversion.
    Inverted(DualAxisInverted),

    /// A wrapper around [`DualAxisSensitivity`] to represent sensitivity.
    Sensitivity(DualAxisSensitivity),

    /// A wrapper around [`DualAxisBounds`] to represent value bounds.
    ValueBounds(DualAxisBounds),

    /// A wrapper around [`DualAxisExclusion`] to represent unscaled deadzone.
    Exclusion(DualAxisExclusion),

    /// A wrapper around [`DualAxisDeadZone`] to represent scaled deadzone.
    DeadZone(DualAxisDeadZone),

    /// A wrapper around [`CircleBounds`] to represent circular value bounds.
    CircleBounds(CircleBounds),

    /// A wrapper around [`CircleExclusion`] to represent unscaled deadzone.
    CircleExclusion(CircleExclusion),

    /// A wrapper around [`CircleDeadZone`] to represent scaled deadzone.
    CircleDeadZone(CircleDeadZone),

    /// Processes input values sequentially through a sequence of [`DualAxisProcessor`]s.
    /// one for the current step and the other for the next step.
    ///
    /// For a straightforward creation of a [`DualAxisProcessor::Pipeline`],
    /// you can use [`DualAxisProcessor::with_processor`] or [`From<Vec<DualAxisProcessor>>::from`] methods.
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// let expected = DualAxisProcessor::Pipeline(vec![
    ///     Arc::new(DualAxisInverted::ALL.into()),
    ///     Arc::new(DualAxisSensitivity::all(2.0).into()),
    /// ]);
    ///
    /// assert_eq!(
    ///     expected,
    ///     DualAxisProcessor::from(DualAxisInverted::ALL).with_processor(DualAxisSensitivity::all(2.0))
    /// );
    ///
    /// assert_eq!(
    ///     expected,
    ///     DualAxisProcessor::from(vec![
    ///         DualAxisInverted::ALL.into(),
    ///         DualAxisSensitivity::all(2.0).into(),
    ///     ])
    /// );
    Pipeline(Vec<Arc<DualAxisProcessor>>),

    /// A user-defined processor that implements [`CustomDualAxisProcessor`].
    Custom(Box<dyn CustomDualAxisProcessor>),
}

impl DualAxisProcessor {
    /// Computes the result by processing the `input_value`.
    #[must_use]
    #[inline]
    pub fn process(&self, input_value: Vec2) -> Vec2 {
        match self {
            Self::None => input_value,
            Self::Inverted(inversion) => inversion.invert(input_value),
            Self::Sensitivity(sensitivity) => sensitivity.scale(input_value),
            Self::ValueBounds(bounds) => bounds.clamp(input_value),
            Self::Exclusion(exclusion) => exclusion.exclude(input_value),
            Self::DeadZone(deadzone) => deadzone.normalize(input_value),
            Self::CircleBounds(bounds) => bounds.clamp(input_value),
            Self::CircleExclusion(exclusion) => exclusion.exclude(input_value),
            Self::CircleDeadZone(deadzone) => deadzone.normalize(input_value),
            Self::Pipeline(sequence) => sequence
                .iter()
                .fold(input_value, |value, next| next.process(value)),
            Self::Custom(processor) => processor.process(input_value),
        }
    }

    /// Appends the given `next_processor` as the next processing step.
    ///
    /// - If either processor is [`DualAxisProcessor::None`], returns the other.
    /// - If the current processor is [`DualAxisProcessor::Pipeline`], pushes the other into it.
    /// - If the given processor is [`DualAxisProcessor::Pipeline`], prepends the current one into it.
    /// - If both processors are [`DualAxisProcessor::Pipeline`], merges the two pipelines.
    /// - If neither processor is [`DualAxisProcessor::None`] nor a pipeline,
    ///     creates a new pipeline containing them.
    #[inline]
    pub fn with_processor(self, next_processor: impl Into<DualAxisProcessor>) -> Self {
        let other = next_processor.into();
        match (self.clone(), other.clone()) {
            (_, Self::None) => self,
            (Self::None, _) => other,
            (Self::Pipeline(mut self_seq), Self::Pipeline(mut next_seq)) => {
                self_seq.append(&mut next_seq);
                Self::Pipeline(self_seq)
            }
            (Self::Pipeline(mut self_seq), _) => {
                self_seq.push(Arc::new(other));
                Self::Pipeline(self_seq)
            }
            (_, Self::Pipeline(mut next_seq)) => {
                next_seq.insert(0, Arc::new(self));
                Self::Pipeline(next_seq)
            }
            (_, _) => Self::Pipeline(vec![Arc::new(self), Arc::new(other)]),
        }
    }
}

impl From<Vec<DualAxisProcessor>> for DualAxisProcessor {
    fn from(value: Vec<DualAxisProcessor>) -> Self {
        Self::Pipeline(value.into_iter().map(Arc::new).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_processor_pipeline() {
        let pipeline = DualAxisProcessor::Pipeline(vec![
            Arc::new(DualAxisInverted::ALL.into()),
            Arc::new(DualAxisSensitivity::all(2.0).into()),
        ]);

        for x in -300..300 {
            let x = x as f32 * 0.01;
            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                assert_eq!(pipeline.process(value), value * -2.0);
            }
        }
    }

    #[test]
    fn test_dual_axis_processor_from_list() {
        assert_eq!(
            DualAxisProcessor::from(vec![]),
            DualAxisProcessor::Pipeline(vec![])
        );

        assert_eq!(
            DualAxisProcessor::from(vec![DualAxisInverted::ALL.into()]),
            DualAxisProcessor::Pipeline(vec![Arc::new(DualAxisInverted::ALL.into())])
        );

        assert_eq!(
            DualAxisProcessor::from(vec![
                DualAxisInverted::ALL.into(),
                DualAxisSensitivity::all(2.0).into(),
            ]),
            DualAxisProcessor::Pipeline(vec![
                Arc::new(DualAxisInverted::ALL.into()),
                Arc::new(DualAxisSensitivity::all(2.0).into()),
            ])
        );
    }
}
