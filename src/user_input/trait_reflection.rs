//! Implementations of the various [`bevy::reflect`] traits required to make our types reflectable.
//!
//! Note that [bevy #3392](https://github.com/bevyengine/bevy/issues/3392) would eliminate the need for this.

use std::{
    any::{Any, TypeId},
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
};

use bevy::reflect::{
    utility::{reflect_hasher, GenericTypePathCell, NonGenericTypeInfoCell},
    FromReflect, FromType, GetTypeRegistration, Reflect, ReflectDeserialize, ReflectFromPtr,
    ReflectKind, ReflectMut, ReflectOwned, ReflectRef, ReflectSerialize, TypeInfo, TypePath,
    TypeRegistration, Typed, ValueInfo,
};

use dyn_eq::DynEq;

mod user_input {
    use super::*;

    use crate::user_input::UserInput;

    dyn_clone::clone_trait_object!(UserInput);
    dyn_eq::eq_trait_object!(UserInput);
    dyn_hash::hash_trait_object!(UserInput);

    impl Reflect for Box<dyn UserInput> {
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

        fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), bevy::reflect::ApplyError> {
            let value = value.as_any();
            if let Some(value) = value.downcast_ref::<Self>() {
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

        fn apply(&mut self, value: &dyn Reflect) {
            Self::try_apply(self, value).unwrap();
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
            value
                .as_any()
                .downcast_ref::<Self>()
                .map(|value| self.dyn_eq(value))
                .or(Some(false))
        }

        fn debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self, f)
        }
    }

    impl Typed for Box<dyn UserInput> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
        }
    }

    impl TypePath for Box<dyn UserInput> {
        fn type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| {
                {
                    format!("std::boxed::Box<dyn {}::UserInput>", module_path!())
                }
            })
        }

        fn short_type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| "Box<dyn UserInput>".to_string())
        }

        fn type_ident() -> Option<&'static str> {
            Some("Box<dyn UserInput>")
        }

        fn crate_name() -> Option<&'static str> {
            Some(module_path!().split(':').next().unwrap())
        }

        fn module_path() -> Option<&'static str> {
            Some(module_path!())
        }
    }

    impl GetTypeRegistration for Box<dyn UserInput> {
        fn get_type_registration() -> TypeRegistration {
            let mut registration = TypeRegistration::of::<Self>();
            registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
            registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
            registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
            registration
        }
    }

    impl FromReflect for Box<dyn UserInput> {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            Some(reflect.as_any().downcast_ref::<Self>()?.clone())
        }
    }
}

mod buttonlike {
    use super::*;

    use crate::user_input::Buttonlike;

    dyn_clone::clone_trait_object!(Buttonlike);
    dyn_eq::eq_trait_object!(Buttonlike);
    dyn_hash::hash_trait_object!(Buttonlike);

    impl Reflect for Box<dyn Buttonlike> {
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

        fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), bevy::reflect::ApplyError> {
            let value = value.as_any();
            if let Some(value) = value.downcast_ref::<Self>() {
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

        fn apply(&mut self, value: &dyn Reflect) {
            Self::try_apply(self, value).unwrap();
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
            value
                .as_any()
                .downcast_ref::<Self>()
                .map(|value| self.dyn_eq(value))
                .or(Some(false))
        }

        fn debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self, f)
        }
    }

    impl Typed for Box<dyn Buttonlike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
        }
    }

    impl TypePath for Box<dyn Buttonlike> {
        fn type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| {
                {
                    format!("std::boxed::Box<dyn {}::Buttonlike>", module_path!())
                }
            })
        }

        fn short_type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| "Box<dyn Buttonlike>".to_string())
        }

        fn type_ident() -> Option<&'static str> {
            Some("Box<dyn Buttonlike>")
        }

        fn crate_name() -> Option<&'static str> {
            Some(module_path!().split(':').next().unwrap())
        }

        fn module_path() -> Option<&'static str> {
            Some(module_path!())
        }
    }

    impl GetTypeRegistration for Box<dyn Buttonlike> {
        fn get_type_registration() -> TypeRegistration {
            let mut registration = TypeRegistration::of::<Self>();
            registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
            registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
            registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
            registration
        }
    }

    impl FromReflect for Box<dyn Buttonlike> {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            Some(reflect.as_any().downcast_ref::<Self>()?.clone())
        }
    }
}

