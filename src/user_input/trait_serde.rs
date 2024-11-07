//! Serialization and deserialization of user input.

use std::sync::RwLock;

use bevy::app::App;
use bevy::reflect::GetTypeRegistration;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_flexitos::ser::require_erased_serialize_impl;
use serde_flexitos::{serialize_trait_object, Registry};

use super::{Axislike, Buttonlike, DualAxislike, TripleAxislike};
use crate::typetag::{InfallibleMapRegistry, RegisterTypeTag};

/// Registry of deserializers for [`Buttonlike`]s.
static mut BUTTONLIKE_REGISTRY: Lazy<RwLock<InfallibleMapRegistry<dyn Buttonlike>>> =
    Lazy::new(|| RwLock::new(InfallibleMapRegistry::new("Buttonlike")));

/// Registry of deserializers for [`Axislike`]s.
static mut AXISLIKE_REGISTRY: Lazy<RwLock<InfallibleMapRegistry<dyn Axislike>>> =
    Lazy::new(|| RwLock::new(InfallibleMapRegistry::new("Axislike")));

/// Registry of deserializers for [`DualAxislike`]s.
static mut DUAL_AXISLIKE_REGISTRY: Lazy<RwLock<InfallibleMapRegistry<dyn DualAxislike>>> =
    Lazy::new(|| RwLock::new(InfallibleMapRegistry::new("DualAxislike")));

/// Registry of deserializers for [`TripleAxislike`]s.
static mut TRIPLE_AXISLIKE_REGISTRY: Lazy<RwLock<InfallibleMapRegistry<dyn TripleAxislike>>> =
    Lazy::new(|| RwLock::new(InfallibleMapRegistry::new("TripleAxislike")));

/// A trait for registering inputs.
pub trait RegisterUserInput {
    /// Registers the specified [`Buttonlike`].
    fn register_buttonlike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn Buttonlike> + GetTypeRegistration;

    /// Registers the specified [`Axislike`].
    fn register_axislike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn Axislike> + GetTypeRegistration;

    /// Registers the specified [`DualAxislike`].
    fn register_dual_axislike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn DualAxislike> + GetTypeRegistration;

    /// Registers the specified [`TripleAxislike`].
    fn register_triple_axislike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn TripleAxislike> + GetTypeRegistration;
}

impl RegisterUserInput for App {
    fn register_buttonlike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn Buttonlike> + GetTypeRegistration,
    {
        let mut registry = unsafe { BUTTONLIKE_REGISTRY.write().unwrap() };
        T::register_typetag(&mut registry);
        self.register_type::<T>();
        self
    }

    fn register_axislike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn Axislike> + GetTypeRegistration,
    {
        let mut registry = unsafe { AXISLIKE_REGISTRY.write().unwrap() };
        T::register_typetag(&mut registry);
        self.register_type::<T>();
        self
    }

    fn register_dual_axislike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn DualAxislike> + GetTypeRegistration,
    {
        let mut registry = unsafe { DUAL_AXISLIKE_REGISTRY.write().unwrap() };
        T::register_typetag(&mut registry);
        self.register_type::<T>();
        self
    }

    fn register_triple_axislike_input<'de, T>(&mut self) -> &mut Self
    where
        T: RegisterTypeTag<'de, dyn TripleAxislike> + GetTypeRegistration,
    {
        let mut registry = unsafe { TRIPLE_AXISLIKE_REGISTRY.write().unwrap() };
        T::register_typetag(&mut registry);
        self.register_type::<T>();
        self
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
            // Check that `Buttonlike` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + Buttonlike>() {
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
            let registry = unsafe { BUTTONLIKE_REGISTRY.read().unwrap() };
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
            // Check that `Axislike` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + Axislike>() {
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
            let registry = unsafe { AXISLIKE_REGISTRY.read().unwrap() };
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
            // Check that `DualAxislike` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + DualAxislike>() {
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
            let registry = unsafe { DUAL_AXISLIKE_REGISTRY.read().unwrap() };
            registry.deserialize_trait_object(deserializer)
        }
    }
}

mod tripleaxislike {
    use crate::user_input::TripleAxislike;

    use super::*;

    impl<'a> Serialize for dyn TripleAxislike + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // Check that `TripleAxislike` has `erased_serde::Serialize` as a super trait,
            // preventing infinite recursion at runtime.
            const fn __check_erased_serialize_super_trait<T: ?Sized + TripleAxislike>() {
                require_erased_serialize_impl::<T>();
            }
            serialize_trait_object(serializer, self.reflect_short_type_path(), self)
        }
    }

    impl<'de> Deserialize<'de> for Box<dyn TripleAxislike> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let registry = unsafe { TRIPLE_AXISLIKE_REGISTRY.read().unwrap() };
            registry.deserialize_trait_object(deserializer)
        }
    }
}

