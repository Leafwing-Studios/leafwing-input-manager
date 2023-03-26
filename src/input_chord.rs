use bevy::reflect::{
    array_apply, utility::GenericTypeInfoCell, Array, ArrayInfo, ArrayIter, FromReflect, FromType,
    GetTypeRegistration, Reflect, ReflectFromPtr, ReflectMut, ReflectOwned, ReflectRef,
    TypeRegistration, Typed,
};
use petitset::PetitSet;
use serde::{Deserialize, Serialize};

use crate::user_input::InputKind;

/// The result of a successful [`InputChord`] insertion operatio.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SuccessfulChordInsertion {
    /// A newly inserted [`InputKind`] stored at the contained index.
    Novel(usize),

    /// An [`InputKind`] that was already in the chord at the contained index.
    Extant(usize),
}

impl From<petitset::SuccesfulSetInsertion> for SuccessfulChordInsertion {
    fn from(value: petitset::SuccesfulSetInsertion) -> Self {
        match value {
            petitset::SuccesfulSetInsertion::NovelElenent(index) => Self::Novel(index),
            petitset::SuccesfulSetInsertion::ExtantElement(index) => Self::Extant(index),
        }
    }
}

/// An error returned when attempting to insert into a full [`InputChord`].
///
/// It contains the [`InputKind`] that could not be inserted.
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

/// Represents a combination of [`InputKind`], representing a singular action
/// when activated simultaneously.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputChord<const CAP: usize>(pub PetitSet<InputKind, CAP>);

impl<const CAP: usize> InputChord<CAP> {
    /// The current number of inputs in the chord.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The capacity of the [`InputChord`].
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Whether or not the chord contains the input.
    pub fn contains(&self, kind: &InputKind) -> bool {
        self.0.contains(kind)
    }

    /// Returns an iterator over the inputs in the chord.
    pub fn iter(&self) -> impl Iterator<Item = &InputKind> {
        self.0.iter()
    }

    /// Whether or not the chord is a subset of another chord.
    pub fn is_subset<const OTHER_CAP: usize>(&self, other: &InputChord<OTHER_CAP>) -> bool {
        self.0.is_subset(&other.0)
    }

    /// Whether or not the chord is a superset of another chord.
    pub fn is_superset<const OTHER_CAP: usize>(&self, other: &InputChord<OTHER_CAP>) -> bool {
        self.0.is_superset(&other.0)
    }

    /// Whether or not the two chords contain common inputs.
    pub fn is_disjoint<const OTHER_CAP: usize>(&self, other: &InputChord<OTHER_CAP>) -> bool {
        self.0.is_disjoint(&other.0)
    }

    /// Whether or not the chord contains *no* inputs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether or not the chord is full.
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    /// Gets the [`InputKind`] at a given index in the [`InputChord`].
    pub fn get_at(&self, index: usize) -> Option<&InputKind> {
        self.0.get_at(index)
    }

    /// Mutably gets the [`InputKind`] at a given index in the [`InputChord`].
    pub fn get_at_mut(&mut self, index: usize) -> Option<&mut InputKind> {
        self.0.get_at_mut(index)
    }

    /// Clears the chord of all inputs.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Removes the [`InputKind`] at a given index in the [`InputChord`] and returns it, if one exists.
    pub fn remove(&mut self, kind: &InputKind) -> Option<usize> {
        self.0.remove(kind)
    }

    /// Removes the [`InputKind`] at a given index in the [`InputChord`] and returns whether or not one existed.
    pub fn remove_at(&mut self, index: usize) -> bool {
        self.0.remove_at(index)
    }

    /// Removes the [`InputKind`] in the [`InputChord`] if it exists.
    /// If it existed, this returns both the removed [`InputKind`] and the index it
    /// was removed from.
    pub fn take(&mut self, kind: &InputKind) -> Option<(usize, InputKind)> {
        self.0.take(kind)
    }

