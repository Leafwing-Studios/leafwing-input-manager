//! Processors for dual-axis input values

use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use bevy::prelude::{FromReflect, Reflect, ReflectDeserialize, ReflectSerialize, TypePath, Vec2};
use bevy::reflect::utility::{reflect_hasher, GenericTypePathCell, NonGenericTypeInfoCell};
use bevy::reflect::{
    FromType, GetTypeRegistration, ReflectFromPtr, ReflectKind, ReflectMut, ReflectOwned,
    ReflectRef, TypeInfo, TypeRegistration, Typed, ValueInfo,
};
use dyn_eq::DynEq;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum DualAxisProcessor {
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

    /// Processes input values sequentially by chaining together two [`DualAxisProcessor`]s,
    /// one for the current step and the other for the next step.
    Sequential(Box<DualAxisProcessor>, Box<DualAxisProcessor>),

    /// A user-defined processor that implements [`CustomDualAxisProcessor`].
    Custom(Box<dyn CustomDualAxisProcessor>),
}

impl DualAxisProcessor {
    /// Computes the result by processing the `input_value`.
    #[must_use]
    #[inline]
    pub fn process(&self, input_value: Vec2) -> Vec2 {
        match self {
            Self::Inverted(inversion) => inversion.invert(input_value),
            Self::Sensitivity(sensitivity) => sensitivity.scale(input_value),
            Self::ValueBounds(bounds) => bounds.clamp(input_value),
            Self::Exclusion(exclusion) => exclusion.exclude(input_value),
            Self::DeadZone(deadzone) => deadzone.normalize(input_value),
            Self::CircleBounds(bounds) => bounds.clamp(input_value),
            Self::CircleExclusion(exclusion) => exclusion.exclude(input_value),
            Self::CircleDeadZone(deadzone) => deadzone.normalize(input_value),
            Self::Sequential(current, next) => next.process(current.process(input_value)),
            Self::Custom(processor) => processor.process(input_value),
        }
    }

    /// Appends the given `next_processor` as the next processing step.
    #[inline]
    pub fn with_processor(self, next_processor: impl Into<DualAxisProcessor>) -> Self {
        Self::Sequential(Box::new(self), Box::new(next_processor.into()))
    }
}

impl Reflect for Box<DualAxisProcessor> {
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
                "Value is not a std::boxed::Box<dyn {}::DualAxisProcessor>.",
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

impl Typed for Box<DualAxisProcessor> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
    }
}

impl TypePath for Box<DualAxisProcessor> {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            {
                format!("std::boxed::Box<{}::DualAxisProcessor>", module_path!())
            }
        })
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| "Box<DualAxisProcessor>".to_string())
    }

    fn type_ident() -> Option<&'static str> {
        Some("Box<DualAxisProcessor>")
    }

    fn crate_name() -> Option<&'static str> {
        Some(module_path!().split(':').next().unwrap())
    }

    fn module_path() -> Option<&'static str> {
        Some(module_path!())
    }
}

impl GetTypeRegistration for Box<DualAxisProcessor> {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for Box<DualAxisProcessor> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.as_any().downcast_ref::<Self>()?.clone())
    }
}
