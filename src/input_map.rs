//! This module contains [`InputMap`] and its supporting methods and impls.

use std::fmt::Debug;
use std::hash::Hash;

#[cfg(feature = "asset")]
use bevy::asset::Asset;
use bevy::prelude::{Component, Deref, DerefMut, Entity, Gamepad, Query, Reflect, Resource, With};
use bevy::utils::HashMap;
use bevy::{log::error, prelude::ReflectComponent};
use bevy::{
    math::{Vec2, Vec3},
    prelude::ReflectResource,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::clashing_inputs::ClashStrategy;
use crate::prelude::updating::CentralInputStore;
use crate::prelude::UserInputWrapper;
use crate::user_input::{Axislike, Buttonlike, DualAxislike, TripleAxislike};
use crate::{Actionlike, InputControlKind};

#[cfg(feature = "gamepad")]
use crate::user_input::gamepad::find_gamepad;

#[cfg(not(feature = "gamepad"))]
fn find_gamepad(_: Option<Query<Entity, With<Gamepad>>>) -> Entity {
    Entity::PLACEHOLDER
}

/// A Multi-Map that allows you to map actions to multiple [`UserInputs`](crate::user_input::UserInput)s,
/// whether they are [`Buttonlike`], [`Axislike`], [`DualAxislike`], or [`TripleAxislike`].
///
/// When inserting a binding, the [`InputControlKind`] of the action variant must match that of the input type.
/// Use [`InputMap::insert`] to insert buttonlike inputs,
/// [`InputMap::insert_axis`] to insert axislike inputs,
/// and [`InputMap::insert_dual_axis`] to insert dual-axislike inputs.
///
/// # Many-to-One Mapping
///
/// You can associate multiple [`Buttonlike`]s (e.g., keyboard keys, mouse buttons, gamepad buttons)
/// with a single action, simplifying handling complex input combinations for the same action.
/// Duplicate associations are ignored.
///
/// # One-to-Many Mapping
///
/// A single [`Buttonlike`] can be mapped to multiple actions simultaneously.
/// This allows flexibility in defining alternative ways to trigger an action.
///
/// # Clash Resolution
///
/// By default, the [`InputMap`] prioritizes larger [`Buttonlike`] combinations to trigger actions.
/// This means if two actions share some inputs, and one action requires all the inputs
/// of the other plus additional ones; only the larger combination will be registered.
///
/// This avoids unintended actions from being triggered by more specific input combinations.
/// For example, pressing both `S` and `Ctrl + S` in your text editor app
/// would only save your file (the larger combination), and not enter the letter `s`.
///
/// This behavior can be customized using the [`ClashStrategy`] resource.
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// // Define your actions.
/// #[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
/// enum Action {
///     #[actionlike(DualAxis)]
///     Move,
///     Run,
///     Jump,
/// }
///
/// // Create an InputMap from an iterable,
/// // allowing for multiple input types per action.
/// let mut input_map = InputMap::new([
///     // Multiple inputs can be bound to the same action.
///     // Note that the type of your iterators must be homogeneous.
///     (Action::Run, KeyCode::ShiftLeft),
///     (Action::Run, KeyCode::ShiftRight),
///     // Note that duplicate associations are ignored.
///     (Action::Run, KeyCode::ShiftRight),
///     (Action::Jump, KeyCode::Space),
/// ])
/// // Associate actions with other input types.
/// .with_dual_axis(Action::Move, VirtualDPad::wasd())
/// .with_dual_axis(Action::Move, GamepadStick::LEFT)
/// // Associate an action with multiple inputs at once.
/// .with_one_to_many(Action::Jump, [KeyCode::KeyJ, KeyCode::KeyU]);
///
/// // You can also use methods like a normal MultiMap.
/// input_map.insert(Action::Jump, KeyCode::KeyM);
///
/// // Remove all bindings to a specific action.
/// input_map.clear_action(&Action::Jump);
///
/// // Remove all bindings.
/// input_map.clear();
/// ```
#[derive(Resource, Component, Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[cfg_attr(feature = "asset", derive(Asset))]
#[reflect(Resource, Component)]
pub struct InputMap<A: Actionlike> {
    /// The underlying map that stores action-input mappings for [`Buttonlike`] actions.
    buttonlike_map: HashMap<A, Vec<Box<dyn Buttonlike>>>,

    /// The underlying map that stores action-input mappings for [`Axislike`] actions.
    axislike_map: HashMap<A, Vec<Box<dyn Axislike>>>,

    /// The underlying map that stores action-input mappings for [`DualAxislike`] actions.
    dual_axislike_map: HashMap<A, Vec<Box<dyn DualAxislike>>>,

    /// The underlying map that stores action-input mappings for [`TripleAxislike`] actions.
    triple_axislike_map: HashMap<A, Vec<Box<dyn TripleAxislike>>>,

    /// The specified gamepad from which this map exclusively accepts input.
    associated_gamepad: Option<Entity>,
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        InputMap {
            buttonlike_map: HashMap::default(),
            axislike_map: HashMap::default(),
            dual_axislike_map: HashMap::default(),
            triple_axislike_map: HashMap::default(),
            associated_gamepad: None,
        }
    }
}

