use std::any::Any;
use std::fmt::Debug;
use std::sync::{LazyLock, RwLock};

use bevy::app::App;
use bevy::prelude::{FromReflect, Reflect, ReflectDeserialize, ReflectSerialize, TypePath, Vec2};
use bevy::reflect::utility::{GenericTypePathCell, NonGenericTypeInfoCell};
use bevy::reflect::{
    erased_serde, FromType, GetTypeRegistration, OpaqueInfo, PartialReflect, ReflectFromPtr,
    ReflectKind, ReflectMut, ReflectOwned, ReflectRef, TypeInfo, TypeRegistration, Typed,
};
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_flexitos::ser::require_erased_serialize_impl;
use serde_flexitos::{serialize_trait_object, Registry};

use crate::input_processing::DualAxisProcessor;
use crate::typetag::{InfallibleMapRegistry, RegisterTypeTag};

/// A trait for creating custom processor that handles dual-axis input values,
/// accepting a [`Vec2`] input and producing a [`Vec2`] output.
///
/// # Examples
///
/// ```rust
/// use std::hash::{Hash, Hasher};
/// use bevy::prelude::*;
/// use bevy::math::FloatOrd;
/// use serde::{Deserialize, Serialize};
/// use leafwing_input_manager::prelude::*;
///
/// /// Doubles the input, takes its absolute value,
/// /// and discards results that meet the specified condition on the X-axis.
/// // If your processor includes fields not implemented Eq and Hash,
/// // implementation is necessary as shown below.
/// // Otherwise, you can derive Eq and Hash directly.
/// #[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
/// pub struct DoubleAbsoluteValueThenRejectX(pub f32);
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[serde_typetag]
/// impl CustomDualAxisProcessor for DoubleAbsoluteValueThenRejectX {
///     fn process(&self, input_value: Vec2) -> Vec2 {
///         // Implement the logic just like you would in a normal function.
///
///         // You can use other processors within this function.
///         let value = DualAxisSensitivity::all(2.0).scale(input_value);
///
///         let value = value.abs();
///         let new_x = if value.x == self.0 {
///             0.0
///         } else {
///             value.x
///         };
///         Vec2::new(new_x, value.y)
///     }
/// }
///
/// // Unfortunately, manual implementation is required due to the float field.
/// impl Eq for DoubleAbsoluteValueThenRejectX {}
/// impl Hash for DoubleAbsoluteValueThenRejectX {
///     fn hash<H: Hasher>(&self, state: &mut H) {
///         // Encapsulate the float field for hashing.
///         FloatOrd(self.0).hash(state);
///     }
/// }
///
/// // Remember to register your processor - it will ensure everything works smoothly!
/// let mut app = App::new();
/// app.register_dual_axis_processor::<DoubleAbsoluteValueThenRejectX>();
///
/// // Now you can use it!
/// let processor = DoubleAbsoluteValueThenRejectX(4.0);
///
/// // Rejected X!
/// assert_eq!(processor.process(Vec2::splat(2.0)), Vec2::new(0.0, 4.0));
/// assert_eq!(processor.process(Vec2::splat(-2.0)), Vec2::new(0.0, 4.0));
///
/// // Others are just doubled absolute value.
/// assert_eq!(processor.process(Vec2::splat(6.0)), Vec2::splat(12.0));
/// assert_eq!(processor.process(Vec2::splat(4.0)), Vec2::splat(8.0));
/// assert_eq!(processor.process(Vec2::splat(0.0)), Vec2::splat(0.0));
/// assert_eq!(processor.process(Vec2::splat(-4.0)), Vec2::splat(8.0));
/// assert_eq!(processor.process(Vec2::splat(-6.0)), Vec2::splat(12.0));
///
/// // The ways to create a DualAxisProcessor.
/// let dual_axis_processor = DualAxisProcessor::Custom(Box::new(processor));
/// assert_eq!(dual_axis_processor, DualAxisProcessor::from(processor));
/// ```
pub trait CustomDualAxisProcessor:
    Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Computes the result by processing the `input_value`.
    fn process(&self, input_value: Vec2) -> Vec2;
}

impl<P: CustomDualAxisProcessor> From<P> for DualAxisProcessor {
    fn from(value: P) -> Self {
        Self::Custom(Box::new(value))
    }
}

dyn_clone::clone_trait_object!(CustomDualAxisProcessor);
dyn_eq::eq_trait_object!(CustomDualAxisProcessor);
dyn_hash::hash_trait_object!(CustomDualAxisProcessor);

