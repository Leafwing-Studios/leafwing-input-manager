//! Logic for updating user input based on the state of the world.

use std::any::TypeId;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy::{
    app::{App, PreUpdate},
    ecs::{
        schedule::IntoScheduleConfigs,
        system::{StaticSystemParam, SystemParam},
    },
    math::{Vec2, Vec3},
    platform::collections::{HashMap, HashSet},
    prelude::{Res, ResMut, Resource},
    reflect::Reflect,
};

use super::{Axislike, Buttonlike, DualAxislike, TripleAxislike};
use crate::buttonlike::ButtonValue;
use crate::{plugin::InputManagerSystem, InputControlKind};

/// An overarching store for all user inputs.
///
/// This resource allows values to be updated and fetched in a single location,
/// and ensures that their values are only recomputed once per frame.
///
/// To add a new kind of input, call [`InputRegistration::register_input_kind`] during [`App`] setup.
#[derive(Resource, Default, Debug, Reflect)]
pub struct CentralInputStore {
    /// Stores the updated values of each kind of input.
    updated_values: HashMap<TypeId, UpdatedValues>,
    /// Tracks the input kinds that have been registered, to avoid redundant system additions.
    registered_input_kinds: HashSet<TypeId>,
}

impl CentralInputStore {
    /// Clears all existing values.
    ///
    /// This should be called once at the start of each frame, before polling for new input.
    pub fn clear(&mut self) {
        // Clear the values inside of each map:
        // the base maps can be reused, but the values inside them need to be replaced each frame.
        for map in self.updated_values.values_mut() {
            match map {
                UpdatedValues::Buttonlike(buttonlikes) => buttonlikes.clear(),
                UpdatedValues::Axislike(axislikes) => axislikes.clear(),
                UpdatedValues::Dualaxislike(dualaxislikes) => dualaxislikes.clear(),
                UpdatedValues::Tripleaxislike(tripleaxislikes) => tripleaxislikes.clear(),
            }
        }
    }

    /// Updates the value of a [`Buttonlike`] input.
    pub fn update_buttonlike<B: Buttonlike>(&mut self, buttonlike: B, value: ButtonValue) {
        let updated_values = self
            .updated_values
            .entry(TypeId::of::<B>())
            .or_insert_with(|| UpdatedValues::Buttonlike(HashMap::default()));

        let UpdatedValues::Buttonlike(buttonlikes) = updated_values else {
            panic!("Expected Buttonlike, found {:?}", updated_values);
        };

        buttonlikes.insert(Box::new(buttonlike), value);
    }

    /// Updates the value of an [`Axislike`] input.
    pub fn update_axislike<A: Axislike>(&mut self, axislike: A, value: f32) {
        let updated_values = self
            .updated_values
            .entry(TypeId::of::<A>())
            .or_insert_with(|| UpdatedValues::Axislike(HashMap::default()));

        let UpdatedValues::Axislike(axislikes) = updated_values else {
            panic!("Expected Axislike, found {:?}", updated_values);
        };

        axislikes.insert(Box::new(axislike), value);
    }

    /// Updates the value of a [`DualAxislike`] input.
    pub fn update_dualaxislike<D: DualAxislike>(&mut self, dualaxislike: D, value: Vec2) {
        let updated_values = self
            .updated_values
            .entry(TypeId::of::<D>())
            .or_insert_with(|| UpdatedValues::Dualaxislike(HashMap::default()));

        let UpdatedValues::Dualaxislike(dualaxislikes) = updated_values else {
            panic!("Expected DualAxislike, found {:?}", updated_values);
        };

        dualaxislikes.insert(Box::new(dualaxislike), value);
    }

    /// Updates the value of a [`TripleAxislike`] input.
    pub fn update_tripleaxislike<T: TripleAxislike>(&mut self, tripleaxislike: T, value: Vec3) {
        let updated_values = self
            .updated_values
            .entry(TypeId::of::<T>())
            .or_insert_with(|| UpdatedValues::Tripleaxislike(HashMap::default()));

        let UpdatedValues::Tripleaxislike(tripleaxislikes) = updated_values else {
            panic!("Expected TripleAxislike, found {:?}", updated_values);
        };

        tripleaxislikes.insert(Box::new(tripleaxislike), value);
    }

