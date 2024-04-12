//! Processors for single-axis input values

use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use bevy::prelude::App;
use bevy::reflect::utility::{reflect_hasher, GenericTypePathCell, NonGenericTypeInfoCell};
use bevy::reflect::{
    erased_serde, FromReflect, FromType, GetTypeRegistration, Reflect, ReflectDeserialize,
    ReflectFromPtr, ReflectKind, ReflectMut, ReflectOwned, ReflectRef, ReflectSerialize, TypeInfo,
    TypePath, TypeRegistration, Typed, ValueInfo,
};
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_flexitos::ser::require_erased_serialize_impl;
use serde_flexitos::{serialize_trait_object, MapRegistry, Registry};

use crate::typetag::RegisterTypeTag;

pub use self::modifier::*;
pub use self::pipeline::*;
pub use self::range::*;

mod modifier;
mod pipeline;
mod range;

/// A trait for processing single-axis input values,
/// accepting a `f32` input and producing a `f32` output.
///
/// # Examples
///
/// ```rust
/// use std::hash::{Hash, Hasher};
/// use bevy::prelude::*;
/// use bevy::utils::FloatOrd;
/// use serde::{Deserialize, Serialize};
/// use leafwing_input_manager::prelude::*;
///
/// /// Doubles the input, takes the absolute value,
/// /// and discards results that meet the specified condition.
/// // If your processor includes fields not implemented Eq and Hash,
/// // implementation is necessary as shown below.
/// // Otherwise, you can derive Eq and Hash directly.
/// #[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
/// pub struct DoubleAbsoluteValueThenIgnored(pub f32);
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[serde_typetag]
/// impl AxisProcessor for DoubleAbsoluteValueThenIgnored {
///     fn process(&self, input_value: f32) -> f32 {
///         // Implement the logic just like you would in a normal function.
///
///         // You can use other processors within this function.
///         let value = AxisSensitivity(2.0).process(input_value);
///
///         let value = value.abs();
///         if value == self.0 {
///             0.0
///         } else {
///             value
///         }
///     }
/// }
///
/// // Unfortunately, manual implementation is required due to the float field.
/// impl Eq for DoubleAbsoluteValueThenIgnored {}
/// impl Hash for DoubleAbsoluteValueThenIgnored {
///     fn hash<H: Hasher>(&self, state: &mut H) {
///         // Encapsulate the float field for hashing.
///         FloatOrd(self.0).hash(state);
///     }
/// }
///
/// // Now you can use it!
/// let processor = DoubleAbsoluteValueThenIgnored(4.0);
///
/// // Rejected!
/// assert_eq!(processor.process(2.0), 0.0);
/// assert_eq!(processor.process(-2.0), 0.0);
///
/// // Others are just doubled absolute value.
/// assert_eq!(processor.process(6.0), 12.0);
/// assert_eq!(processor.process(4.0), 8.0);
/// assert_eq!(processor.process(0.0), 0.0);
/// assert_eq!(processor.process(-4.0), 8.0);
/// assert_eq!(processor.process(-6.0), 12.0);
/// ```
pub trait AxisProcessor:
    Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Computes the result by processing the `input_value`.
    fn process(&self, input_value: f32) -> f32;
}

dyn_clone::clone_trait_object!(AxisProcessor);
dyn_eq::eq_trait_object!(AxisProcessor);
dyn_hash::hash_trait_object!(AxisProcessor);

impl Reflect for Box<dyn AxisProcessor> {
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
            *self = value.clone();
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
        let type_id = TypeId::of::<Box<dyn AxisProcessor>>();
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

impl Typed for Box<dyn AxisProcessor> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
    }
}

impl TypePath for Box<dyn AxisProcessor> {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            {
                format!("std::boxed::Box(dyn {}::AxisProcessor)", module_path!())
            }
        })
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| "Box(dyn AxisProcessor)".to_string())
    }

    fn type_ident() -> Option<&'static str> {
        Some("AxisProcessor")
    }

    fn crate_name() -> Option<&'static str> {
        Some(module_path!().split(':').next().unwrap())
    }

    fn module_path() -> Option<&'static str> {
        Some(module_path!())
    }
}

impl GetTypeRegistration for Box<dyn AxisProcessor> {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for Box<dyn AxisProcessor> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        Some(reflect.as_any().downcast_ref::<Self>()?.clone())
    }
}

impl<'a> Serialize for dyn AxisProcessor + 'a {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Check that `ExampleObj` has `erased_serde::Serialize` as a supertrait, preventing infinite recursion at runtime.
        const fn __check_erased_serialize_supertrait<T: ?Sized + AxisProcessor>() {
            require_erased_serialize_impl::<T>();
        }
        serialize_trait_object(serializer, self.reflect_short_type_path(), self)
    }
}

impl<'de> Deserialize<'de> for Box<dyn AxisProcessor> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let registry = unsafe { PROCESSOR_REGISTRY.read().unwrap() };
        registry.deserialize_trait_object(deserializer)
    }
}

/// Registry of deserializers for [`AxisProcessor`]s.
static mut PROCESSOR_REGISTRY: Lazy<RwLock<MapRegistry<dyn AxisProcessor>>> =
    Lazy::new(|| RwLock::new(MapRegistry::new("AxisProcessor")));

/// A trait for registering a specific [`AxisProcessor`].
pub trait RegisterAxisProcessor {
    /// Registers the specified [`AxisProcessor`].
    fn register_axis_processor<'de, T: RegisterTypeTag<'de, dyn AxisProcessor>>(
        &mut self,
    ) -> &mut Self;
}

impl RegisterAxisProcessor for App {
    #[allow(unsafe_code)]
    fn register_axis_processor<'de, T: RegisterTypeTag<'de, dyn AxisProcessor>>(
        &mut self,
    ) -> &mut Self {
        let mut registry = unsafe { PROCESSOR_REGISTRY.write().unwrap() };
        T::register_typetag(&mut registry);
        self
    }
}
