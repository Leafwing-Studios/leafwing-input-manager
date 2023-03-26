use bevy::reflect::{
    array_apply, utility::GenericTypeInfoCell, Array, ArrayInfo, ArrayIter, FromReflect, FromType,
    GetTypeRegistration, Reflect, ReflectFromPtr, ReflectMut, ReflectOwned, ReflectRef,
    TypeRegistration, Typed,
};
use petitset::PetitSet;
use serde::{Deserialize, Serialize};

use crate::user_input::InputKind;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputChord<const CAP: usize>(pub PetitSet<InputKind, CAP>);

/// The `Ok` result of a successful [`InputChord`] insertion operation
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SuccesfulSetInsertion {
    /// This is a newly inserted [`InputKind`] stored at the provided index
    NovelElenent(usize),

    /// This [`InputKind`] was already in the set: it is stored at the provided index
    ExtantElement(usize),
}

impl From<petitset::SuccesfulSetInsertion> for SuccesfulSetInsertion {
    fn from(value: petitset::SuccesfulSetInsertion) -> Self {
        match value {
            petitset::SuccesfulSetInsertion::NovelElenent(index) => Self::NovelElenent(index),
            petitset::SuccesfulSetInsertion::ExtantElement(index) => Self::ExtantElement(index),
        }
    }
}

/// An error returned when attempting to insert into a full [`InputChord`].
///
/// It contains the element that could not be inserted.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct CapacityError(pub InputKind);

impl core::fmt::Debug for CapacityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("A `InputChord` has overflowed.").finish()
    }
}

impl core::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl From<petitset::CapacityError<InputKind>> for CapacityError {
    fn from(value: petitset::CapacityError<InputKind>) -> Self {
        Self(value.0)
    }
}

impl<const CAP: usize> InputChord<CAP> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn contains(&self, element: &InputKind) -> bool {
        self.0.contains(element)
    }

    pub fn iter(&self) -> impl Iterator<Item = &InputKind> {
        self.0.iter()
    }

    pub fn is_subset<const OTHER_CAP: usize>(&self, other: &InputChord<OTHER_CAP>) -> bool {
        self.0.is_subset(&other.0)
    }

    pub fn is_superset<const OTHER_CAP: usize>(&self, other: &InputChord<OTHER_CAP>) -> bool {
        self.0.is_superset(&other.0)
    }

    pub fn is_disjoint<const OTHER_CAP: usize>(&self, other: &InputChord<OTHER_CAP>) -> bool {
        self.0.is_disjoint(&other.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    pub fn get_at(&self, index: usize) -> Option<&InputKind> {
        self.0.get_at(index)
    }

    pub fn get_at_mut(&mut self, index: usize) -> Option<&mut InputKind> {
        self.0.get_at_mut(index)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn remove(&mut self, element: &InputKind) -> Option<usize> {
        self.0.remove(element)
    }

    pub fn remove_at(&mut self, index: usize) -> bool {
        self.0.remove_at(index)
    }

    pub fn take(&mut self, element: &InputKind) -> Option<(usize, InputKind)> {
        self.0.take(element)
    }

    pub fn take_at(&mut self, index: usize) -> Option<InputKind> {
        self.0.take_at(index)
    }

    pub fn identical(&self, other: Self) -> bool {
        self.0.identical(other.0)
    }

    pub fn insert(&mut self, element: InputKind) -> SuccesfulSetInsertion {
        self.0.insert(element).into()
    }

    pub fn try_insert(
        &mut self,
        element: InputKind,
    ) -> Result<SuccesfulSetInsertion, CapacityError> {
        match self.0.try_insert(element) {
            Ok(success) => Ok(success.into()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn insert_at(&mut self, element: InputKind, index: usize) -> Option<InputKind> {
        self.0.insert_at(element, index)
    }
}

impl<const CAP: usize> From<PetitSet<InputKind, CAP>> for InputChord<CAP> {
    fn from(value: PetitSet<InputKind, CAP>) -> Self {
        Self(value)
    }
}

impl<const CAP: usize> Typed for InputChord<CAP> {
    fn type_info() -> &'static bevy::reflect::TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();

        CELL.get_or_insert::<Self, _>(|| {
            bevy::reflect::TypeInfo::Array(ArrayInfo::new::<Self, InputKind>(CAP))
        })
    }
}

impl<const CAP: usize> Array for InputChord<CAP> {
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.0
            .get_at(index)
            .map(|input_kind| input_kind as &dyn Reflect)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.0
            .get_at_mut(index)
            .map(|input_kind| input_kind as &mut dyn Reflect)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> bevy::reflect::ArrayIter {
        ArrayIter::new(self)
    }

    fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
        self.0
            .into_iter()
            .map(|value| Box::new(value) as Box<dyn Reflect>)
            .collect()
    }
}

impl<const CAP: usize> Reflect for InputChord<CAP> {
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn get_type_info(&self) -> &'static bevy::reflect::TypeInfo {
        <Self as Typed>::type_info()
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
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
        array_apply(self, value)
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }

    fn reflect_ref(&self) -> bevy::reflect::ReflectRef {
        ReflectRef::Array(self)
    }

    fn reflect_mut(&mut self) -> bevy::reflect::ReflectMut {
        ReflectMut::Array(self)
    }

    fn reflect_owned(self: Box<Self>) -> bevy::reflect::ReflectOwned {
        ReflectOwned::Array(self)
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic())
    }
}

impl<const CAP: usize> FromReflect for InputChord<CAP> {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let ReflectRef::Array(ref_array) = reflect.reflect_ref() {
            let mut new_petitset = PetitSet::new();

            for (index, field) in ref_array.iter().enumerate() {
                new_petitset.insert_at(<InputKind>::from_reflect(field)?, index);
            }

            Some(InputChord(new_petitset))
        } else {
            None
        }
    }
}

impl<const CAP: usize> GetTypeRegistration for InputChord<CAP> {
    fn get_type_registration() -> bevy::reflect::TypeRegistration {
        let mut registration = TypeRegistration::of::<InputChord<CAP>>();
        registration.insert::<ReflectFromPtr>(FromType::<InputChord<CAP>>::from_type());
        registration
    }
}