// Constructors
impl<A: Actionlike> InputMap<A> {
    /// Creates an [`InputMap`] from an iterator over [`Buttonlike`] action-input bindings.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn new(bindings: impl IntoIterator<Item = (A, impl Buttonlike)>) -> Self {
        bindings
            .into_iter()
            .fold(Self::default(), |map, (action, input)| {
                map.with(action, input)
            })
    }

    /// Associates an `action` with a specific [`Buttonlike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with(mut self, action: A, button: impl Buttonlike) -> Self {
        self.insert(action, button);
        self
    }

    /// Associates an `action` with a specific [`Axislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_axis(mut self, action: A, axis: impl Axislike) -> Self {
        self.insert_axis(action, axis);
        self
    }

    /// Associates an `action` with a specific [`DualAxislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_dual_axis(mut self, action: A, dual_axis: impl DualAxislike) -> Self {
        self.insert_dual_axis(action, dual_axis);
        self
    }

    /// Associates an `action` with a specific [`TripleAxislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_triple_axis(mut self, action: A, triple_axis: impl TripleAxislike) -> Self {
        self.insert_triple_axis(action, triple_axis);
        self
    }

    /// Associates an `action` with multiple [`Buttonlike`] `inputs` provided by an iterator.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_one_to_many(
        mut self,
        action: A,
        inputs: impl IntoIterator<Item = impl Buttonlike>,
    ) -> Self {
        self.insert_one_to_many(action, inputs);
        self
    }

    /// Adds multiple action-input bindings provided by an iterator.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_multiple(
        mut self,
        bindings: impl IntoIterator<Item = (A, impl Buttonlike)>,
    ) -> Self {
        self.insert_multiple(bindings);
        self
    }
}

#[inline(always)]
fn insert_unique<K, V>(map: &mut HashMap<K, Vec<V>>, key: &K, value: V)
where
    K: Clone + Eq + Hash,
    V: PartialEq,
{
    if let Some(list) = map.get_mut(key) {
        if !list.contains(&value) {
            list.push(value);
        }
    } else {
        map.insert(key.clone(), vec![value]);
    }
}

// Insertion
impl<A: Actionlike> InputMap<A> {
    /// Inserts a binding between an `action` and a specific [`Buttonlike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    #[track_caller]
    pub fn insert(&mut self, action: A, button: impl Buttonlike) -> &mut Self {
        self.insert_boxed(action, Box::new(button));
        self
    }

    /// See [`InputMap::insert`] for details.
    ///
    /// This method accepts a boxed [`Buttonlike`] `input`, allowing for
    /// generics to be used in place of specifics. (E.g. seralizing from a config file).
    #[inline(always)]
    #[track_caller]
    pub fn insert_boxed(&mut self, action: A, button: Box<dyn Buttonlike>) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::Button,
            "Cannot map a Buttonlike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::Button {
            error!(
                "Cannot map a Buttonlike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        insert_unique(&mut self.buttonlike_map, &action, button);
        self
    }

    /// Inserts a binding between an `action` and a specific [`Axislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    #[track_caller]
    pub fn insert_axis(&mut self, action: A, axis: impl Axislike) -> &mut Self {
        self.insert_axis_boxed(action, Box::new(axis));
        self
    }

    /// See [`InputMap::insert_axis`] for details.
    ///
    /// This method accepts a boxed [`Axislike`] `input`, allowing for
    /// generics to be used in place of specifics. (E.g. seralizing from a config file).
    #[inline(always)]
    #[track_caller]
    pub fn insert_axis_boxed(&mut self, action: A, axis: Box<dyn Axislike>) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::Axis,
            "Cannot map an Axislike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::Axis {
            error!(
                "Cannot map an Axislike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        insert_unique(&mut self.axislike_map, &action, axis);
        self
    }

    /// Inserts a binding between an `action` and a specific [`DualAxislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    #[track_caller]
    pub fn insert_dual_axis(&mut self, action: A, dual_axis: impl DualAxislike) -> &mut Self {
        self.insert_dual_axis_boxed(action, Box::new(dual_axis));
        self
    }