    /// Check if a [`Buttonlike`] input is currently pressing.
    pub fn pressed<B: Buttonlike + Hash + Eq + Clone>(&self, buttonlike: &B) -> bool {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<B>()) else {
            return false;
        };

        let UpdatedValues::Buttonlike(buttonlikes) = updated_values else {
            panic!("Expected Buttonlike, found {:?}", updated_values);
        };

        // PERF: surely there's a way to avoid cloning here
        let boxed_buttonlike: Box<dyn Buttonlike> = Box::new(buttonlike.clone());

        buttonlikes
            .get(&boxed_buttonlike)
            .copied()
            .map(|button| button.pressed)
            .unwrap_or(false)
    }

    /// Fetches the value of a [`Buttonlike`] input.
    ///
    /// This should be between 0.0 and 1.0, where 0.0 is not pressed and 1.0 is fully pressed.
    pub fn button_value<B: Buttonlike + Hash + Eq + Clone>(&self, buttonlike: &B) -> f32 {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<B>()) else {
            return 0.0;
        };

        let UpdatedValues::Buttonlike(buttonlikes) = updated_values else {
            panic!("Expected Buttonlike, found {:?}", updated_values);
        };

        // PERF: surely there's a way to avoid cloning here
        let boxed_buttonlike: Box<dyn Buttonlike> = Box::new(buttonlike.clone());

        buttonlikes
            .get(&boxed_buttonlike)
            .copied()
            .map(|button| button.value)
            .unwrap_or(0.0)
    }

    /// Fetches the value of an [`Axislike`] input.
    ///
    /// This should be between -1.0 and 1.0, where -1.0 is fully left or down and 1.0 is fully right or up.
    pub fn value<A: Axislike + Hash + Eq + Clone>(&self, axislike: &A) -> f32 {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<A>()) else {
            return 0.0;
        };

        let UpdatedValues::Axislike(axislikes) = updated_values else {
            panic!("Expected Axislike, found {:?}", updated_values);
        };

        // PERF: surely there's a way to avoid cloning here
        let boxed_axislike: Box<dyn Axislike> = Box::new(axislike.clone());

        axislikes.get(&boxed_axislike).copied().unwrap_or(0.0)
    }

    /// Fetches the value of a [`DualAxislike`] input.
    pub fn pair<D: DualAxislike + Hash + Eq + Clone>(&self, dualaxislike: &D) -> Vec2 {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<D>()) else {
            return Vec2::ZERO;
        };

        let UpdatedValues::Dualaxislike(dualaxislikes) = updated_values else {
            panic!("Expected DualAxislike, found {:?}", updated_values);
        };

        // PERF: surely there's a way to avoid cloning here
        let boxed_dualaxislike: Box<dyn DualAxislike> = Box::new(dualaxislike.clone());

        dualaxislikes
            .get(&boxed_dualaxislike)
            .copied()
            .unwrap_or(Vec2::ZERO)
    }

    /// Fetches the value of a [`TripleAxislike`] input.
    pub fn triple<T: TripleAxislike + Hash + Eq + Clone>(&self, tripleaxislike: &T) -> Vec3 {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<T>()) else {
            return Vec3::ZERO;
        };

        let UpdatedValues::Tripleaxislike(tripleaxislikes) = updated_values else {
            panic!("Expected TripleAxislike, found {:?}", updated_values);
        };

        // PERF: surely there's a way to avoid cloning here
        let boxed_tripleaxislike: Box<dyn TripleAxislike> = Box::new(tripleaxislike.clone());

        tripleaxislikes
            .get(&boxed_tripleaxislike)
            .copied()
            .unwrap_or(Vec3::ZERO)
    }
}

#[derive(Resource)]
/// This resource exists for each input type that implements [`UpdatableInput`] (e.g. [`bevy::input::mouse::MouseButton`], [`bevy::input::keyboard::KeyCode`]).
/// Set [`EnabledInput::is_enabled`] to `false` to disable corresponding input handling.
pub struct EnabledInput<T> {
    /// Set this flag to `false` to disable input handling of corresponding events.
    pub is_enabled: bool,
    _p: PhantomData<T>,
}

