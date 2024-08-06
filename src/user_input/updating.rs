//! Logic for updating user input based on the state of the world.

use std::any::TypeId;

use bevy::{
    app::{App, PreUpdate},
    math::Vec2,
    prelude::{
        Gamepad, GamepadButtonType, IntoSystemConfigs, KeyCode, MouseButton, Res, ResMut, Resource,
    },
    utils::{HashMap, HashSet},
};

use crate::plugin::InputManagerSystem;

use super::{
    Axislike, AxislikeChord, Buttonlike, ButtonlikeChord, DualAxislike, DualAxislikeChord,
    GamepadControlAxis, GamepadControlDirection, GamepadStick, GamepadVirtualAxis,
    GamepadVirtualDPad, KeyboardVirtualAxis, KeyboardVirtualDPad, ModifierKey, MouseMoveAxis,
    MouseMoveDirection, MouseScrollAxis, MouseScrollDirection, UserInput,
};

/// An overarching store for all user input.
///
/// This resource allows values to be updated and fetched in a single location,
/// and ensures that their values are only recomputed once per frame.
///
/// To add a new kind of input, call [`CentralInputStore::register_input_kind`] during [`App`] setup.
#[derive(Resource, Default)]
pub struct CentralInputStore {
    /// Stores the updated values of each kind of input.
    updated_values: HashMap<TypeId, UpdatedValues>,
    /// Tracks the input kinds that have been registered, to avoid redundant system additions.
    registered_input_kinds: HashSet<TypeId>,
}

impl CentralInputStore {
    /// Registers a new kind of input.
    ///
    /// This will allow the input to be updated based on the state of the world,
    /// by adding the [`UpdatableUserInput::update`] system to [`InputManagerSystem::Update`]
    /// during [`PreUpdate`].
    ///
    /// This method has no effect if the input kind has already been registered.
    pub fn register_input_kind<I: UpdatableUserInput>(&mut self, app: &mut App) {
        // Ensure this method is idempotent.
        if self.registered_input_kinds.contains(&TypeId::of::<I>()) {
            return;
        }

        self.registered_input_kinds.insert(TypeId::of::<I>());
        app.add_systems(PreUpdate, I::update.in_set(InputManagerSystem::Update));
    }

    /// Registers the standard input types defined by [`bevy`] and [`leafwing_input_manager`](crate).
    pub fn register_standard_input_kinds(&mut self, app: &mut App) {
        /*
        // Buttonlike
        self.register_input_kind::<KeyCode>(app);
        self.register_input_kind::<ModifierKey>(app);
        self.register_input_kind::<MouseButton>(app);
        self.register_input_kind::<MouseMoveDirection>(app);
        self.register_input_kind::<MouseScrollDirection>(app);
        self.register_input_kind::<GamepadButtonType>(app);
        self.register_input_kind::<GamepadControlDirection>(app);
        self.register_input_kind::<ButtonlikeChord>(app);

        // Axislike
        self.register_input_kind::<KeyboardVirtualAxis>(app);
        self.register_input_kind::<MouseMoveAxis>(app);
        self.register_input_kind::<MouseScrollAxis>(app);
        self.register_input_kind::<GamepadVirtualAxis>(app);
        self.register_input_kind::<GamepadControlAxis>(app);
        self.register_input_kind::<AxislikeChord>(app);

        // Dualaxislike
        self.register_input_kind::<KeyboardVirtualDPad>(app);
        self.register_input_kind::<MouseMove>(app);
        self.register_input_kind::<MouseScroll>(app);
        self.register_input_kind::<GamepadVirtualDPad>(app);
        self.register_input_kind::<GamepadStick>(app);
        self.register_input_kind::<DualAxislikeChord>(app);
        */
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
    pub fn pressed(&self, buttonlike: &dyn Buttonlike) -> bool {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<dyn Buttonlike>()) else {
            return false;
        };

        let UpdatedValues::Buttonlike(buttonlikes) = updated_values else {
            panic!("Expected Buttonlike, found {:?}", updated_values);
        };

        buttonlikes.get(buttonlike).copied().unwrap_or(false)
    }

    /// Fetches the value of an [`Axislike`] input.
    pub fn value(&self, axislike: &dyn Axislike) -> f32 {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<dyn Axislike>()) else {
            return 0.0;
        };

        let UpdatedValues::Axislike(axislikes) = updated_values else {
            panic!("Expected Axislike, found {:?}", updated_values);
        };

        axislikes.get(axislike).copied().unwrap_or(0.0)
    }

    /// Fetches the value of a [`DualAxislike`] input.
    pub fn pair(&self, dualaxislike: &dyn DualAxislike) -> Vec2 {
        let Some(updated_values) = self.updated_values.get(&TypeId::of::<dyn DualAxislike>())
        else {
            return Vec2::ZERO;
        };

        let UpdatedValues::Dualaxislike(dualaxislikes) = updated_values else {
            panic!("Expected DualAxislike, found {:?}", updated_values);
        };

        dualaxislikes
            .get(dualaxislike)
            .copied()
            .unwrap_or(Vec2::ZERO)
    }
}

#[derive(Debug)]
enum UpdatedValues {
    Buttonlike(HashMap<Box<dyn Buttonlike>, bool>),
    Axislike(HashMap<Box<dyn Axislike>, f32>),
    Dualaxislike(HashMap<Box<dyn DualAxislike>, Vec2>),
}

impl CentralInputStore {}

/// A trait that enables user input to be updated based on the state of the world.
///
/// This subtrait of [`UserInput`] is only used during plugin setup;
/// all other uses should use the object-safe [`UserInput`] trait instead.
pub trait UpdatableUserInput: UserInput {
    /// The resource data that must be fetched from the world in order to update the user input.
    ///
    /// # Panics
    ///
    /// This type cannot be [`CentralInputStore`], as that would cause mutable aliasing and panic at runtime.
    type SourceData: Resource;

    /// A system that updates the central store of user input based on the state of the world.
    ///
    /// When defining these systems, use the `update` methods on [`CentralInputStore`] to store the new values.
    ///
    /// # Warning
    ///
    /// This system should not be added manually: instead, call [`CentralInputStore::register_input_kind`].
    fn update(central_input_store: ResMut<CentralInputStore>, source_data: Res<Self::SourceData>);
}