    /// See [`InputMap::insert_dual_axis`] for details.
    ///
    /// This method accepts a boxed [`DualAxislike`] `input`, allowing for
    /// generics to be used in place of specifics. (E.g. seralizing from a config file).
    #[inline(always)]
    #[track_caller]
    pub fn insert_dual_axis_boxed(&mut self, action: A, axis: Box<dyn DualAxislike>) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::DualAxis,
            "Cannot map a DualAxislike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::DualAxis {
            error!(
                "Cannot map a DualAxislike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        insert_unique(&mut self.dual_axislike_map, &action, axis);
        self
    }

    /// Inserts a binding between an `action` and a specific [`TripleAxislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    #[track_caller]
    pub fn insert_triple_axis(&mut self, action: A, triple_axis: impl TripleAxislike) -> &mut Self {
        self.insert_triple_axis_boxed(action, Box::new(triple_axis));
        self
    }

    /// See [`InputMap::insert_triple_axis`] for details.
    ///
    /// This method accepts a boxed [`TripleAxislike`] `input`, allowing for
    /// generics to be used in place of specifics. (E.g. seralizing from a config file).
    #[inline(always)]
    #[track_caller]
    pub fn insert_triple_axis_boxed(
        &mut self,
        action: A,
        triple_axis: Box<dyn TripleAxislike>,
    ) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::TripleAxis,
            "Cannot map a TripleAxislike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::TripleAxis {
            error!(
                "Cannot map a TripleAxislike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        insert_unique(&mut self.triple_axislike_map, &action, triple_axis);
        self
    }

    /// Inserts bindings between the same `action` and multiple [`Buttonlike`] `inputs` provided by an iterator.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// To insert a chord, such as Control + A, use a [`ButtonlikeChord`](crate::user_input::ButtonlikeChord).
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert_one_to_many(
        &mut self,
        action: A,
        inputs: impl IntoIterator<Item = impl Buttonlike>,
    ) -> &mut Self {
        let inputs = inputs
            .into_iter()
            .map(|input| Box::new(input) as Box<dyn Buttonlike>);
        self.insert_one_to_many_boxed(action, inputs);
        self
    }

    /// See [`InputMap::insert_one_to_many`] for details.
    ///
    /// This method accepts an iterator, over a boxed [`Buttonlike`] `input`, allowing for
    /// generics to be used in place of specifics. (E.g. seralizing from a config file).
    #[inline(always)]
    pub fn insert_one_to_many_boxed(
        &mut self,
        action: A,
        inputs: impl IntoIterator<Item = Box<dyn Buttonlike>>,
    ) -> &mut Self {
        let inputs = inputs.into_iter();
        if let Some(bindings) = self.buttonlike_map.get_mut(&action) {
            for input in inputs {
                if !bindings.contains(&input) {
                    bindings.push(input);
                }
            }
        } else {
            self.buttonlike_map
                .insert(action, inputs.unique().collect());
        }
        self
    }

    /// Inserts multiple action-input [`Buttonlike`] bindings provided by an iterator.
    /// Note that all elements within the iterator must be of the same type (homogeneous).
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert_multiple(
        &mut self,
        bindings: impl IntoIterator<Item = (A, impl Buttonlike)>,
    ) -> &mut Self {
        for (action, input) in bindings.into_iter() {
            self.insert(action, input);
        }
        self
    }

    /// See [`InputMap::insert_multiple`] for details.
    ///
    /// This method accepts an iterator, over a boxed [`Buttonlike`] `input`, allowing for
    /// generics to be used in place of specifics. (E.g. seralizing from a config file).
    #[inline(always)]
    pub fn insert_multiple_boxed(
        &mut self,
        bindings: impl IntoIterator<Item = (A, Box<dyn Buttonlike>)>,
    ) -> &mut Self {
        for (action, input) in bindings.into_iter() {
            self.insert_boxed(action, input);
        }
        self
    }

    /// Merges the provided [`InputMap`] into this `map`, combining their bindings,
    /// avoiding duplicates.
    ///
    /// If the associated gamepads do not match, the association will be removed.
    pub fn merge(&mut self, other: &InputMap<A>) -> &mut Self {
        if self.associated_gamepad != other.associated_gamepad {
            self.clear_gamepad();
        }

        for (other_action, other_inputs) in other.iter_buttonlike() {
            for other_input in other_inputs.iter().cloned() {
                insert_unique(&mut self.buttonlike_map, other_action, other_input);
            }
        }

        for (other_action, other_inputs) in other.iter_axislike() {
            for other_input in other_inputs.iter().cloned() {
                insert_unique(&mut self.axislike_map, other_action, other_input);
            }
        }

        for (other_action, other_inputs) in other.iter_dual_axislike() {
            for other_input in other_inputs.iter().cloned() {
                insert_unique(&mut self.dual_axislike_map, other_action, other_input);
            }
        }

        for (other_action, other_inputs) in other.iter_triple_axislike() {
            for other_input in other_inputs.iter().cloned() {
                insert_unique(&mut self.triple_axislike_map, other_action, other_input);
            }
        }

        self
    }
}