// SAFETY: The `Resource` derive requires `T` to implement `Send + Sync` as well, but since it's
// used only as `PhantomData`, it's safe to say that `EnabledInput` is `Send + Sync` regardless of `T`.
unsafe impl<T> Send for EnabledInput<T> {}
unsafe impl<T> Sync for EnabledInput<T> {}

impl<T: UpdatableInput> Default for EnabledInput<T> {
    fn default() -> Self {
        Self {
            is_enabled: true,
            _p: PhantomData,
        }
    }
}

/// Trait for registering updatable inputs with the central input store
pub trait InputRegistration {
    /// Registers a new source of raw input data of a matching `kind`.
    ///
    /// This will allow the input to be updated based on the state of the world,
    /// by adding the [`UpdatableInput::compute`] system to [`InputManagerSystem::Unify`] during [`PreUpdate`].
    ///
    /// To improve clarity and data consistency, only one kind of input should be registered for each new data stream:
    /// compute the values of all related inputs from the data stored the [`CentralInputStore`].
    ///
    /// This method has no effect if the input kind has already been registered.
    fn register_input_kind<I: UpdatableInput>(&mut self, kind: InputControlKind);
}

impl InputRegistration for App {
    fn register_input_kind<I: UpdatableInput>(&mut self, kind: InputControlKind) {
        let mut central_input_store = self.world_mut().resource_mut::<CentralInputStore>();

        // Ensure this method is idempotent.
        if central_input_store
            .registered_input_kinds
            .contains(&TypeId::of::<I>())
        {
            return;
        }

        central_input_store.updated_values.insert(
            TypeId::of::<I>(),
            UpdatedValues::from_input_control_kind(kind),
        );
        central_input_store
            .registered_input_kinds
            .insert(TypeId::of::<I>());
        self.insert_resource(EnabledInput::<I>::default());
        self.add_systems(
            PreUpdate,
            I::compute
                .in_set(InputManagerSystem::Unify)
                .run_if(input_is_enabled::<I>),
        );
    }
}

pub(crate) fn input_is_enabled<T: UpdatableInput>(enabled_input: Res<EnabledInput<T>>) -> bool {
    enabled_input.is_enabled
}

/// Registers the standard input types defined by [`bevy`] and [`leafwing_input_manager`](crate).
///
/// The set of input kinds registered by this method is controlled by the features enabled:
/// turn off default features to avoid registering input kinds that are not needed.
#[allow(unused_variables)]
pub(crate) fn register_standard_input_kinds(app: &mut App) {
    // Buttonlike
    #[cfg(feature = "keyboard")]
    app.register_input_kind::<bevy::input::keyboard::KeyCode>(InputControlKind::Button);
    #[cfg(feature = "mouse")]
    app.register_input_kind::<bevy::input::mouse::MouseButton>(InputControlKind::Button);
    #[cfg(feature = "gamepad")]
    app.register_input_kind::<bevy::input::gamepad::GamepadButton>(InputControlKind::Button);

    // Axislike
    #[cfg(feature = "gamepad")]
    app.register_input_kind::<bevy::input::gamepad::GamepadAxis>(InputControlKind::Axis);

    // Dualaxislike
    #[cfg(feature = "mouse")]
    app.register_input_kind::<crate::prelude::MouseMove>(InputControlKind::DualAxis);
    #[cfg(feature = "mouse")]
    app.register_input_kind::<crate::prelude::MouseScroll>(InputControlKind::DualAxis);
}

/// A map of values that have been updated during the current frame.
///
/// The key should be the default form of the input if there is no need to differentiate between possible inputs of the same type,
/// and the value should be the updated value fetched from [`UpdatableInput::SourceData`].
#[derive(Debug, Reflect)]
enum UpdatedValues {
    Buttonlike(HashMap<Box<dyn Buttonlike>, ButtonValue>),
    Axislike(HashMap<Box<dyn Axislike>, f32>),
    Dualaxislike(HashMap<Box<dyn DualAxislike>, Vec2>),
    Tripleaxislike(HashMap<Box<dyn TripleAxislike>, Vec3>),
}

