//! Type tag registration for trait objects

use std::fmt::{self, Debug, Formatter};
use std::sync::RwLock;
use std::{borrow::Cow, collections::BTreeMap};

use bevy::reflect::erased_serde;
use serde::Deserializer;
use serde::de::{self, DeserializeSeed, MapAccess, Visitor};
pub use serde_flexitos::Registry;
use serde_flexitos::{DeserializeFn, GetError};

/// A trait for registering type tags.
pub trait RegisterTypeTag<'de, T: ?Sized> {
    /// Registers the specified type tag into the [`InfallibleMapRegistry`].
    fn register_typetag(registry: &mut InfallibleMapRegistry<T>);
}

/// An infallible version of [`MapRegistry`](serde_flexitos::MapRegistry)
/// that allows multiple registrations of deserializers.
pub struct InfallibleMapRegistry<O: ?Sized, I = Cow<'static, str>> {
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

impl<O: ?Sized, I: Ord + Debug> Registry for InfallibleMapRegistry<O, I> {
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

/// Deserializes a trait object from a single id-value pair, using the deserialize functions in `registry`.
///
/// Unlike [`Registry::deserialize_trait_object`], the read lock is only held while looking up the
/// deserialize function for the id, not while that function runs.
/// Composite inputs (chords, virtual axes, ...) deserialize their nested inputs through the same registry,
/// and taking `RwLock::read` again on a thread that already holds it can deadlock
/// when another thread is waiting for the write lock to register inputs.
pub(crate) fn deserialize_trait_object<'de, O: ?Sized, D: Deserializer<'de>>(
    registry: &RwLock<InfallibleMapRegistry<O>>,
    deserializer: D,
) -> Result<Box<O>, D::Error> {
    let trait_object_name = registry.read().unwrap().get_trait_object_name();
    deserializer.deserialize_map(TraitObjectVisitor {
        registry,
        trait_object_name,
    })
}

struct TraitObjectVisitor<'r, O: ?Sized> {
    registry: &'r RwLock<InfallibleMapRegistry<O>>,
    trait_object_name: &'static str,
}

impl<'de, O: ?Sized> Visitor<'de> for TraitObjectVisitor<'_, O> {
    type Value = Box<O>;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "an id-value pair for `Box<dyn {}>`",
            self.trait_object_name
        )
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let Some(id) = map.next_key::<Cow<'static, str>>()? else {
            return Err(de::Error::invalid_length(0, &self));
        };
        // Copy the function out so the guard is dropped before deserializing the value.
        let deserialize_fn = *self
            .registry
            .read()
            .unwrap()
            .get_deserialize_fn(id)
            .map_err(de::Error::custom)?;
        map.next_value_seed(DeserializeWithFn(deserialize_fn))
    }
}

struct DeserializeWithFn<O: ?Sized>(DeserializeFn<O>);

impl<'de, O: ?Sized> DeserializeSeed<'de> for DeserializeWithFn<O> {
    type Value = Box<O>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.0)(&mut erased).map_err(de::Error::custom)
    }
}
