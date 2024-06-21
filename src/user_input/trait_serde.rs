//! Serialization and deserialization of user input.

use std::sync::RwLock;

use bevy::app::App;
use bevy::reflect::GetTypeRegistration;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_flexitos::ser::require_erased_serialize_impl;
use serde_flexitos::{serialize_trait_object, MapRegistry, Registry};

use crate::typetag::RegisterTypeTag;

use super::UserInput;

/// Registry of deserializers for [`UserInput`]s.
static mut INPUT_REGISTRY: Lazy<RwLock<MapRegistry<dyn UserInput>>> =
    Lazy::new(|| RwLock::new(MapRegistry::new("UserInput")));

/// A trait for registering a specific [`UserInput`].
pub trait RegisterUserInput {
    /// Registers the specified [`UserInput`].
    fn register_user_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn UserInput> + GetTypeRegistration;
}

impl RegisterUserInput for App {
    fn register_user_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn UserInput> + GetTypeRegistration,
    {
        let mut registry = unsafe { INPUT_REGISTRY.write().unwrap() };
        T::register_typetag(&mut registry);
        self.register_type::<T>();
        self
    }
}

mod user_input {
    use super::*;

    impl<'a> Serialize for dyn UserInput + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Check that `UserInput` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + UserInput>() {
                require_erased_serialize_impl::<T>();
            }
            serialize_trait_object(serializer, self.reflect_short_type_path(), self)
        }
    }

    impl<'de> Deserialize<'de> for Box<dyn UserInput> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let registry = unsafe { INPUT_REGISTRY.read().unwrap() };
            registry.deserialize_trait_object(deserializer)
        }
    }
}

mod buttonlike {
    use crate::user_input::Buttonlike;

    use super::*;

    impl<'a> Serialize for dyn Buttonlike + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Check that `UserInput` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + UserInput>() {
                require_erased_serialize_impl::<T>();
            }
            serialize_trait_object(serializer, self.reflect_short_type_path(), self)
        }
    }

    impl<'de> Deserialize<'de> for Box<dyn Buttonlike> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let registry = unsafe { INPUT_REGISTRY.read().unwrap() };
            registry.deserialize_trait_object(deserializer)
        }
    }
}

mod axislike {
    use crate::user_input::Axislike;

    use super::*;

    impl<'a> Serialize for dyn Axislike + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Check that `UserInput` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + UserInput>() {
                require_erased_serialize_impl::<T>();
            }
            serialize_trait_object(serializer, self.reflect_short_type_path(), self)
        }
    }

    impl<'de> Deserialize<'de> for Box<dyn Axislike> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let registry = unsafe { INPUT_REGISTRY.read().unwrap() };
            registry.deserialize_trait_object(deserializer)
        }
    }
}

mod dualaxislike {
    use crate::user_input::DualAxislike;

    use super::*;

    impl<'a> Serialize for dyn DualAxislike + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Check that `UserInput` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + UserInput>() {
                require_erased_serialize_impl::<T>();
            }
            serialize_trait_object(serializer, self.reflect_short_type_path(), self)
        }
    }

    impl<'de> Deserialize<'de> for Box<dyn DualAxislike> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let registry = unsafe { INPUT_REGISTRY.read().unwrap() };
            registry.deserialize_trait_object(deserializer)
        }
    }
}
