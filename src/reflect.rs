//! Module with manual implementation of [`Reflect`] trait for external types.

use bevy::{
    prelude::{Deref, DerefMut},
    reflect::{
        list_apply, list_partial_eq, utility::GenericTypeInfoCell, FromReflect, FromType,
        GetTypeRegistration, List, ListInfo, Reflect, ReflectFromPtr, ReflectMut, ReflectOwned,
        ReflectRef, TypeInfo, TypePath, TypeRegistration, Typed,
    },
};
use derive_more::{AsMut, AsRef, From};
use serde::{Deserialize, Serialize};

/// Newtype wrapper for [`PetitSet`](petitset::PetitSet) capable of reflection.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Deref,
    DerefMut,
    TypePath,
    AsRef,
    AsMut,
    From,
)]
#[serde(transparent)]
#[type_path = "petitset::PetitSet"]
pub struct ReflectPetitSet<T, const CAP: usize>(petitset::PetitSet<T, CAP>)
where
    T: Reflect + Clone + Eq;

impl<T, const CAP: usize> Default for ReflectPetitSet<T, CAP>
where
    T: Reflect + Clone + Eq,
{
    fn default() -> Self {
        Self(petitset::PetitSet::<T, CAP>::default())
    }
}

impl<T, const CAP: usize> Typed for ReflectPetitSet<T, CAP>
where
    T: FromReflect + TypePath + Clone + Eq,
{
    fn type_info() -> &'static TypeInfo {
        static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
        CELL.get_or_insert::<Self, _>(|| TypeInfo::List(ListInfo::new::<Self, T>()))
    }
}

impl<T, const CAP: usize> FromReflect for ReflectPetitSet<T, CAP>
where
    T: FromReflect + TypePath + Clone + Eq,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        if let ReflectRef::List(ref_list) = reflect.reflect_ref() {
            let mut new_set = petitset::PetitSet::<T, CAP>::new();
            for field in ref_list.iter() {
                let element = T::from_reflect(field)?;
                new_set.try_insert(element).ok()?;
            }
            Some(Self(new_set))
        } else {
            None
        }
    }
}

impl<T, const CAP: usize> GetTypeRegistration for ReflectPetitSet<T, CAP>
where
    T: FromReflect + TypePath + Clone + Eq,
{
    fn get_type_registration() -> TypeRegistration {
        let mut registration = TypeRegistration::of::<Self>();
        registration.insert::<ReflectFromPtr>(FromType::<Self>::from_type());
        registration
    }
}

impl<T, const CAP: usize> List for ReflectPetitSet<T, CAP>
where
    T: FromReflect + TypePath + Clone + Eq,
{
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.0.get_at(index).map(|item| item as &dyn Reflect)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.0
            .get_at_mut(index)
            .map(|item| item as &mut dyn Reflect)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn iter(&self) -> bevy::reflect::ListIter {
        bevy::reflect::ListIter::new(self as &dyn List)
    }

    fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
        self.0
            .into_iter()
            .map(|item| Box::new(item) as Box<dyn Reflect>)
            .collect()
    }

    fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {
        let element = element.take::<T>().unwrap_or_else(|value| {
            T::from_reflect(&*value).unwrap_or_else(|| {
                panic!(
                    "Attempted to insert invalid value of type {}.",
                    value.type_name()
                )
            })
        });
        self.0.insert_at(element, index);
    }

    fn remove(&mut self, index: usize) -> Box<dyn Reflect> {
        Box::new(self.0.remove_at(index))
    }
}

impl<T, const CAP: usize> Reflect for ReflectPetitSet<T, CAP>
where
    T: FromReflect + TypePath + Clone + Eq,
{
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn get_represented_type_info(&self) -> Option<&'static bevy::reflect::TypeInfo> {
        Some(<Self as Typed>::type_info())
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
        list_apply(self, value);
    }

    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
        *self = value.take()?;
        Ok(())
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::List(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::List(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::List(self)
    }

    fn clone_value(&self) -> Box<dyn Reflect> {
        Box::new(self.clone_dynamic())
    }

    fn reflect_partial_eq(&self, value: &dyn Reflect) -> Option<bool> {
        list_partial_eq(self, value)
    }
}

mod tests {
    #[test]
    fn reflect_petit_set() {
        use bevy::reflect::List;

        let mut set: Box<dyn List> = Box::new(crate::reflect::ReflectPetitSet::<i32, 8>::default());

        assert!(set.get_represented_type_info().is_some());
        assert_eq!(set.len(), 0);

        set.push(Box::new(100));
        set.push(Box::new(200));

        assert_eq!(set.len(), 2);
        assert_eq!(set.get(0).unwrap().reflect_partial_eq(&100), Some(true));
        assert_eq!(set.get(1).unwrap().reflect_partial_eq(&200), Some(true));
        assert!(set.get(2).is_none());
    }
}
