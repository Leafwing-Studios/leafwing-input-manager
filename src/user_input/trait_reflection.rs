//! Implementations of the various [`bevy::reflect`] traits required to make our types reflectable.
//!
//! Note that [bevy #3392](https://github.com/bevyengine/bevy/issues/3392) would eliminate the need for this.

use std::any::Any;

use bevy::reflect::{
    utility::{GenericTypePathCell, NonGenericTypeInfoCell},
    FromReflect, FromType, GetTypeRegistration, OpaqueInfo, Reflect, ReflectDeserialize,
    ReflectFromPtr, ReflectKind, ReflectMut, ReflectOwned, ReflectRef, ReflectSerialize, TypeInfo,
    TypePath, TypeRegistration, Typed,
};

use dyn_eq::DynEq;

mod buttonlike {
    use bevy::reflect::PartialReflect;

    use super::*;

    use crate::user_input::Buttonlike;

    dyn_clone::clone_trait_object!(Buttonlike);
    dyn_eq::eq_trait_object!(Buttonlike);
    dyn_hash::hash_trait_object!(Buttonlike);

    impl PartialReflect for Box<dyn Buttonlike> {
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

        fn try_apply(
            &mut self,
            value: &dyn PartialReflect,
        ) -> Result<(), bevy::reflect::ApplyError> {
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

    impl Reflect for Box<dyn Buttonlike> {
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

    impl Typed for Box<dyn Buttonlike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
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
            module_path!().split(':').next()
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
        fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
            Some(reflect.try_downcast_ref::<Self>()?.clone())
        }
    }
}

mod axislike {
    use bevy::reflect::PartialReflect;

    use super::*;

    use crate::user_input::Axislike;

    dyn_clone::clone_trait_object!(Axislike);
    dyn_eq::eq_trait_object!(Axislike);
    dyn_hash::hash_trait_object!(Axislike);

    impl PartialReflect for Box<dyn Axislike> {
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

        fn try_apply(
            &mut self,
            value: &dyn PartialReflect,
        ) -> Result<(), bevy::reflect::ApplyError> {
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

    impl Reflect for Box<dyn Axislike> {
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

    impl Typed for Box<dyn Axislike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
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
            module_path!().split(':').next()
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
        fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
            Some(reflect.try_downcast_ref::<Self>()?.clone())
        }
    }
}

mod dualaxislike {
    use bevy::reflect::{OpaqueInfo, PartialReflect};

    use super::*;

    use crate::user_input::DualAxislike;

    dyn_clone::clone_trait_object!(DualAxislike);
    dyn_eq::eq_trait_object!(DualAxislike);
    dyn_hash::hash_trait_object!(DualAxislike);

    impl PartialReflect for Box<dyn DualAxislike> {
        fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
            Some(Self::type_info())
        }

        fn try_apply(
            &mut self,
            value: &dyn PartialReflect,
        ) -> Result<(), bevy::reflect::ApplyError> {
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

    impl Reflect for Box<dyn DualAxislike> {
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

    impl Typed for Box<dyn DualAxislike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
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
            module_path!().split(':').next()
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
        fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
            Some(reflect.try_downcast_ref::<Self>()?.clone())
        }
    }
}

mod tripleaxislike {
    use bevy::reflect::{OpaqueInfo, PartialReflect};

    use super::*;

    use crate::user_input::TripleAxislike;

    dyn_clone::clone_trait_object!(TripleAxislike);
    dyn_eq::eq_trait_object!(TripleAxislike);
    dyn_hash::hash_trait_object!(TripleAxislike);

    impl PartialReflect for Box<dyn TripleAxislike> {
        fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
            Some(Self::type_info())
        }

        fn try_apply(
            &mut self,
            value: &dyn PartialReflect,
        ) -> Result<(), bevy::reflect::ApplyError> {
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

    impl Reflect for Box<dyn TripleAxislike> {
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

    impl Typed for Box<dyn TripleAxislike> {
        fn type_info() -> &'static TypeInfo {
            static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
            CELL.get_or_set(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
        }
    }

    impl TypePath for Box<dyn TripleAxislike> {
        fn type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| {
                {
                    format!("std::boxed::Box<dyn {}::TripleAxislike>", module_path!())
                }
            })
        }

        fn short_type_path() -> &'static str {
            static CELL: GenericTypePathCell = GenericTypePathCell::new();
            CELL.get_or_insert::<Self, _>(|| "Box<dyn TripleAxislike>".to_string())
        }

        fn type_ident() -> Option<&'static str> {
            Some("Box<dyn TripleAxislike>")
        }

        fn crate_name() -> Option<&'static str> {
            module_path!().split(':').next()
        }

        fn module_path() -> Option<&'static str> {
            Some(module_path!())
        }
    }

    impl GetTypeRegistration for Box<dyn TripleAxislike> {
        fn get_type_registration() -> TypeRegistration {
            let mut registration = TypeRegistration::of::<Self>();
            registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
            registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
            registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
            registration
        }
    }

    impl FromReflect for Box<dyn TripleAxislike> {
        fn from_reflect(reflect: &dyn PartialReflect) -> Option<Self> {
            Some(reflect.try_downcast_ref::<Self>()?.clone())
        }
    }
}