// Configuration
impl<A: Actionlike> InputMap<A> {
    /// Fetches the gamepad [`Entity`] associated with the one controlled by this input map.
    ///
    /// If this is [`None`], input from any connected gamepad will be used.
    #[must_use]
    #[inline]
    pub const fn gamepad(&self) -> Option<Entity> {
        self.associated_gamepad
    }

    /// Assigns a particular gamepad [`Entity`] to the one controlled by this input map.
    ///
    /// Use this when an [`InputMap`] should exclusively accept input
    /// from a particular gamepad.
    ///
    /// If this is not called, input from any connected gamepad will be used.
    /// The first matching non-zero input will be accepted,
    /// as determined by gamepad registration order.
    ///
    /// Because of this robust fallback behavior,
    /// this method can typically be ignored when writing single-player games.
    #[inline]
    pub fn with_gamepad(mut self, gamepad: Entity) -> Self {
        self.set_gamepad(gamepad);
        self
    }

    /// Assigns a particular gamepad [`Entity`] to the one controlled by this input map.
    ///
    /// Use this when an [`InputMap`] should exclusively accept input
    /// from a particular gamepad.
    ///
    /// If this is not called, input from any connected gamepad will be used.
    /// The first matching non-zero input will be accepted,
    /// as determined by gamepad registration order.
    ///
    /// Because of this robust fallback behavior,
    /// this method can typically be ignored when writing single-player games.
    #[inline]
    pub fn set_gamepad(&mut self, gamepad: Entity) -> &mut Self {
        self.associated_gamepad = Some(gamepad);
        self
    }

    /// Clears any gamepad [`Entity`] associated with the one controlled by this input map.
    #[inline]
    pub fn clear_gamepad(&mut self) -> &mut Self {
        self.associated_gamepad = None;
        self
    }
}

// Check whether actions are pressed
impl<A: Actionlike> InputMap<A> {
    /// Checks if the `action` are currently pressed by any of the associated [`Buttonlike`]s.
    ///
    /// Accounts for clashing inputs according to the [`ClashStrategy`] and remove conflicting actions.
    #[must_use]
    pub fn pressed(
        &self,
        action: &A,
        input_store: &CentralInputStore,
        clash_strategy: ClashStrategy,
    ) -> bool {
        let processed_actions = self.process_actions(None, input_store, clash_strategy);

        let Some(updated_value) = processed_actions.get(action) else {
            return false;
        };

        match updated_value {
            UpdatedValue::Button(state) => *state,
            _ => false,
        }
    }

    /// Determines the correct state for each action according to provided [`CentralInputStore`].
    ///
    /// This method uses the input bindings for each action to determine how to parse the input data,
    /// and generates corresponding [`ButtonData`](crate::action_state::ButtonData),
    /// [`AxisData`](crate::action_state::AxisData) and [`DualAxisData`](crate::action_state::DualAxisData).
    ///
    /// For [`Buttonlike`] actions, this accounts for clashing inputs according to the [`ClashStrategy`] and removes conflicting actions.
    ///
    /// [`Buttonlike`] inputs will be pressed if any of the associated inputs are pressed.
    /// [`Axislike`] and [`DualAxislike`] inputs will be the sum of all associated inputs.
    #[must_use]
    pub fn process_actions(
        &self,
        gamepads: Option<Query<Entity, With<Gamepad>>>,
        input_store: &CentralInputStore,
        clash_strategy: ClashStrategy,
    ) -> UpdatedActions<A> {
        let mut updated_actions = UpdatedActions::default();
        let gamepad = self.associated_gamepad.unwrap_or(find_gamepad(gamepads));

        // Generate the base action data for each action
        for (action, _input_bindings) in self.iter_buttonlike() {
            let mut final_state = false;
            for binding in _input_bindings {
                if binding.pressed(input_store, gamepad) {
                    final_state = true;
                    break;
                }
            }

            updated_actions.insert(action.clone(), UpdatedValue::Button(final_state));
        }

        for (action, _input_bindings) in self.iter_axislike() {
            let mut final_value = 0.0;
            for binding in _input_bindings {
                final_value += binding.value(input_store, gamepad);
            }

            updated_actions.insert(action.clone(), UpdatedValue::Axis(final_value));
        }

        for (action, _input_bindings) in self.iter_dual_axislike() {
            let mut final_value = Vec2::ZERO;
            for binding in _input_bindings {
                final_value += binding.axis_pair(input_store, gamepad);
            }

            updated_actions.insert(action.clone(), UpdatedValue::DualAxis(final_value));
        }

        for (action, _input_bindings) in self.iter_triple_axislike() {
            let mut final_value = Vec3::ZERO;
            for binding in _input_bindings {
                final_value += binding.axis_triple(input_store, gamepad);
            }

            updated_actions.insert(action.clone(), UpdatedValue::TripleAxis(final_value));
        }

        // Handle clashing inputs, possibly removing some pressed actions from the list
        self.handle_clashes(&mut updated_actions, input_store, clash_strategy, gamepad);

        updated_actions
    }
}

