//! Helpful abstractions over user inputs of all sorts.
//!
//! This module provides abstractions and utilities for defining and handling user inputs
//! across various input devices such as gamepads, keyboards, and mice.
//! It offers a unified interface for querying input values and states,
//! making it easier to manage and process user interactions within a Bevy application.
//!
//! # Traits
//!
//! - [`UserInput`]: A trait for defining a specific kind of user input.
//!   It provides methods for checking if the input is active,
//!   retrieving its current value, and detecting when it started or finished.
//!
//! # Modules
//!
//! ## General Input Settings
//!
//! - [`axislike_processors`]: Utilities for configuring axis-like inputs.
//!
//! ## General Inputs
//!
//! - [`gamepad`]: Utilities for handling gamepad inputs.
//! - [`keyboard`]: Utilities for handling keyboard inputs.
//! - [`mouse`]: Utilities for handling mouse inputs.
//!
//! ## Specific Inputs
//!
//! - [`chord`]: A combination of buttons, pressed simultaneously.

use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::RwLock;

use bevy::prelude::{App, Vec2};
use bevy::reflect::utility::{reflect_hasher, GenericTypePathCell, NonGenericTypeInfoCell};
use bevy::reflect::{
    erased_serde, DynamicTypePath, FromReflect, FromType, GetTypeRegistration, Reflect,
    ReflectDeserialize, ReflectFromPtr, ReflectKind, ReflectMut, ReflectOwned, ReflectRef,
    ReflectSerialize, TypeInfo, TypePath, TypeRegistration, Typed, ValueInfo,
};
use dyn_clone::DynClone;
use dyn_eq::DynEq;
use dyn_hash::DynHash;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_flexitos::ser::require_erased_serialize_impl;
use serde_flexitos::{serialize_trait_object, MapRegistry, Registry};

use crate::input_streams::InputStreams;
use crate::typetag::RegisterTypeTag;

pub mod chord;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;

#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub enum InputKind {
    Button,
    Axis,
    DualAxis,
}

/// A trait for defining the behavior expected from different user input sources.
///
/// Implementers of this trait should provide methods for accessing and
/// processing user input data.
pub trait UserInput:
    Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Checks if the user input is currently active.
    fn is_active(&self, input_streams: &InputStreams) -> bool;

    /// Retrieves the current value of the user input.
    fn value(&self, input_streams: &InputStreams) -> f32;

    /// Retrieves the current dual-axis value of the user input if available.
    fn axis_pair(&self, input_streams: &InputStreams) -> Option<Vec2>;
}

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

    fn apply(&mut self, value: &dyn Reflect) {
        let value = value.as_any();
        if let Some(value) = value.downcast_ref::<Self>() {
            *self = value.clone();
        } else {
            panic!(
                "Value is not a std::boxed::Box<dyn {}::UserInput>.",
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