#[cfg(any(feature = "keyboard", feature = "mouse"))]
#[cfg(test)]
mod tests {
    use crate as leafwing_input_manager;
    use bevy::prelude::{App, Reflect};
    use leafwing_input_manager_macros::Actionlike;

    #[derive(Actionlike, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
    pub enum Action {
        Foo,
    }

    fn register_input_deserializers() {
        let mut app = App::new();

        // Add the plugin to register input deserializers
        app.add_plugins(crate::prelude::InputManagerPlugin::<Action>::default());
    }

    #[cfg(feature = "keyboard")]
    #[test]
    fn test_button_serde() {
        use crate::prelude::Buttonlike;
        use bevy::prelude::KeyCode;
        use serde_test::{assert_tokens, Token};

        register_input_deserializers();

        let boxed_input: Box<dyn Buttonlike> = Box::new(KeyCode::KeyB);
        assert_tokens(
            &boxed_input,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyB",
                },
                Token::MapEnd,
            ],
        );
    }

    #[cfg(feature = "mouse")]
    #[test]
    fn test_mouse_button_serde() {
        use bevy::prelude::MouseButton;
        use serde_test::{assert_tokens, Token};

        use crate::prelude::Buttonlike;

        register_input_deserializers();

        let boxed_input: Box<dyn Buttonlike> = Box::new(MouseButton::Left);
        assert_tokens(
            &boxed_input,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("MouseButton"),
                Token::UnitVariant {
                    name: "MouseButton",
                    variant: "Left",
                },
                Token::MapEnd,
            ],
        );
    }

    #[cfg(feature = "mouse")]
    #[test]
    fn test_axis_serde() {
        use crate::prelude::{Axislike, MouseScrollAxis};
        use serde_test::{assert_tokens, Token};

        register_input_deserializers();

        let boxed_input: Box<dyn Axislike> = Box::new(MouseScrollAxis::Y);
        assert_tokens(
            &boxed_input,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("MouseScrollAxis"),
                Token::Struct {
                    name: "MouseScrollAxis",
                    len: 2,
                },
                Token::BorrowedStr("axis"),
                Token::Enum {
                    name: "DualAxisType",
                },
                Token::Str("Y"),
                Token::Unit,
                Token::BorrowedStr("processors"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::MapEnd,
            ],
        );
    }

    #[cfg(feature = "mouse")]
    #[test]
    fn test_dual_axis_serde() {
        use crate::prelude::{DualAxislike, MouseMove};
        use serde_test::{assert_tokens, Token};

        register_input_deserializers();

        let boxed_input: Box<dyn DualAxislike> = Box::new(MouseMove::default());
        assert_tokens(
            &boxed_input,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("MouseMove"),
                Token::Struct {
                    name: "MouseMove",
                    len: 1,
                },
                Token::Str("processors"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::StructEnd,
                Token::MapEnd,
            ],
        );
    }

    #[cfg(feature = "keyboard")]
    #[test]
    fn test_triple_axis_serde() {
        use crate::prelude::{TripleAxislike, VirtualDPad3D};
        use bevy::prelude::KeyCode;
        use serde_test::{assert_tokens, Token};

        register_input_deserializers();

        let boxed_input: Box<dyn TripleAxislike> = Box::new(VirtualDPad3D::new(
            KeyCode::KeyW,
            KeyCode::KeyS,
            KeyCode::KeyA,
            KeyCode::KeyD,
            KeyCode::KeyF,
            KeyCode::KeyB,
        ));
        assert_tokens(
            &boxed_input,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("VirtualDPad3D"),
                Token::Struct {
                    name: "VirtualDPad3D",
                    len: 6,
                },
                Token::Str("up"),
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyW",
                },
                Token::MapEnd,
                Token::Str("down"),
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyS",
                },
                Token::MapEnd,
                Token::Str("left"),
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyA",
                },
                Token::MapEnd,
                Token::Str("right"),
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyD",
                },
                Token::MapEnd,
                Token::Str("forward"),
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyF",
                },
                Token::MapEnd,
                Token::Str("backward"),
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "KeyB",
                },
                Token::MapEnd,
                Token::StructEnd,
                Token::MapEnd,
            ],
        );
    }
}