/// The output returned by [`InputMap::process_actions`],
/// used by [`ActionState::update`](crate::action_state::ActionState) to update the state of each action.
#[derive(Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct UpdatedActions<A: Actionlike>(pub HashMap<A, UpdatedValue>);

impl<A: Actionlike> UpdatedActions<A> {
    /// Returns `true` if the action is both buttonlike and pressed.
    pub fn pressed(&self, action: &A) -> bool {
        match self.0.get(action) {
            Some(UpdatedValue::Button(state)) => *state,
            _ => false,
        }
    }
}

/// An enum representing the updated value of an action.
///
/// Used in [`UpdatedActions`] to store the updated state of each action.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UpdatedValue {
    /// A buttonlike action that was pressed or released.
    Button(bool),
    /// An axislike action that was updated.
    Axis(f32),
    /// A dual-axislike action that was updated.
    DualAxis(Vec2),
    /// A triple-axislike action that was updated.
    TripleAxis(Vec3),
}

impl<A: Actionlike> Default for UpdatedActions<A> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

// Utilities
impl<A: Actionlike> InputMap<A> {
    /// Returns an iterator over all registered [`Buttonlike`] actions with their input bindings.
    pub fn iter_buttonlike(&self) -> impl Iterator<Item = (&A, &Vec<Box<dyn Buttonlike>>)> {
        self.buttonlike_map.iter()
    }

    /// Returns an iterator over all registered [`Axislike`] actions with their input bindings.
    pub fn iter_axislike(&self) -> impl Iterator<Item = (&A, &Vec<Box<dyn Axislike>>)> {
        self.axislike_map.iter()
    }

    /// Returns an iterator over all registered [`DualAxislike`] actions with their input bindings.
    pub fn iter_dual_axislike(&self) -> impl Iterator<Item = (&A, &Vec<Box<dyn DualAxislike>>)> {
        self.dual_axislike_map.iter()
    }

    /// Returns an iterator over all registered [`TripleAxislike`] actions with their input bindings.
    pub fn iter_triple_axislike(
        &self,
    ) -> impl Iterator<Item = (&A, &Vec<Box<dyn TripleAxislike>>)> {
        self.triple_axislike_map.iter()
    }

    /// Returns an iterator over all registered [`Buttonlike`] action-input bindings.
    pub fn buttonlike_bindings(&self) -> impl Iterator<Item = (&A, &dyn Buttonlike)> {
        self.buttonlike_map
            .iter()
            .flat_map(|(action, inputs)| inputs.iter().map(move |input| (action, input.as_ref())))
    }

    /// Returns an iterator over all registered [`Axislike`] action-input bindings.
    pub fn axislike_bindings(&self) -> impl Iterator<Item = (&A, &dyn Axislike)> {
        self.axislike_map
            .iter()
            .flat_map(|(action, inputs)| inputs.iter().map(move |input| (action, input.as_ref())))
    }

    /// Returns an iterator over all registered [`DualAxislike`] action-input bindings.
    pub fn dual_axislike_bindings(&self) -> impl Iterator<Item = (&A, &dyn DualAxislike)> {
        self.dual_axislike_map
            .iter()
            .flat_map(|(action, inputs)| inputs.iter().map(move |input| (action, input.as_ref())))
    }

    /// Returns an iterator over all registered [`TripleAxislike`] action-input bindings.
    pub fn triple_axislike_bindings(&self) -> impl Iterator<Item = (&A, &dyn TripleAxislike)> {
        self.triple_axislike_map
            .iter()
            .flat_map(|(action, inputs)| inputs.iter().map(move |input| (action, input.as_ref())))
    }

    /// Returns an iterator over all registered [`Buttonlike`] actions.
    pub fn buttonlike_actions(&self) -> impl Iterator<Item = &A> {
        self.buttonlike_map.keys()
    }

    /// Returns an iterator over all registered [`Axislike`] actions.
    pub fn axislike_actions(&self) -> impl Iterator<Item = &A> {
        self.axislike_map.keys()
    }

    /// Returns an iterator over all registered [`DualAxislike`] actions.
    pub fn dual_axislike_actions(&self) -> impl Iterator<Item = &A> {
        self.dual_axislike_map.keys()
    }

    /// Returns an iterator over all registered [`TripleAxislike`] actions.
    pub fn triple_axislike_actions(&self) -> impl Iterator<Item = &A> {
        self.triple_axislike_map.keys()
    }

