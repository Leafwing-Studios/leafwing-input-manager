//! Helpful abstractions over user inputs of all sorts.
//!
//! This module simplifies user input handling in Bevy applications
//! by providing abstractions and utilities for various input devices
//! like gamepads, keyboards, and mice. It offers a unified interface
//! for querying input values and states, reducing boilerplate code
//! and making user interactions easier to manage.
//!
//! The foundation of this module lies in the [`UserInput`] trait,
//! used to define the behavior expected from a specific user input source.
//!
//! Need something specific? You can also create your own inputs by implementing the trait for specific needs.
//!
//! Feel free to suggest additions to the built-in inputs if you have a common use case!
//!
//! ## Control Types
//!
//! [`UserInput`]s use the method [`UserInput::kind`] returning an [`InputControlKind`]
//! to classify the behavior of the input (buttons, analog axes, etc.).
//!
//! - [`InputControlKind::Button`]: Represents a digital input with an on/off state (e.g., button press).
//!   These inputs typically provide two values, typically `0.0` (inactive) and `1.0` (fully active).
//!
//! - [`InputControlKind::Axis`]: Represents an analog input (e.g., mouse wheel)
//!   with a continuous value typically ranging from `-1.0` (fully left/down) to `1.0` (fully right/up).
//!   Non-zero values are considered active.
//!
//! - [`InputControlKind::DualAxis`]: Represents a combination of two analog axes (e.g., thumb stick).
//!   These inputs provide separate X and Y values typically ranging from `-1.0` to `1.0`.
//!   Non-zero values are considered active.
//!
//! ## Basic Inputs
//!
//! [`UserInput`]s use the method [`UserInput::decompose`] returning a [`BasicInputs`]
//! used for clashing detection, see [clashing input check](crate::clashing_inputs) for details.
//!
//! ## Raw Input Events
//!
//! [`UserInput`]s use the method [`UserInput::raw_inputs`] returning a [`RawInputs`]
//! used for sending fake input events, see [input mocking](crate::input_mocking::MockInput) for details.
//!
//! ## Built-in Inputs
//!
//! ### Gamepad Inputs
//!
//! - Check gamepad button presses using Bevy's [`GamepadButtonType`] directly.
//! - Access physical sticks using [`GamepadStick`], [`GamepadControlAxis`], and [`GamepadControlDirection`].
//!
//! ### Keyboard Inputs
//!
//! - Check physical keys presses using Bevy's [`KeyCode`] directly.
//! - Use [`ModifierKey`] to check for either left or right modifier keys is pressed.
//!
//! ### Mouse Inputs
//!
//! - Check mouse buttons presses using Bevy's [`MouseButton`] directly.
//! - Track mouse motion with [`MouseMove`], [`MouseMoveAxis`], and [`MouseMoveDirection`].
//! - Capture mouse wheel events with [`MouseScroll`], [`MouseScrollAxis`], and [`MouseScrollDirection`].
//!
//! ### Complex Composition
//!
//! - Combine multiple inputs into a virtual button using [`InputChord`].
//!   - Only active if all its inner inputs are active simultaneously.
//!   - Combine values from all inner single-axis inputs if available.
//!   - Retrieve values from the first encountered dual-axis input within the chord.
//!
//! - Create a virtual axis control:
//!   - [`GamepadVirtualAxis`] from two [`GamepadButtonType`]s.
//!   - [`KeyboardVirtualAxis`] from two [`KeyCode`]s.
//!
//! - Create a virtual directional pad (D-pad) for dual-axis control:
//!   - [`GamepadVirtualDPad`] from four [`GamepadButtonType`]s.
//!   - [`KeyboardVirtualDPad`] from four [`KeyCode`]s.
//!
//! [`GamepadAxisType`]: bevy::prelude::GamepadAxisType
//! [`GamepadButtonType`]: bevy::prelude::GamepadButtonType
//! [`KeyCode`]: bevy::prelude::KeyCode
//! [`MouseButton`]: bevy::prelude::MouseButton

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

use crate::axislike::DualAxisData;
use crate::clashing_inputs::BasicInputs;
use crate::input_streams::InputStreams;
use crate::raw_inputs::RawInputs;
use crate::typetag::RegisterTypeTag;

pub use self::chord::*;
pub use self::gamepad::*;
pub use self::keyboard::*;
pub use self::mouse::*;

pub mod chord;
pub mod gamepad;
pub mod keyboard;
pub mod mouse;