    /// Removes the [`InputKind`] in the [`InputChord`] at a given index, if it exists.
    /// If it existed, this returns the removed [`InputKind`].
    pub fn take_at(&mut self, index: usize) -> Option<InputKind> {
        self.0.take_at(index)
    }

    /// Whether or not the chord is identical in both inputs and input order.
    pub fn identical(&self, other: Self) -> bool {
        self.0.identical(other.0)
    }

    /// Insert a new [`InputKind`] to the chord in the first available slot.
    ///
    /// Returns a [`SuccessfulChordInsertion`], containing both the index at
    /// which the [`InputKind`] is stored and whether it was already present.
    ///
    /// # Panics
    /// Panics if the chord is full and the [`InputKind`] is not a duplicate.
    pub fn insert(&mut self, kind: InputKind) -> SuccessfulChordInsertion {
        self.0.insert(kind).into()
    }

    /// Attempt to insert a new [`InputKind`] to the chord in the first available slot.
    ///
    /// Inserts the [`InputKind`] if able, then returns either a [`SuccesfulSetInsertion`]
    /// upon success or a [`CapacityError`] upon error.
    pub fn try_insert(
        &mut self,
        kind: InputKind,
    ) -> Result<SuccessfulChordInsertion, CapacityError> {
        match self.0.try_insert(kind) {
            Ok(success) => Ok(success.into()),
            Err(error) => Err(error.into()),
        }
    }

    /// Insert a new [`InputKind`] to the chord at the given index.
    ///
    /// If a matching [`InputKind`] already existed in the chord, it will be moved to the supplied index.
    /// Any [`InputKind`] that was previously there will be moved to the matching [`InputKind`]'s original index.
    ///
    /// Returns `Some(T)` of any [`InputKind`] removed by this operation.
    ///
    /// # Panics
    /// Panics if the given index is larger than CAP.
    pub fn insert_at(&mut self, kind: InputKind, index: usize) -> Option<InputKind> {
        self.0.insert_at(kind, index)
    }

    /// Constructs a new chord by consuming [`InputKind`] values from an iterator.
    ///
    /// The consumed values will be stored in order, with duplicate [`InputKind`] discarded.
    ///
    /// Returns an error if the iterator produces more than `CAP` distinct [`InputKind`].
    /// The returned error will include the [`InputKind`] that could not be inserted.
    pub fn try_from_iter<I: IntoIterator<Item = InputKind>>(
        input_iter: I,
    ) -> Result<Self, CapacityError> {
        let try_set = PetitSet::try_from_iter(input_iter);

        match try_set {
            Ok(set) => Ok(Self(set)),
            Err(error) => Err(CapacityError(error.0 .1)),
        }
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

impl<const CAP: usize> FromIterator<InputKind> for InputChord<CAP> {
    /// Panics if the iterator contains more than `CAP` distinct [`InputKind`]s.
    fn from_iter<I: IntoIterator<Item = InputKind>>(iter: I) -> Self {
        Self(PetitSet::try_from_iter(iter).unwrap())
    }
}

impl<const CAP: usize> IntoIterator for InputChord<CAP> {
    type Item = InputKind;
    type IntoIter = InputChordIter<CAP>;

    fn into_iter(self) -> Self::IntoIter {
        InputChordIter {
            chord: self,
            cursor: 0,
        }
    }
}

/// An [`Iterator`] struct for [`InputChord`].
#[derive(Clone, Debug)]
pub struct InputChordIter<const CAP: usize> {
    pub(crate) chord: InputChord<CAP>,
    cursor: usize,
}

impl<const CAP: usize> Iterator for InputChordIter<CAP> {
    type Item = InputKind;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.chord.0.next_filled_index(self.cursor) {
            self.cursor = index + 1;
            let result = self.chord.take_at(index);
            debug_assert!(result.is_some());
            result
        } else {
            self.cursor = CAP;
            None
        }
    }
}
