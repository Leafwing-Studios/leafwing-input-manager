//! The dynamic action type.
//!
//! This type is primarily intended for use when declaring every single action in a single enum isn't feasible,
//! for example due to a complex crate hierarchy where various feature crates want to individually declare their own
//! actions without needing to coordinate with other crates.
//!
//! Example:
//! ```
//! # use leafwing_input_manager::dynamic_action::{DynActionMarker, DynActionRegistry};
//!
//! // In crate `feature_one`
//! #[derive(DynActionMarker)]
//! struct FeatureOneAction;
//!
//! fn register_feature_one_actions(registry: &mut DynActionRegistry) {
//!     // There could potentially be many types registered here
//!     registry.register::<FeatureOneAction>();
//! }
//!
//! // In crate `feature_two`
//! #[derive(DynActionMarker)]
//! struct FeatureTwoAction;
//!
//! fn register_feature_two_actions(registry: &mut DynActionRegistry) {
//!     // There could potentially be many types registered here
//!     registry.register::<FeatureTwoAction>();
//! }
//!
//! // In crate `top_level_crate`, which depends on `feature_one` and `feature_two`
//!
//! let mut registry = DynActionRegistry::get().unwrap();
//! register_feature_one_actions(&mut registry);
//! register_feature_two_actions(&mut registry);
//! registry.finish();
//!
//! ```

use std::any::TypeId;

use bevy::{prelude::Resource, utils::HashMap};
use once_cell::sync::OnceCell;

use crate::Actionlike;

pub use leafwing_input_manager_macros::DynActionMarker;

static DYN_ACTION_MAP: OnceCell<HashMap<TypeId, usize>> = OnceCell::new();
static REGISTRY_CREATED: OnceCell<()> = OnceCell::new();

/// The runtime representation of action s declared via marker types
#[derive(Copy, Clone)]
pub struct DynAction(usize);

/// Coordinates the registration of dynamic action types
#[derive(Resource)]
pub struct DynActionRegistry(Vec<TypeId>);

impl DynActionRegistry {
    /// Tries to get a [`DynActionRegistry`]. This will fail if this function has been called before!
    pub fn get() -> Option<Self> {
        REGISTRY_CREATED.set(()).is_ok().then(|| Self(vec![]))
    }

    /// Registers the given dynamic action type to enable its usage once registration is finalized using [`DynActionRegistry::finish`]
    pub fn register<A: DynActionMarker>(&mut self) {
        self.0.push(TypeId::of::<A>())
    }

    /// Puts the registered types in a global static and enables [`DynAction`] using systems to work.
    ///
    /// Note: Do not create instances of any type in this crate that uses [`DynAction`] as its [`Actionlike`] type before calling this function.
    pub fn finish(self) {
        let map = self
            .0
            .into_iter()
            .enumerate()
            .map(|(i, type_id)| (type_id, i))
            .collect();
        // this cannot fail because this function is the only place where this static is set,
        // and this function s self value can only be created once
        DYN_ACTION_MAP.set(map).unwrap()
    }
}

impl DynAction {
    fn get<A: DynActionMarker>() -> Self {
        DynAction(
            *DYN_ACTION_MAP
                .get()
                .unwrap()
                .get(&TypeId::of::<A>())
                .unwrap(),
        )
    }
}

/// Trait implemented by marker types meant to be used as actions
pub trait DynActionMarker: Sized + 'static {
    /// Gets the [`DynAction`] value associated with this type for use with other parts of this crate
    fn get_action() -> DynAction {
        DynAction::get::<Self>()
    }
}

impl<A: DynActionMarker> From<A> for DynAction {
    fn from(_: A) -> DynAction {
        DynAction::get::<A>()
    }
}

impl Actionlike for DynAction {
    fn n_variants() -> usize {
        DYN_ACTION_MAP.get().unwrap().len()
    }

    fn get_at(index: usize) -> Option<Self> {
        (index < Self::n_variants()).then_some(DynAction(index))
    }

    fn index(&self) -> usize {
        self.0
    }
}
