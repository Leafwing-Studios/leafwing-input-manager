//! Type tag registration for trait objects

use std::collections::BTreeMap;

pub use serde_flexitos::Registry;
use serde_flexitos::{DeserializeFn, GetError};

/// A trait for registering type tags.
pub trait RegisterTypeTag<'de, T: ?Sized> {
    /// Registers the specified type tag into the [`InfallibleMapRegistry`].
    fn register_typetag(registry: &mut InfallibleMapRegistry<T>);
}

/// An infallible [`Registry`] that allows multiple registrations of deserializers.
pub struct InfallibleMapRegistry<O: ?Sized, I = &'static str> {
    deserialize_fns: BTreeMap<I, Option<DeserializeFn<O>>>,
    trait_object_name: &'static str,
}

impl<O: ?Sized, I> InfallibleMapRegistry<O, I> {
    /// Creates a new registry, using `trait_object_name` as the name of `O` for diagnostic purposes.
    #[inline]
    pub fn new(trait_object_name: &'static str) -> Self {
        Self {
            deserialize_fns: BTreeMap::new(),
            trait_object_name,
        }
    }
}

impl<O: ?Sized, I: Ord> Registry for InfallibleMapRegistry<O, I> {
    type Identifier = I;
    type TraitObject = O;

    #[inline]
    fn register(&mut self, id: I, deserialize_fn: DeserializeFn<O>) {
        self.deserialize_fns
            .entry(id)
            .or_insert_with(|| Some(deserialize_fn));
    }

    #[inline]
    fn get_deserialize_fn(&self, id: I) -> Result<&DeserializeFn<O>, GetError<I>> {
        match self.deserialize_fns.get(&id) {
            Some(Some(deserialize_fn)) => Ok(deserialize_fn),
            _ => Err(GetError::NotRegistered { id }),
        }
    }

    #[inline]
    fn get_trait_object_name(&self) -> &'static str {
        self.trait_object_name
    }
}
