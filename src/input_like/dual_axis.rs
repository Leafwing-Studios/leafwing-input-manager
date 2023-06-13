use crate::axislike::DualAxisData;
use crate::input_like::mouse_motion_axis::MouseMotionAxis;
use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use crate::prelude::MouseWheelAxis;
use bevy::math::Vec2;
use bevy::prelude::{GamepadAxisType, Reflect, World};
use bevy::reflect::utility::NonGenericTypeInfoCell;
use bevy::reflect::{ReflectMut, ReflectOwned, ReflectRef, TypeInfo, Typed, ValueInfo};
use serde::{Deserialize, Deserializer, Serialize};
use std::any::Any;

/// Two [`SingleAxisLike`]s combined as one input that implements [`DualAxisLike`].
#[derive(Debug, Serialize, Deserialize)]
pub struct DualAxis {
    /// The axis representing horizontal movement.
    #[serde(deserialize_with = "deserialize_boxed_single_axis")]
    pub x: Box<dyn SingleAxisLike>,
    /// The axis representing vertical movement.
    #[serde(deserialize_with = "deserialize_boxed_single_axis")]
    pub y: Box<dyn SingleAxisLike>,
}

impl Clone for DualAxis {
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone_axis(),
            y: self.y.clone_axis(),
        }
    }
}

fn deserialize_boxed_single_axis<'de, D>(
    _deserializer: D,
) -> Result<Box<dyn SingleAxisLike>, D::Error>
where
    D: Deserializer<'de>,
{
    todo!("Implement deserialize for `Box<dyn SingleAxisLike>`");
}

impl PartialEq for DualAxis {
    fn eq(&self, other: &Self) -> bool {
        self.x.eq(&other.x) && self.y.eq(&other.y)
    }
}

impl Eq for DualAxis {}

impl DualAxis {
    /// The default size of the deadzone used by constructor methods.
    ///
    /// This cannot be changed, but the struct can be easily manually constructed.
    pub const DEFAULT_DEADZONE: f32 = 0.1;

    /// Creates a [`DualAxis`] with both `positive_low` and `negative_low` in both axes set to `threshold`.
    #[must_use]
    pub fn new(
        x_axis: impl Into<Box<dyn SingleAxisLike>>,
        y_axis: impl Into<Box<dyn SingleAxisLike>>,
    ) -> DualAxis {
        DualAxis {
            x: x_axis.into(),
            y: y_axis.into(),
        }
    }

    // TODO

    /// Creates a [`DualAxis`] for the left analogue stick of the gamepad.
    #[must_use]
    pub fn left_stick() -> DualAxis {
        DualAxis::new(GamepadAxisType::LeftStickX, GamepadAxisType::LeftStickY)
    }

    /// Creates a [`DualAxis`] for the right analogue stick of the gamepad.
    #[must_use]
    pub fn right_stick() -> DualAxis {
        DualAxis::new(GamepadAxisType::RightStickX, GamepadAxisType::RightStickY)
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseWheel`](bevy::input::mouse::MouseWheel) movement
    pub fn mouse_wheel() -> DualAxis {
        DualAxis {
            x: Box::new(MouseWheelAxis::X),
            y: Box::new(MouseWheelAxis::Y),
        }
    }

    /// Creates a [`DualAxis`] corresponding to horizontal and vertical [`MouseMotion`](bevy::input::mouse::MouseMotion) movement
    pub fn mouse_motion() -> DualAxis {
        DualAxis {
            x: Box::new(MouseMotionAxis::X),
            y: Box::new(MouseMotionAxis::Y),
        }
    }
}

impl ButtonLike for DualAxis {
    fn input_pressed(&self, world: &World) -> bool {
        self.x.input_pressed(world) || self.y.input_pressed(world)
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(self.clone())
    }
}

impl SingleAxisLike for DualAxis {
    fn input_value(&self, world: &World) -> f32 {
        let x = self.x.input_value(world);
        let y = self.y.input_value(world);
        Vec2::new(x, y).length()
    }

    fn clone_axis(&self) -> Box<dyn SingleAxisLike> {
        Box::new(self.clone())
    }
}

impl DualAxisLike for DualAxis {
    fn input_axis_pair(&self, world: &World) -> DualAxisData {
        DualAxisData::new(self.x.input_value(world), self.y.input_value(world))
    }
}

impl InputLikeObject for DualAxis {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        Some(self)
    }

    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike> {
        Some(self)
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        Box::new(self.clone())
    }

    fn as_serialize(&self) -> &dyn erased_serde::Serialize {
        self
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self
    }
}

impl<'a> InputLike<'a> for DualAxis {}

impl Typed for DualAxis {
    fn type_info() -> &'static TypeInfo
    where
        Self: Sized,
    {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
    }
}

impl Reflect for DualAxis {
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn get_type_info(&self) -> &'static TypeInfo {
        <Self as Typed>::type_info()
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