    /// Returns a reference to the [`UserInput`](crate::user_input::UserInput) inputs associated with the given `action`.
    ///
    /// # Warning
    ///
    /// Unlike the other `get` methods, this method is forced to clone the inputs
    /// due to the lack of [trait upcasting coercion](https://github.com/rust-lang/rust/issues/65991).
    ///
    /// As a result, no equivalent `get_mut` method is provided.
    #[must_use]
    pub fn get(&self, action: &A) -> Option<Vec<UserInputWrapper>> {
        match action.input_control_kind() {
            InputControlKind::Button => {
                let buttonlike = self.buttonlike_map.get(action)?;
                Some(
                    buttonlike
                        .iter()
                        .map(|input| UserInputWrapper::Button(input.clone()))
                        .collect(),
                )
            }
            InputControlKind::Axis => {
                let axislike = self.axislike_map.get(action)?;
                Some(
                    axislike
                        .iter()
                        .map(|input| UserInputWrapper::Axis(input.clone()))
                        .collect(),
                )
            }
            InputControlKind::DualAxis => {
                let dual_axislike = self.dual_axislike_map.get(action)?;
                Some(
                    dual_axislike
                        .iter()
                        .map(|input| UserInputWrapper::DualAxis(input.clone()))
                        .collect(),
                )
            }
            InputControlKind::TripleAxis => {
                let triple_axislike = self.triple_axislike_map.get(action)?;
                Some(
                    triple_axislike
                        .iter()
                        .map(|input| UserInputWrapper::TripleAxis(input.clone()))
                        .collect(),
                )
            }
        }
    }

    /// Returns a reference to the [`Buttonlike`] inputs associated with the given `action`.
    #[must_use]
    pub fn get_buttonlike(&self, action: &A) -> Option<&Vec<Box<dyn Buttonlike>>> {
        self.buttonlike_map.get(action)
    }

    /// Returns a mutable reference to the [`Buttonlike`] inputs mapped to `action`
    #[must_use]
    pub fn get_buttonlike_mut(&mut self, action: &A) -> Option<&mut Vec<Box<dyn Buttonlike>>> {
        self.buttonlike_map.get_mut(action)
    }

    /// Returns a reference to the [`Axislike`] inputs associated with the given `action`.
    #[must_use]
    pub fn get_axislike(&self, action: &A) -> Option<&Vec<Box<dyn Axislike>>> {
        self.axislike_map.get(action)
    }

    /// Returns a mutable reference to the [`Axislike`] inputs mapped to `action`
    #[must_use]
    pub fn get_axislike_mut(&mut self, action: &A) -> Option<&mut Vec<Box<dyn Axislike>>> {
        self.axislike_map.get_mut(action)
    }

    /// Returns a reference to the [`DualAxislike`] inputs associated with the given `action`.
    #[must_use]
    pub fn get_dual_axislike(&self, action: &A) -> Option<&Vec<Box<dyn DualAxislike>>> {
        self.dual_axislike_map.get(action)
    }

    /// Returns a mutable reference to the [`DualAxislike`] inputs mapped to `action`
    #[must_use]
    pub fn get_dual_axislike_mut(&mut self, action: &A) -> Option<&mut Vec<Box<dyn DualAxislike>>> {
        self.dual_axislike_map.get_mut(action)
    }

    /// Returns a reference to the [`TripleAxislike`] inputs associated with the given `action`.
    #[must_use]
    pub fn get_triple_axislike(&self, action: &A) -> Option<&Vec<Box<dyn TripleAxislike>>> {
        self.triple_axislike_map.get(action)
    }

    /// Returns a mutable reference to the [`TripleAxislike`] inputs mapped to `action`
    #[must_use]
    pub fn get_triple_axislike_mut(
        &mut self,
        action: &A,
    ) -> Option<&mut Vec<Box<dyn TripleAxislike>>> {
        self.triple_axislike_map.get_mut(action)
    }

    /// Count the total number of registered input bindings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buttonlike_map.values().map(Vec::len).sum::<usize>()
            + self.axislike_map.values().map(Vec::len).sum::<usize>()
            + self.dual_axislike_map.values().map(Vec::len).sum::<usize>()
            + self
                .triple_axislike_map
                .values()
                .map(Vec::len)
                .sum::<usize>()
    }

    /// Returns `true` if the map contains no action-input bindings.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the map, removing all action-input bindings.
    pub fn clear(&mut self) {
        self.buttonlike_map.clear();
        self.axislike_map.clear();
        self.dual_axislike_map.clear();
        self.triple_axislike_map.clear();
    }
}