impl PartialReflect for Box<dyn CustomDualAxisProcessor> {
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(Self::type_info())
    }

    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Opaque
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Opaque(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Opaque(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Opaque(self)
    }

    fn clone_value(&self) -> Box<dyn PartialReflect> {
        Box::new(self.clone())
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), bevy::reflect::ApplyError> {
        if let Some(value) = value.try_downcast_ref::<Self>() {
            *self = value.clone();
            Ok(())
        } else {
            Err(bevy::reflect::ApplyError::MismatchedTypes {
                from_type: self
                    .reflect_type_ident()
                    .unwrap_or_default()
                    .to_string()
                    .into_boxed_str(),
                to_type: self
                    .reflect_type_ident()
                    .unwrap_or_default()
                    .to_string()
                    .into_boxed_str(),
            })
        }
    }

    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }

    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Ok(self)
    }

    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        Some(self)
    }

    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        Some(self)
    }
}

impl Reflect for Box<dyn CustomDualAxisProcessor> {
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

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }
}

impl Typed for Box<dyn CustomDualAxisProcessor> {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl TypePath for Box<dyn CustomDualAxisProcessor> {
    fn type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| {
            {
                format!(
                    "std::boxed::Box<dyn {}::CustomDualAxisProcessor>",
                    module_path!()
                )
            }
        })
    }

    fn short_type_path() -> &'static str {
        static CELL: GenericTypePathCell = GenericTypePathCell::new();
        CELL.get_or_insert::<Self, _>(|| "Box<dyn CustomDualAxisProcessor>".to_string())
    }

    fn type_ident() -> Option<&'static str> {
        Some("Box<dyn CustomDualAxisProcessor>")
    }

    fn crate_name() -> Option<&'static str> {
        Some(module_path!().split(':').next().unwrap())
    }

    fn module_path() -> Option<&'static str> {
        Some(module_path!())
    }
}

impl GetTypeRegistration for Box<dyn CustomDualAxisProcessor> {
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
        registration
    }
}

impl FromReflect for Box<dyn CustomDualAxisProcessor> {
    fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
        Some(reflect.try_downcast_ref::<Self>()?.clone())
    }
}

impl Serialize for dyn CustomDualAxisProcessor + '_ {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Check that `CustomDualAxisProcessor` has `erased_serde::Serialize` as a super trait,
        // preventing infinite recursion at runtime.
        const fn __check_erased_serialize_super_trait<T: ?Sized + CustomDualAxisProcessor>() {
            require_erased_serialize_impl::<T>();
        }
        serialize_trait_object(serializer, self.reflect_short_type_path(), self)
    }
}

impl<'de> Deserialize<'de> for Box<dyn CustomDualAxisProcessor> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let registry = PROCESSOR_REGISTRY.read().unwrap();
        registry.deserialize_trait_object(deserializer)
    }
}

/// Registry of deserializers for [`CustomDualAxisProcessor`]s.
static PROCESSOR_REGISTRY: LazyLock<RwLock<InfallibleMapRegistry<dyn CustomDualAxisProcessor>>> =
    LazyLock::new(|| RwLock::new(InfallibleMapRegistry::new("CustomDualAxisProcessor")));

/// A trait for registering a specific [`CustomDualAxisProcessor`].
pub trait RegisterDualAxisProcessorExt {
    /// Registers the specified [`CustomDualAxisProcessor`].
    fn register_dual_axis_processor<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn CustomDualAxisProcessor> + GetTypeRegistration;
}

impl RegisterDualAxisProcessorExt for App {
    fn register_dual_axis_processor<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn CustomDualAxisProcessor> + GetTypeRegistration,
    {
        let mut registry = PROCESSOR_REGISTRY.write().unwrap();
        T::register_typetag(&mut registry);
        self.register_type::<T>();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as leafwing_input_manager;
    use leafwing_input_manager_macros::serde_typetag;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn test_custom_dual_axis_processor() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
        struct CustomDualAxisInverted;

        #[serde_typetag]
        impl CustomDualAxisProcessor for CustomDualAxisInverted {
            fn process(&self, input_value: Vec2) -> Vec2 {
                -input_value
            }
        }

        let mut app = App::new();
        app.register_dual_axis_processor::<CustomDualAxisInverted>();

        let custom: Box<dyn CustomDualAxisProcessor> = Box::new(CustomDualAxisInverted);
        assert_tokens(
            &custom,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("CustomDualAxisInverted"),
                Token::UnitStruct {
                    name: "CustomDualAxisInverted",
                },
                Token::MapEnd,
            ],
        );

        let processor = DualAxisProcessor::Custom(custom);
        assert_eq!(DualAxisProcessor::from(CustomDualAxisInverted), processor);

        for x in -300..300 {
            let x = x as f32 * 0.01;
            for y in -300..300 {
                let y = y as f32 * 0.01;
                let value = Vec2::new(x, y);

                assert_eq!(processor.process(value), -value);
                assert_eq!(CustomDualAxisInverted.process(value), -value);
            }
        }
    }
}
