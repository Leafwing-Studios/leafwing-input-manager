use crate::input_like::virtual_dpad::deserialize_boxed_button;
use crate::input_like::ButtonLike;
use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode};
use bevy::prelude::Reflect;
use bevy::reflect::utility::NonGenericTypeInfoCell;
use bevy::reflect::{ReflectMut, ReflectOwned, ReflectRef, TypeInfo, TypePath, Typed, ValueInfo};
use serde::{Deserialize, Serialize};
use std::any::Any;

/// A virtual Axis that you can get a value between -1 and 1 from.
#[derive(Debug, Clone, Serialize, Deserialize, TypePath)]
pub struct VirtualAxis {
    /// The input that represents the negative direction of this virtual axis
    #[serde(deserialize_with = "deserialize_boxed_button")]
    pub negative: Box<dyn ButtonLike>,
    /// The input that represents the positive direction of this virtual axis
    #[serde(deserialize_with = "deserialize_boxed_button")]
    pub positive: Box<dyn ButtonLike>,
}

impl PartialEq for VirtualAxis {
    fn eq(&self, other: &Self) -> bool {
        self.negative.eq(&other.negative) && self.positive.eq(&other.positive)
    }
}

impl Eq for VirtualAxis {}

impl VirtualAxis {
    /// Generates a [`VirtualAxis`] corresponding to the horizontal arrow keyboard keycodes
    pub fn horizontal_arrow_keys() -> VirtualAxis {
        VirtualAxis {
            negative: KeyCode::Left.into(),
            positive: KeyCode::Right.into(),
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the horizontal arrow keyboard keycodes
    pub fn vertical_arrow_keys() -> VirtualAxis {
        VirtualAxis {
            negative: KeyCode::Down.into(),
            positive: KeyCode::Up.into(),
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the `AD` keyboard keycodes.
    pub fn ad() -> VirtualAxis {
        VirtualAxis {
            negative: KeyCode::A.into(),
            positive: KeyCode::D.into(),
        }
    }

    /// Generates a [`VirtualAxis`] corresponding to the `WS` keyboard keycodes.
    pub fn ws() -> VirtualAxis {
        VirtualAxis {
            negative: KeyCode::S.into(),
            positive: KeyCode::W.into(),
        }
    }

    #[allow(clippy::doc_markdown)]
    /// Generates a [`VirtualAxis`] corresponding to the horizontal DPad buttons on a gamepad.
    pub fn horizontal_dpad() -> VirtualAxis {
        VirtualAxis {
            negative: GamepadButtonType::DPadLeft.into(),
            positive: GamepadButtonType::DPadRight.into(),
        }
    }

    #[allow(clippy::doc_markdown)]
    /// Generates a [`VirtualAxis`] corresponding to the vertical DPad buttons on a gamepad.
    pub fn vertical_dpad() -> VirtualAxis {
        VirtualAxis {
            negative: GamepadButtonType::DPadDown.into(),
            positive: GamepadButtonType::DPadUp.into(),
        }
    }
}

impl Typed for VirtualAxis {
    fn type_info() -> &'static TypeInfo
    where
        Self: Sized,
    {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
    }
}

impl Reflect for VirtualAxis {
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn get_represented_type_info(&self) -> Option<&'static TypeInfo> {
        Some(<Self as Typed>::type_info())
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
            panic!("Value is not a {}.", std::any::type_name::<Self>());
        }
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
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

    fn reflect_partial_eq(&self, value: &dyn Reflect) -> Option<bool> {
        let value = value.as_any();
        if let Some(value) = value.downcast_ref::<Self>() {
            Some(std::cmp::PartialEq::eq(self, value))
        } else {
            Some(false)
        }
    }
}
