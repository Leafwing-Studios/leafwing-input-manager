use crate::axislike::DualAxisData;
use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use bevy::prelude::{Reflect, World};
use bevy::reflect::utility::NonGenericTypeInfoCell;
use bevy::reflect::{ReflectMut, ReflectOwned, ReflectRef, TypeInfo, Typed, ValueInfo};
use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone, serde::Serialize, Eq, PartialEq, serde::Deserialize)]
pub struct Chord {
    #[serde(deserialize_with = "deserialize_chord_inner")]
    pub inputs: Vec<Box<dyn InputLikeObject>>,
}

impl Chord {
    pub fn new(inputs: impl IntoIterator<Item = impl Into<Box<dyn InputLikeObject>>>) -> Self {
        let inputs = inputs.into_iter().map(|x| x.into()).collect();
        Self { inputs }
    }

    pub fn contains(&self, input: &dyn InputLikeObject) -> bool {
        self.raw_inputs().iter().any(|x| x.as_ref().eq(input))
    }
}

fn deserialize_chord_inner<'de, D>(
    _deserializer: D,
) -> Result<Vec<Box<dyn InputLikeObject>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    todo!("Implement deserialize for `Vec<Box<dyn InputLikeObject>>`");
}

impl ButtonLike for Chord {
    fn input_pressed(&self, world: &World) -> bool {
        self.inputs.iter().all(|input| {
            input
                .as_button()
                .map(|button| button.input_pressed(world))
                .unwrap_or_default()
        })
    }

    fn clone_button(&self) -> Box<dyn ButtonLike> {
        Box::new(self.clone())
    }
}

impl DualAxisLike for Chord {
    fn input_axis_pair(&self, world: &World) -> DualAxisData {
        let result = self
            .inputs
            .iter()
            .filter_map(|i| i.as_dual_axis())
            .map(|dual_axis| dual_axis.input_axis_pair(world))
            .fold(DualAxisData::default(), |result, dual_axis_data| {
                result.merged_with(dual_axis_data)
            });
        result
    }
}

impl InputLikeObject for Chord {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        None
    }

    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike> {
        Some(self)
    }

    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>> {
        self.inputs.iter().flat_map(|x| x.raw_inputs()).collect()
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

    fn len(&self) -> usize {
        self.inputs.iter().flat_map(|x| x.raw_inputs()).count()
    }
}

impl<'a> InputLike<'a> for Chord {}

impl Reflect for Chord {
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
            Some(PartialEq::eq(self, value))
        } else {
            Some(false)
        }
    }
}

impl Typed for Chord {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_set(|| TypeInfo::Value(ValueInfo::new::<Self>()))
    }
}