/// Classifies [`UserInput`]s based on their behavior (buttons, analog axes, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum InputControlKind {
    /// A single input with binary state (active or inactive), typically a button press (on or off).
    Button,

    /// A single analog or digital input, often used for range controls like a thumb stick on a gamepad or mouse wheel,
    /// providing a value within a min-max range.
    Axis,

    /// A combination of two axis-like inputs, often used for directional controls like a D-pad on a gamepad,
    /// providing separate values for the X and Y axes.
    DualAxis,
}

/// A trait for defining the behavior expected from different user input sources.
///
/// Implementers of this trait should provide methods for accessing and processing user input data.
///
/// # Examples
///
/// ```rust
/// use std::hash::{Hash, Hasher};
/// use bevy::prelude::*;
/// use bevy::math::FloatOrd;
/// use serde::{Deserialize, Serialize};
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::input_streams::InputStreams;
/// use leafwing_input_manager::axislike::{DualAxisType, DualAxisData};
/// use leafwing_input_manager::raw_inputs::RawInputs;
/// use leafwing_input_manager::clashing_inputs::BasicInputs;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
/// pub struct MouseScrollAlwaysFiveOnYAxis;
///
/// // Add this attribute for ensuring proper serialization and deserialization.
/// #[serde_typetag]
/// impl UserInput for MouseScrollAlwaysFiveOnYAxis {
///     fn kind(&self) -> InputControlKind {
///         // Returns the kind of input this represents.
///         //
///         // In this case, it represents an axial input.
///         InputControlKind::Axis
///     }
///
///     fn pressed(&self, input_streams: &InputStreams) -> bool {
///         // Checks if the input is currently active.
///         //
///         // Since this virtual mouse scroll always outputs a value,
///         // it will always return `true`.
///         true
///     }
///
///     fn value(&self, input_streams: &InputStreams) -> f32 {
///         // Gets the current value of the input as an `f32`.
///         //
///         // This input always represents a scroll of `5.0` on the Y-axis.
///         5.0
///     }
///
///     fn axis_pair(&self, input_streams: &InputStreams) -> Option<DualAxisData> {
///         // Gets the values of this input along the X and Y axes (if applicable).
///         //
///         // This input only represents movement on the Y-axis,
///         // so it returns `None`.
///         None
///     }
///
///     fn decompose(&self) -> BasicInputs {
///         // Gets the most basic form of this input for clashing input detection.
///         //
///         // This input is a simple, atomic unit,
///         // so it is returned as a `BasicInputs::Simple`.
///         BasicInputs::Simple(Box::new(*self))
///     }
///
///     fn raw_inputs(&self) -> RawInputs {
///         // Defines the raw input events used for simulating this input.
///         //
///         // This input simulates a mouse scroll event on the Y-axis.
///         RawInputs::from_mouse_scroll_axes([DualAxisType::Y])
///     }
/// }
///
/// // Remember to register your input - it will ensure everything works smoothly!
/// let mut app = App::new();
/// app.register_user_input::<MouseScrollAlwaysFiveOnYAxis>();
/// ```
pub trait UserInput:
    Send + Sync + Debug + DynClone + DynEq + DynHash + Reflect + erased_serde::Serialize
{
    /// Defines the kind of behavior that the input should be.
    fn kind(&self) -> InputControlKind;

    /// Returns the most basic inputs that make up this input.
    ///
    /// For inputs that represent a simple, atomic control,
    /// this method should always return a [`BasicInputs::Simple`] that only contains the input itself.
    fn decompose(&self) -> BasicInputs;

    /// Returns the raw input events that make up this input.
    fn raw_inputs(&self) -> RawInputs;
}

/// A trait used for buttonlike user inputs, which can be pressed or released.
pub trait Buttonlike: UserInput {
    /// Checks if the input is currently active.
    fn pressed(&self, input_streams: &InputStreams) -> bool;

    /// Checks if the input is currently inactive.
    fn released(&self, input_streams: &InputStreams) -> bool {
        !self.pressed(input_streams)
    }
}

/// A trait used for axis-like user inputs, which provide a continuous value.
pub trait Axislike: UserInput {
    /// Gets the current value of the input as an `f32`.
    fn value(&self, input_streams: &InputStreams) -> f32;
}

/// A trait used for dual-axis-like user inputs, which provide separate X and Y values.
pub trait DualAxislike: UserInput {
    /// Gets the values of this input along the X and Y axes (if applicable).
    fn axis_pair(&self, input_streams: &InputStreams) -> DualAxisData;
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