// Removing
impl<A: Actionlike> InputMap<A> {
    /// Clears all input bindings associated with the `action`.
    pub fn clear_action(&mut self, action: &A) {
        match action.input_control_kind() {
            InputControlKind::Button => {
                self.buttonlike_map.remove(action);
            }
            InputControlKind::Axis => {
                self.axislike_map.remove(action);
            }
            InputControlKind::DualAxis => {
                self.dual_axislike_map.remove(action);
            }
            InputControlKind::TripleAxis => {
                self.triple_axislike_map.remove(action);
            }
        }
    }

    /// Removes the input for the `action` at the provided index.
    ///
    /// Returns `Some(())` if the input was found and removed, or `None` if no matching input was found.
    ///
    /// # Note
    ///
    /// The original input cannot be returned, as the trait object may differ based on the [`InputControlKind`].
    pub fn remove_at(&mut self, action: &A, index: usize) -> Option<()> {
        match action.input_control_kind() {
            InputControlKind::Button => {
                let input_bindings = self.buttonlike_map.get_mut(action)?;
                if input_bindings.len() > index {
                    input_bindings.remove(index);
                    Some(())
                } else {
                    None
                }
            }
            InputControlKind::Axis => {
                let input_bindings = self.axislike_map.get_mut(action)?;
                if input_bindings.len() > index {
                    input_bindings.remove(index);
                    Some(())
                } else {
                    None
                }
            }
            InputControlKind::DualAxis => {
                let input_bindings = self.dual_axislike_map.get_mut(action)?;
                if input_bindings.len() > index {
                    input_bindings.remove(index);
                    Some(())
                } else {
                    None
                }
            }
            InputControlKind::TripleAxis => {
                let input_bindings = self.triple_axislike_map.get_mut(action)?;
                if input_bindings.len() > index {
                    input_bindings.remove(index);
                    Some(())
                } else {
                    None
                }
            }
        }
    }

    /// Removes the input for the `action` if it exists
    ///
    /// Returns [`Some`] with index if the input was found, or [`None`] if no matching input was found.
    pub fn remove(&mut self, action: &A, input: impl Buttonlike) -> Option<usize> {
        let bindings = self.buttonlike_map.get_mut(action)?;
        let boxed_input: Box<dyn Buttonlike> = Box::new(input);
        let index = bindings.iter().position(|input| input == &boxed_input)?;
        bindings.remove(index);
        Some(index)
    }
}

impl<A: Actionlike, U: Buttonlike> From<HashMap<A, Vec<U>>> for InputMap<A> {
    /// Converts a [`HashMap`] mapping actions to multiple [`Buttonlike`]s into an [`InputMap`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use bevy::utils::HashMap;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// // Create an InputMap from a HashMap mapping actions to their key bindings.
    /// let mut map: HashMap<Action, Vec<KeyCode>> = HashMap::default();
    ///
    /// // Bind the "run" action to either the left or right shift keys to trigger the action.
    /// map.insert(
    ///     Action::Run,
    ///     vec![KeyCode::ShiftLeft, KeyCode::ShiftRight],
    /// );
    ///
    /// let input_map = InputMap::from(map);
    /// ```
    fn from(raw_map: HashMap<A, Vec<U>>) -> Self {
        let mut input_map = Self::default();
        for (action, inputs) in raw_map.into_iter() {
            input_map.insert_one_to_many(action, inputs);
        }
        input_map
    }
}

impl<A: Actionlike, U: Buttonlike> FromIterator<(A, U)> for InputMap<A> {
    fn from_iter<T: IntoIterator<Item = (A, U)>>(iter: T) -> Self {
        let mut input_map = Self::default();
        for (action, input) in iter.into_iter() {
            input_map.insert(action, input);
        }
        input_map
    }
}

#[cfg(feature = "keyboard")]
mod tests {
    use bevy::prelude::Reflect;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate as leafwing_input_manager;
    use crate::prelude::*;

    #[derive(Actionlike, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, Reflect)]
    enum Action {
        Run,
        Jump,
        Hide,
        #[actionlike(Axis)]
        Axis,
        #[actionlike(DualAxis)]
        DualAxis,
        #[actionlike(TripleAxis)]
        TripleAxis,
    }