impl UpdatedValues {
    fn from_input_control_kind(kind: InputControlKind) -> Self {
        match kind {
            InputControlKind::Button => Self::Buttonlike(HashMap::default()),
            InputControlKind::Axis => Self::Axislike(HashMap::default()),
            InputControlKind::DualAxis => Self::Dualaxislike(HashMap::default()),
            InputControlKind::TripleAxis => Self::Tripleaxislike(HashMap::default()),
        }
    }
}

/// A trait that enables user input to be updated based on the state of the world.
///
/// This trait is intended to be used for the values stored inside of [`CentralInputStore`].
/// For the actual user inputs that you might bind actions to, use [`UserInput`](crate::user_input::UserInput) instead.
///
/// The values of each [`UserInput`](crate::user_input::UserInput) type will be computed by calling the methods on [`CentralInputStore`],
/// and so the [`UpdatableInput`] trait is only needed when defining new kinds of input that we can
/// derive user-facing inputs from.
///
/// In simple cases, a type will be both [`UserInput`](crate::user_input::UserInput) and [`UpdatableInput`],
/// however when configuration is needed (such as for processed axes or virtual d-pads),
/// two distinct types must be used.
///
/// To add a new kind of input, call [`InputRegistration::register_input_kind`] during [`App`] setup.
pub trait UpdatableInput: 'static {
    /// The [`SystemParam`] that must be fetched from the world in order to update the user input.
    ///
    /// # Panics
    ///
    /// This type cannot be [`CentralInputStore`], as that would cause mutable aliasing and panic at runtime.
    type SourceData: SystemParam;

    /// A system that updates the central store of user input based on the state of the world.
    ///
    /// When defining these systems, use the `update` methods on [`CentralInputStore`] to store the new values.
    ///
    /// # Warning
    ///
    /// This system should not be added manually: instead, call [`InputRegistration::register_input_kind`].
    fn compute(
        central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use leafwing_input_manager_macros::Actionlike;

    use crate as leafwing_input_manager;
    use crate::plugin::{CentralInputStorePlugin, InputManagerPlugin};

    #[derive(Actionlike, Debug, PartialEq, Eq, Hash, Clone, Reflect)]
    enum TestAction {
        Run,
        Jump,
    }

    #[test]
    fn central_input_store_is_added_by_plugins() {
        let mut app = App::new();
        app.add_plugins(CentralInputStorePlugin);
        assert!(app.world().contains_resource::<CentralInputStore>());

        let mut app = App::new();
        app.add_plugins(InputManagerPlugin::<TestAction>::default());
        assert!(app.world().contains_resource::<CentralInputStore>());
    }

    #[test]
    fn number_of_maps_matches_number_of_registered_input_kinds() {
        let mut app = App::new();
        app.add_plugins(CentralInputStorePlugin);
        let central_input_store = app.world().resource::<CentralInputStore>();

        assert_eq!(
            central_input_store.updated_values.len(),
            central_input_store.registered_input_kinds.len()
        );
    }

    #[cfg(feature = "mouse")]
    #[test]
    fn compute_call_updates_central_store() {
        use bevy::ecs::system::RunSystemOnce;
        use bevy::prelude::*;

        let mut world = World::new();
        world.init_resource::<CentralInputStore>();

        // MouseButton has a very straightforward implementation, so we can use it for testing.
        let mut mouse_button_input = ButtonInput::<MouseButton>::default();
        mouse_button_input.press(MouseButton::Left);
        assert!(mouse_button_input.pressed(MouseButton::Left));
        dbg!(&mouse_button_input);
        world.insert_resource(mouse_button_input);

        world.run_system_once(MouseButton::compute).unwrap();
        let central_input_store = world.resource::<CentralInputStore>();
        dbg!(central_input_store);
        assert!(central_input_store.pressed(&MouseButton::Left));
    }
}