mod axislike {
    use super::*;

    use crate::user_input::Axislike;

    dyn_clone::clone_trait_object!(Axislike);
    dyn_eq::eq_trait_object!(Axislike);
    dyn_hash::hash_trait_object!(Axislike);

    impl Reflect for Box<dyn Axislike> {
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

        fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), bevy::reflect::ApplyError> {
            let value = value.as_any();
            if let Some(value) = value.downcast_ref::<Self>() {
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

        fn apply(&mut self, value: &dyn Reflect) {
            Self::try_apply(self, value).unwrap();
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
            value
                .as_any()
                .downcast_ref::<Self>()
                .map(|value| self.dyn_eq(value))
                .or(Some(false))
        }

        fn debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self, f)
        }
    }

    impl Typed for Box<dyn Axislike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
        }
    }

    impl TypePath for Box<dyn Axislike> {
        fn type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| {
                {
                    format!("std::boxed::Box<dyn {}::Axislike>", module_path!())
                }
            })
        }

        fn short_type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| "Box<dyn Axislike>".to_string())
        }

        fn type_ident() -> Option<&'static str> {
            Some("Box<dyn Axislike>")
        }

        fn crate_name() -> Option<&'static str> {
            Some(module_path!().split(':').next().unwrap())
        }

        fn module_path() -> Option<&'static str> {
            Some(module_path!())
        }
    }

    impl GetTypeRegistration for Box<dyn Axislike> {
        fn get_type_registration() -> TypeRegistration {
            let mut registration = TypeRegistration::of::<Self>();
            registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
            registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
            registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
            registration
        }
    }

    impl FromReflect for Box<dyn Axislike> {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            Some(reflect.as_any().downcast_ref::<Self>()?.clone())
        }
    }
}

mod dualaxislike {
    use super::*;

    use crate::user_input::DualAxislike;

    dyn_clone::clone_trait_object!(DualAxislike);
    dyn_eq::eq_trait_object!(DualAxislike);
    dyn_hash::hash_trait_object!(DualAxislike);

    impl Reflect for Box<dyn DualAxislike> {
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

        fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), bevy::reflect::ApplyError> {
            let value = value.as_any();
            if let Some(value) = value.downcast_ref::<Self>() {
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

        fn apply(&mut self, value: &dyn Reflect) {
            Self::try_apply(self, value).unwrap();
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
            value
                .as_any()
                .downcast_ref::<Self>()
                .map(|value| self.dyn_eq(value))
                .or(Some(false))
        }

        fn debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            Debug::fmt(self, f)
        }
    }

    impl Typed for Box<dyn DualAxislike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
        }
    }

    impl TypePath for Box<dyn DualAxislike> {
        fn type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| {
                {
                    format!("std::boxed::Box<dyn {}::DualAxislike>", module_path!())
                }
            })
        }

        fn short_type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| "Box<dyn DualAxislike>".to_string())
        }

        fn type_ident() -> Option<&'static str> {
            Some("Box<dyn DualAxislike>")
        }

        fn crate_name() -> Option<&'static str> {
            Some(module_path!().split(':').next().unwrap())
        }

        fn module_path() -> Option<&'static str> {
            Some(module_path!())
        }
    }

    impl GetTypeRegistration for Box<dyn DualAxislike> {
        fn get_type_registration() -> TypeRegistration {
            let mut registration = TypeRegistration::of::<Self>();
            registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
            registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
            registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
            registration
        }
    }

    impl FromReflect for Box<dyn DualAxislike> {
        fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
            Some(reflect.as_any().downcast_ref::<Self>()?.clone())
        }
    }
}
