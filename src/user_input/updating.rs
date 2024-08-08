//! Logic for updating user input based on the state of the world.

use std::any::TypeId;
use std::hash::Hash;

use bevy::{
    app::{App, PreUpdate},
    math::Vec2,
    prelude::{IntoSystemConfigs, Res, ResMut, Resource},
    reflect::Reflect,
    utils::{HashMap, HashSet},
};

use crate::{plugin::InputManagerSystem, InputControlKind};

use super::{Axislike, Buttonlike, DualAxislike};

/// An overarching store for all user input.
///
/// This resource allows values to be updated and fetched in a single location,
/// and ensures that their values are only recomputed once per frame.
///
/// To add a new kind of input, call [`CentralInputStore::register_input_kind`] during [`App`] setup.
#[derive(Resource, Default, Debug, Reflect)]
pub struct CentralInputStore {
    /// Stores the updated values of each kind of input.
    updated_values: HashMap<TypeId, UpdatedValues>,
    /// Tracks the input kinds that have been registered, to avoid redundant system additions.
    registered_input_kinds: HashSet<TypeId>,
}

impl CentralInputStore {
    /// Registers a new source of raw input data of a matching `kind`.
    ///
    /// This will allow the input to be updated based on the state of the world,
    /// by adding the [`UpdatableInput::compute`] system to [`InputManagerSystem::Unify`] during [`PreUpdate`].
    ///
    /// To improve clarity and data consistency, only one kind of input should be registered for each new data stream:
    /// compute the values of all related inputs from the data stored the [`CentralInputStore`].
    ///
    /// This method has no effect if the input kind has already been registered.
    pub fn register_input_kind<I: UpdatableInput>(
        &mut self,
        kind: InputControlKind,
        app: &mut App,
    ) {
        // Ensure this method is idempotent.
        if self.registered_input_kinds.contains(&TypeId::of::<I>()) {
            return;
        }

        self.updated_values.insert(
            TypeId::of::<I>(),
            UpdatedValues::from_input_control_kind(kind),
        );
        self.registered_input_kinds.insert(TypeId::of::<I>());
        app.add_systems(PreUpdate, I::compute.in_set(InputManagerSystem::Unify));
    }

    /// Registers the standard input types defined by [`bevy`] and [`leafwing_input_manager`](crate).
    ///
    /// The set of input kinds registered by this method is controlled by the features enabled:
    /// turn off default features to avoid registering input kinds that are not needed.
    #[allow(unused_variables)]
    pub fn register_standard_input_kinds(&mut self, app: &mut App) {
        // Buttonlike
        #[cfg(feature = "keyboard")]
        self.register_input_kind::<bevy::input::keyboard::KeyCode>(InputControlKind::Button, app);
        #[cfg(feature = "mouse")]
        self.register_input_kind::<bevy::input::mouse::MouseButton>(InputControlKind::Button, app);
        #[cfg(feature = "gamepad")]
        self.register_input_kind::<bevy::input::gamepad::GamepadButton>(
            InputControlKind::Button,
            app,
        );

        // Axislike
        #[cfg(feature = "gamepad")]
        self.register_input_kind::<bevy::input::gamepad::GamepadAxis>(InputControlKind::Axis, app);

        // Dualaxislike
        #[cfg(feature = "mouse")]
        self.register_input_kind::<crate::prelude::MouseMove>(InputControlKind::DualAxis, app);
        #[cfg(feature = "mouse")]
        self.register_input_kind::<crate::prelude::MouseScroll>(InputControlKind::DualAxis, app);
    }

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
            }
        }
    }

    /// Updates the value of a [`Buttonlike`] input.
    pub fn update_buttonlike<B: Buttonlike>(&mut self, buttonlike: B, pressed: bool) {
        let updated_values = self
            .updated_values
            .entry(TypeId::of::<B>())
            .or_insert_with(|| UpdatedValues::Buttonlike(HashMap::new()));

        let UpdatedValues::Buttonlike(buttonlikes) = updated_values else {
            panic!("Expected Buttonlike, found {:?}", updated_values);
        };

        buttonlikes.insert(Box::new(buttonlike), pressed);
    }

    /// Updates the value of an [`Axislike`] input.
    pub fn update_axislike<A: Axislike>(&mut self, axislike: A, value: f32) {
        let updated_values = self
            .updated_values
            .entry(TypeId::of::<A>())
            .or_insert_with(|| UpdatedValues::Axislike(HashMap::new()));

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
            .or_insert_with(|| UpdatedValues::Dualaxislike(HashMap::new()));

        let UpdatedValues::Dualaxislike(dualaxislikes) = updated_values else {
            panic!("Expected Axislike, found {:?}", updated_values);
        };

        dualaxislikes.insert(Box::new(dualaxislike), value);
    }

    /// Fetches the value of a [`Buttonlike`] input.
    pub fn pressed<B: Buttonlike + Hash + Eq + Clone>(&self, buttonlike: &B) -> bool {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<B>()) else {
            return false;
        };

        let UpdatedValues::Buttonlike(buttonlikes) = updated_values else {
            panic!("Expected Buttonlike, found {:?}", updated_values);
        };

        // PERF: surely there's a way to avoid cloning here
        let boxed_buttonlike: Box<dyn Buttonlike> = Box::new(buttonlike.clone());

        buttonlikes.get(&boxed_buttonlike).copied().unwrap_or(false)
    }

    /// Fetches the value of an [`Axislike`] input.
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
}

/// A map of values that have been updated during the current frame.
///
/// The key should be the default form of the input if there is no need to differentiate between possible inputs of the same type,
/// and the value should be the updated value fetched from [`UpdatableInput::SourceData`].
#[derive(Debug, Reflect)]
enum UpdatedValues {
    Buttonlike(HashMap<Box<dyn Buttonlike>, bool>),
    Axislike(HashMap<Box<dyn Axislike>, f32>),
    Dualaxislike(HashMap<Box<dyn DualAxislike>, Vec2>),
}

impl UpdatedValues {
    fn from_input_control_kind(kind: InputControlKind) -> Self {
        match kind {
            InputControlKind::Button => Self::Buttonlike(HashMap::new()),
            InputControlKind::Axis => Self::Axislike(HashMap::new()),
            InputControlKind::DualAxis => Self::Dualaxislike(HashMap::new()),
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
/// To add a new kind of input, call [`CentralInputStore::register_input_kind`] during [`App`] setup.
pub trait UpdatableInput: 'static {
    /// The resource data that must be fetched from the world in order to update the user input.
    ///
    /// # Panics
    ///
    /// This type cannot be [`CentralInputStore`], as that would cause mutable aliasing and panic at runtime.
    // TODO: Ideally this should be a `SystemParam` for more flexibility.
    type SourceData: Resource;

    /// A system that updates the central store of user input based on the state of the world.
    ///
    /// When defining these systems, use the `update` methods on [`CentralInputStore`] to store the new values.
    ///
    /// # Warning
    ///
    /// This system should not be added manually: instead, call [`CentralInputStore::register_input_kind`].
    fn compute(central_input_store: ResMut<CentralInputStore>, source_data: Res<Self::SourceData>);
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

        world.run_system_once(MouseButton::compute);
        let central_input_store = world.resource::<CentralInputStore>();
        dbg!(central_input_store);
        assert!(central_input_store.pressed(&MouseButton::Left));
    }
}
