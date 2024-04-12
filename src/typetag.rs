//! Type tag registration for trait objects

pub use serde_flexitos::{MapRegistry, Registry};

/// A trait for registering type tags.
pub trait RegisterTypeTag<'de, T: ?Sized> {
    /// Registers the specified type tag into the [`MapRegistry`].
    fn register_typetag(registry: &mut MapRegistry<T>);
}