    #[test]
    fn creation() {
        use bevy::input::keyboard::KeyCode;

        let input_map = InputMap::default()
            .with(Action::Run, KeyCode::KeyW)
            .with(Action::Run, KeyCode::ShiftLeft)
            // Duplicate associations should be ignored
            .with(Action::Run, KeyCode::ShiftLeft)
            .with_one_to_many(Action::Run, [KeyCode::KeyR, KeyCode::ShiftRight])
            .with_multiple([
                (Action::Jump, KeyCode::Space),
                (Action::Hide, KeyCode::ControlLeft),
                (Action::Hide, KeyCode::ControlRight),
            ]);

        let expected_bindings: HashMap<Box<dyn Buttonlike>, Action> = HashMap::from([
            (Box::new(KeyCode::KeyW) as Box<dyn Buttonlike>, Action::Run),
            (
                Box::new(KeyCode::ShiftLeft) as Box<dyn Buttonlike>,
                Action::Run,
            ),
            (Box::new(KeyCode::KeyR) as Box<dyn Buttonlike>, Action::Run),
            (
                Box::new(KeyCode::ShiftRight) as Box<dyn Buttonlike>,
                Action::Run,
            ),
            (
                Box::new(KeyCode::Space) as Box<dyn Buttonlike>,
                Action::Jump,
            ),
            (
                Box::new(KeyCode::ControlLeft) as Box<dyn Buttonlike>,
                Action::Hide,
            ),
            (
                Box::new(KeyCode::ControlRight) as Box<dyn Buttonlike>,
                Action::Hide,
            ),
        ]);

        for (action, input) in input_map.buttonlike_bindings() {
            let expected_action = expected_bindings.get(input).unwrap();
            assert_eq!(expected_action, action);
        }
    }

    #[test]
    fn insertion_idempotency() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::Space);

        let expected: Vec<Box<dyn Buttonlike>> = vec![Box::new(KeyCode::Space)];
        assert_eq!(input_map.get_buttonlike(&Action::Run), Some(&expected));

        // Duplicate insertions should not change anything
        input_map.insert(Action::Run, KeyCode::Space);
        assert_eq!(input_map.get_buttonlike(&Action::Run), Some(&expected));
    }

    #[test]
    fn multiple_insertion() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::Space);
        input_map.insert(Action::Run, KeyCode::Enter);

        let expected: Vec<Box<dyn Buttonlike>> =
            vec![Box::new(KeyCode::Space), Box::new(KeyCode::Enter)];
        assert_eq!(input_map.get_buttonlike(&Action::Run), Some(&expected));
    }

    #[test]
    fn input_clearing() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::Space);

        // Clearing action
        input_map.clear_action(&Action::Run);
        assert_eq!(input_map, InputMap::default());

        // Remove input at existing index
        input_map.insert(Action::Run, KeyCode::Space);
        input_map.insert(Action::Run, KeyCode::ShiftLeft);
        assert!(input_map.remove_at(&Action::Run, 1).is_some());
        assert!(
            input_map.remove_at(&Action::Run, 1).is_none(),
            "Should return None on second removal at the same index"
        );
        assert!(input_map.remove_at(&Action::Run, 0).is_some());
        assert!(
            input_map.remove_at(&Action::Run, 0).is_none(),
            "Should return None on second removal at the same index"
        );
    }

    #[test]
    fn merging() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::default();
        let mut default_keyboard_map = InputMap::default();
        default_keyboard_map.insert(Action::Run, KeyCode::ShiftLeft);
        default_keyboard_map.insert(
            Action::Hide,
            ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyH]),
        );

        let mut default_gamepad_map = InputMap::default();
        default_gamepad_map.insert(Action::Run, KeyCode::Numpad0);
        default_gamepad_map.insert(Action::Hide, KeyCode::Numpad7);

        // Merging works
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_keyboard_map);

        // Merging is idempotent
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_keyboard_map);
    }

    #[cfg(feature = "gamepad")]
    #[test]
    fn gamepad_swapping() {
        let mut input_map = InputMap::<Action>::default();
        assert_eq!(input_map.gamepad(), None);

        input_map.set_gamepad(Entity::from_raw(123));
        assert_eq!(input_map.gamepad(), Some(Entity::from_raw(123)));

        input_map.clear_gamepad();
        assert_eq!(input_map.gamepad(), None);
    }

    #[cfg(feature = "keyboard")]
    #[test]
    fn input_map_serde() {
        use bevy::prelude::{App, KeyCode};
        use serde_test::{assert_tokens, Token};

        let mut app = App::new();

        // Add the plugin to register input deserializers
        app.add_plugins(InputManagerPlugin::<Action>::default());

        let input_map = InputMap::new([(Action::Hide, KeyCode::ControlLeft)]);
        assert_tokens(
            &input_map,
            &[
                Token::Struct {
                    name: "InputMap",
                    len: 5,
                },
                Token::Str("buttonlike_map"),
                Token::Map { len: Some(1) },
                Token::UnitVariant {
                    name: "Action",
                    variant: "Hide",
                },
                Token::Seq { len: Some(1) },
                Token::Map { len: Some(1) },
                Token::BorrowedStr("KeyCode"),
                Token::UnitVariant {
                    name: "KeyCode",
                    variant: "ControlLeft",
                },
                Token::MapEnd,
                Token::SeqEnd,
                Token::MapEnd,
                Token::Str("axislike_map"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::Str("dual_axislike_map"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::Str("triple_axislike_map"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::Str("associated_gamepad"),
                Token::None,
                Token::StructEnd,
            ],
        );
    }
}
