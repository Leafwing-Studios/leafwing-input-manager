/// Implements the Bevy's [`Reflect`](bevy::reflect::Reflect) for boxed trait objects.
#[doc(hidden)]
#[macro_export]
macro_rules! __reflect_trait_object {
    ($ObjectTrait:ident) => {
        impl ::bevy::reflect::Reflect for ::std::boxed::Box<dyn $ObjectTrait> {
            fn get_represented_type_info(
                &self,
            ) -> ::core::option::Option<&'static ::bevy::reflect::TypeInfo> {
                ::core::option::Option::Some(<Self as ::bevy::reflect::Typed>::type_info())
            }

            fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn ::core::any::Any> {
                self
            }

            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn ::core::any::Any {
                self
            }

            fn into_reflect(
                self: ::std::boxed::Box<Self>,
            ) -> ::std::boxed::Box<dyn ::bevy::reflect::Reflect> {
                self
            }

            fn as_reflect(&self) -> &dyn ::bevy::reflect::Reflect {
                self
            }

            fn as_reflect_mut(&mut self) -> &mut dyn ::bevy::reflect::Reflect {
                self
            }

            fn apply(&mut self, value: &dyn ::bevy::reflect::Reflect) {
                let value = value.as_any();
                if let ::core::option::Option::Some(value) = value.downcast_ref::<Self>() {
                    *self = value.clone();
                } else {
                    panic!(
                        "Value is not a std::boxed::Box<dyn {}::{}>.",
                        module_path!(),
                        stringify!($ObjectTrait),
                    );
                }
            }

            fn set(
                &mut self,
                value: ::std::boxed::Box<dyn ::bevy::reflect::Reflect>,
            ) -> Result<(), ::std::boxed::Box<dyn bevy::reflect::Reflect>> {
                *self = value.take()?;
                ::core::result::Result::Ok(())
            }

            fn reflect_kind(&self) -> ::bevy::reflect::ReflectKind {
                ::bevy::reflect::ReflectKind::Value
            }

            fn reflect_ref(&self) -> ::bevy::reflect::ReflectRef {
                ::bevy::reflect::ReflectRef::Value(self)
            }

            fn reflect_mut(&mut self) -> ::bevy::reflect::ReflectMut {
                ::bevy::reflect::ReflectMut::Value(self)
            }

            fn reflect_owned(self: ::std::boxed::Box<Self>) -> ::bevy::reflect::ReflectOwned {
                ::bevy::reflect::ReflectOwned::Value(self)
            }

            fn clone_value(&self) -> ::std::boxed::Box<dyn bevy::reflect::Reflect> {
                ::std::boxed::Box::new(self.clone())
            }

            fn reflect_hash(&self) -> ::core::option::Option<u64> {
                let mut hasher = ::bevy::reflect::utility::reflect_hasher();
                ::core::hash::Hash::hash(&::core::any::Any::type_id(self), &mut hasher);
                ::core::hash::Hash::hash(self, &mut hasher);
                let result = ::core::hash::Hasher::finish(&hasher);
                ::core::option::Option::Some(result)
            }

            fn reflect_partial_eq(
                &self,
                value: &dyn ::bevy::reflect::Reflect,
            ) -> ::core::option::Option<bool> {
                let value = value.as_any();
                value
                    .downcast_ref::<Self>()
                    .map(|value| self.dyn_eq(value))
                    .or(::core::option::Option::Some(false))
            }

            fn debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(self, f)
            }
        }

        impl ::bevy::reflect::Typed for ::std::boxed::Box<dyn $ObjectTrait> {
            fn type_info() -> &'static ::bevy::reflect::TypeInfo {
                static CELL: ::bevy::reflect::utility::NonGenericTypeInfoCell =
                    ::bevy::reflect::utility::NonGenericTypeInfoCell::new();
                CELL.get_or_set(|| {
                    ::bevy::reflect::TypeInfo::Value(bevy::reflect::ValueInfo::new::<Self>())
                })
            }
        }

        impl ::bevy::reflect::TypePath for ::std::boxed::Box<dyn $ObjectTrait> {
            fn type_path() -> &'static str {
                static CELL: ::bevy::reflect::utility::GenericTypePathCell =
                    ::bevy::reflect::utility::GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    ::std::format!(
                        "std::boxed::Box(dyn {}::{})",
                        module_path!(),
                        stringify!($ObjectTrait),
                    )
                })
            }

            fn short_type_path() -> &'static str {
                static CELL: ::bevy::reflect::utility::GenericTypePathCell =
                    ::bevy::reflect::utility::GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    ::std::string::String::from(::core::concat!(
                        "Box(dyn ",
                        core::stringify!($ObjectTrait),
                        ")"
                    ))
                })
            }

            fn type_ident() -> ::core::option::Option<&'static str> {
                ::core::option::Option::Some(::core::stringify!($ObjectTrait))
            }

            fn crate_name() -> ::core::option::Option<&'static str> {
                ::core::option::Option::Some(::core::module_path!().split(':').next().unwrap())
            }

            fn module_path() -> ::core::option::Option<&'static str> {
                ::core::option::Option::Some(::core::module_path!())
            }
        }

        impl ::bevy::reflect::GetTypeRegistration for ::std::boxed::Box<dyn $ObjectTrait> {
            fn get_type_registration() -> ::bevy::reflect::TypeRegistration {
                use ::bevy::reflect::*;
                let mut registration = TypeRegistration::of::<Self>();
                registration.insert::<ReflectDeserialize>(FromType::<Self>::from_type());
                registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
                registration.insert::<ReflectSerialize>(FromType::<Self>::from_type());
                registration
            }
        }

        impl ::bevy::reflect::FromReflect for ::std::boxed::Box<dyn $ObjectTrait> {
            fn from_reflect(
                reflect: &dyn ::bevy::reflect::Reflect,
            ) -> ::core::option::Option<Self> {
                ::core::option::Option::Some(reflect.as_any().downcast_ref::<Self>()?.clone())
            }
        }
    };
}
