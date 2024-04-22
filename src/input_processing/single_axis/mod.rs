//! Processors for single-axis input values

use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use bevy::prelude::{FromReflect, Reflect, ReflectDeserialize, ReflectSerialize, TypePath};
use bevy::reflect::utility::{reflect_hasher, GenericTypePathCell, NonGenericTypeInfoCell};
use bevy::reflect::{
    FromType, GetTypeRegistration, ReflectFromPtr, ReflectKind, ReflectMut, ReflectOwned,
    ReflectRef, TypeInfo, TypeRegistration, Typed, ValueInfo,
};
use bevy::utils::FloatOrd;
use dyn_eq::DynEq;
use serde::{Deserialize, Serialize};

pub use self::custom::*;
pub use self::range::*;

mod custom;
mod range;

/// A processor for single-axis input values,
/// accepting a `f32` input and producing a `f32` output.
#[must_use]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum AxisProcessor {
    /// Flips the sign of input values, resulting in a directional reversal of control.
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// assert_eq!(AxisProcessor::Inverted.process(2.5), -2.5);
    /// assert_eq!(AxisProcessor::Inverted.process(-2.5), 2.5);
    /// ```
    Inverted,

    /// Scales input values using a specified multiplier to fine-tune the responsiveness of control.
    ///
    /// ```rust
    /// use leafwing_input_manager::prelude::*;
    ///
    /// // Doubled!
    /// assert_eq!(AxisProcessor::Sensitivity(2.0).process(2.0), 4.0);
    ///
    /// // Halved!
    /// assert_eq!(AxisProcessor::Sensitivity(0.5).process(2.0), 1.0);
    ///
    /// // Negated and halved!
    /// assert_eq!(AxisProcessor::Sensitivity(-0.5).process(2.0), -1.0);
    /// ```
    Sensitivity(f32),

    /// A wrapper around [`AxisBounds`] to represent value bounds.
    ValueBounds(AxisBounds),

    /// A wrapper around [`AxisExclusion`] to represent unscaled deadzone.
    Exclusion(AxisExclusion),

    /// A wrapper around [`AxisDeadZone`] to represent scaled deadzone.
    DeadZone(AxisDeadZone),

    /// Processes input values sequentially by chaining together two [`AxisProcessor`]s,
    /// one for the current step and the other for the next step.
    Sequential(Box<AxisProcessor>, Box<AxisProcessor>),

    /// A user-defined processor that implements [`CustomAxisProcessor`].
    Custom(Box<dyn CustomAxisProcessor>),
}

impl AxisProcessor {
    /// Computes the result by processing the `input_value`.
    #[must_use]
    #[inline]
    pub fn process(&self, input_value: f32) -> f32 {
        match self {
            Self::Inverted => -input_value,
            Self::Sensitivity(sensitivity) => sensitivity * input_value,
            Self::ValueBounds(bounds) => bounds.clamp(input_value),
            Self::Exclusion(exclusion) => exclusion.exclude(input_value),
            Self::DeadZone(deadzone) => deadzone.normalize(input_value),
            Self::Sequential(current, next) => next.process(current.process(input_value)),
            Self::Custom(processor) => processor.process(input_value),
        }
    }

    /// Appends the given `next_processor` as the next processing step.
    #[inline]
    pub fn with_processor(self, next_processor: impl Into<AxisProcessor>) -> Self {
        Self::Sequential(Box::new(self), Box::new(next_processor.into()))
    }
}

impl Eq for AxisProcessor {}

impl Hash for AxisProcessor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Inverted => {}
            Self::Sensitivity(sensitivity) => FloatOrd(*sensitivity).hash(state),
            Self::ValueBounds(bounds) => bounds.hash(state),
            Self::Exclusion(exclusion) => exclusion.hash(state),
            Self::DeadZone(deadzone) => deadzone.hash(state),
            Self::Sequential(current, next) => {
                current.hash(state);
                next.hash(state);
            }
            Self::Custom(processor) => processor.hash(state),
        }
    }
}

impl Reflect for Box<AxisProcessor> {
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(Self::type_info())
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }

    fn apply(&mut self, value: &dyn Reflect) {
        let value = value.as_any();
        if let Some(value) = value.downcast_ref::<Self>() {
            self.clone_from(value);
        } else {
            panic!(
                "Value is not a std::boxed::Box<dyn {}::AxisProcessor>.",
                module_path!(),
            );
        }
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }

    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Value
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Value(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Value(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Value(self)
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn reflect_hash(&self) -> Option<u64> {
        let mut hasher = reflect_hasher();
        let type_id = TypeId::of::<Self>();
        Hash::hash(&type_id, &mut hasher);
        Hash::hash(self, &mut hasher);
        Some(hasher.finish())
    }

    fn reflect_partial_eq(&self, value: &dyn Reflect) -> Option<bool> {
        let value = value.as_any();
        value
            .downcast_ref::<Self>()
            .map(|value| self.dyn_eq(value))
            .or(Some(false))
    }

    fn debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Typed for Box<AxisProcessor> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
    }
}

impl TypePath for Box<AxisProcessor> {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            {
                format!("std::boxed::Box<{}::AxisProcessor>", module_path!())
            }
        })
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| "Box<AxisProcessor>".to_string())
    }

    fn type_ident() -> Option<&'static str> {
        Some("Box<AxisProcessor>")
    }

    fn crate_name() -> Option<&'static str> {
        Some(module_path!().split(':').next().unwrap())
    }

    fn module_path() -> Option<&'static str> {
        Some(module_path!())
    }
}

impl GetTypeRegistration for Box<AxisProcessor> {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for Box<AxisProcessor> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.as_any().downcast_ref::<Self>()?.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis_inversion_processor() {
        for value in -300..300 {
            let value = value as f32 * 0.01;

            assert_eq!(AxisProcessor::Inverted.process(value), -value);
            assert_eq!(AxisProcessor::Inverted.process(-value), value);
        }
    }

    #[test]
    fn test_axis_sensitivity_processor() {
        for value in -300..300 {
            let value = value as f32 * 0.01;

            for sensitivity in -300..300 {
                let sensitivity = sensitivity as f32 * 0.01;

                let processor = AxisProcessor::Sensitivity(sensitivity);
                assert_eq!(processor.process(value), sensitivity * value);
            }
        }
    }
}
