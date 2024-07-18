//! This module contains [`InputMap`] and its supporting methods and impls.

use std::fmt::Debug;

#[cfg(feature = "asset")]
use bevy::asset::Asset;
use bevy::log::error;
use bevy::math::Vec2;
use bevy::prelude::{Component, Gamepad, Reflect, Resource};
use bevy::utils::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::clashing_inputs::ClashStrategy;
use crate::input_streams::InputStreams;
use crate::prelude::UserInputWrapper;
use crate::user_input::{Axislike, Buttonlike, DualAxislike};
use crate::{Actionlike, InputControlKind};

/// A Multi-Map that allows you to map actions to multiple [`UserInputs`](crate::user_input::UserInput)s,
/// whether they are [`Buttonlike`], [`Axislike`] or [`DualAxislike`].
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
/// use leafwing_input_manager::InputControlKind;
///
/// // Define your actions.
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
/// enum Action {
///     Move,
///     Run,
///     Jump,
/// }
///
/// // Because our actions aren't all Buttonlike, we can't derive Actionlike.
/// impl Actionlike for Action {
///     // Record what kind of inputs make sense for each action.
///     fn input_control_kind(&self) -> InputControlKind {
///         match self {
///             Action::Move => InputControlKind::DualAxis,
///             Action::Run | Action::Jump => InputControlKind::Button,
///         }
///     }
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
/// .with_dual_axis(Action::Move, KeyboardVirtualDPad::WASD)
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
pub struct InputMap<A: Actionlike> {
    /// The underlying map that stores action-input mappings for [`Buttonlike`] actions.
    buttonlike_map: HashMap<A, Vec<Box<dyn Buttonlike>>>,

    /// The underlying map that stores action-input mappings for [`Axislike`] actions.
    axislike_map: HashMap<A, Vec<Box<dyn Axislike>>>,

    /// The underlying map that stores action-input mappings for [`DualAxislike`] actions.
    dual_axislike_map: HashMap<A, Vec<Box<dyn DualAxislike>>>,

    /// The specified [`Gamepad`] from which this map exclusively accepts input.
    associated_gamepad: Option<Gamepad>,
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        InputMap {
            buttonlike_map: HashMap::default(),
            axislike_map: HashMap::default(),
            dual_axislike_map: HashMap::default(),
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
    pub fn with_dual_axis(mut self, action: A, axis: impl DualAxislike) -> Self {
        self.insert_dual_axis(action, axis);
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

// Insertion
impl<A: Actionlike> InputMap<A> {
    /// Inserts a binding between an `action` and a specific boxed dyn [`Buttonlike`].
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    fn insert_boxed(&mut self, action: A, button: Box<dyn Buttonlike>) -> &mut Self {
        if let Some(bindings) = self.buttonlike_map.get_mut(&action) {
            if !bindings.contains(&button) {
                bindings.push(button);
            }
        } else {
            self.buttonlike_map.insert(action, vec![button]);
        }

        self
    }

    /// Inserts a binding between an `action` and a specific [`Buttonlike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert(&mut self, action: A, button: impl Buttonlike) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::Button,
            "Cannot map a buttonlike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::Button {
            error!(
                "Cannot map a buttonlike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        self.insert_boxed(action, Box::new(button));
        self
    }

    /// Inserts a binding between an `action` and a specific [`Axislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert_axis(&mut self, action: A, axis: impl Axislike) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::Axis,
            "Cannot map a axislike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::Axis {
            error!(
                "Cannot map a axislike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        let axis = Box::new(axis) as Box<dyn Axislike>;
        if let Some(bindings) = self.axislike_map.get_mut(&action) {
            if !bindings.contains(&axis) {
                bindings.push(axis);
            }
        } else {
            self.axislike_map.insert(action, vec![axis]);
        }
        self
    }

    /// Inserts a binding between an `action` and a specific [`Axislike`] `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert_dual_axis(&mut self, action: A, dual_axis: impl DualAxislike) -> &mut Self {
        debug_assert!(
            action.input_control_kind() == InputControlKind::DualAxis,
            "Cannot map a axislike input for action {:?} of kind {:?}",
            action,
            action.input_control_kind()
        );

        if action.input_control_kind() != InputControlKind::DualAxis {
            error!(
                "Cannot map a axislike input for action {:?} of kind {:?}",
                action,
                action.input_control_kind()
            );

            return self;
        }

        let dual_axis = Box::new(dual_axis) as Box<dyn DualAxislike>;
        if let Some(bindings) = self.dual_axislike_map.get_mut(&action) {
            if !bindings.contains(&dual_axis) {
                bindings.push(dual_axis);
            }
        } else {
            self.dual_axislike_map.insert(action, vec![dual_axis]);
        }
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

    /// Merges the provided [`InputMap`] into this `map`, combining their bindings,
    /// avoiding duplicates.
    ///
    /// If the associated gamepads do not match, the association will be removed.
    pub fn merge(&mut self, other: &InputMap<A>) -> &mut Self {
        if self.associated_gamepad != other.associated_gamepad {
            self.clear_gamepad();
        }

        for (other_action, other_inputs) in other.buttonlike_map.iter() {
            for other_input in other_inputs.iter().cloned() {
                self.insert_boxed(other_action.clone(), other_input);
            }
        }

        self
    }
}

// Configuration
impl<A: Actionlike> InputMap<A> {
    /// Fetches the [`Gamepad`] associated with the entity controlled by this input map.
    ///
    /// If this is [`None`], input from any connected gamepad will be used.
    #[must_use]
    #[inline]
    pub const fn gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
    }

    /// Assigns a particular [`Gamepad`] to the entity controlled by this input map.
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
    pub fn with_gamepad(mut self, gamepad: Gamepad) -> Self {
        self.set_gamepad(gamepad);
        self
    }

    /// Assigns a particular [`Gamepad`] to the entity controlled by this input map.
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
    pub fn set_gamepad(&mut self, gamepad: Gamepad) -> &mut Self {
        self.associated_gamepad = Some(gamepad);
        self
    }

    /// Clears any [`Gamepad`] associated with the entity controlled by this input map.
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
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> bool {
        self.process_actions(input_streams, clash_strategy)
            .button_actions
            .get(action)
            .copied()
            .unwrap_or_default()
    }

    /// Determines the correct state for each action according to provided [`InputStreams`].
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
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> UpdatedActions<A> {
        let mut button_actions = HashMap::new();
        let mut axis_actions = HashMap::new();
        let mut dual_axis_actions = HashMap::new();

        // Generate the base action data for each action
        for (action, _input_bindings) in self.iter_buttonlike() {
            let mut final_state = false;
            for binding in _input_bindings {
                if binding.pressed(input_streams) {
                    final_state = true;
                    break;
                }
            }

            button_actions.insert(action.clone(), final_state);
        }

        for (action, _input_bindings) in self.iter_axislike() {
            let mut final_value = 0.0;
            for binding in _input_bindings {
                final_value += binding.value(input_streams);
            }

            axis_actions.insert(action.clone(), final_value);
        }

        for (action, _input_bindings) in self.iter_dual_axislike() {
            let mut final_value = Vec2::ZERO;
            for binding in _input_bindings {
                final_value += binding.axis_pair(input_streams);
            }

            dual_axis_actions.insert(action.clone(), final_value);
        }

        // Handle clashing inputs, possibly removing some pressed actions from the list
        self.handle_clashes(&mut button_actions, input_streams, clash_strategy);

        UpdatedActions {
            button_actions,
            axis_actions,
            dual_axis_actions,
        }
    }
}

/// The output returned by [`InputMap::process_actions`],
/// used by [`ActionState::update`](crate::action_state::ActionState) to update the state of each action.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdatedActions<A: Actionlike> {
    /// The updated state of each buttonlike action.
    pub button_actions: HashMap<A, bool>,
    /// The updated state of each axislike action.
    pub axis_actions: HashMap<A, f32>,
    /// The updated state of each dual-axislike action.
    pub dual_axis_actions: HashMap<A, Vec2>,
}

impl<A: Actionlike> Default for UpdatedActions<A> {
    fn default() -> Self {
        Self {
            button_actions: Default::default(),
            axis_actions: Default::default(),
            dual_axis_actions: Default::default(),
        }
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
                let buttonlike = self.axislike_map.get(action)?;
                Some(
                    buttonlike
                        .iter()
                        .map(|input| UserInputWrapper::Axis(input.clone()))
                        .collect(),
                )
            }
            InputControlKind::DualAxis => {
                let buttonlike = self.dual_axislike_map.get(action)?;
                Some(
                    buttonlike
                        .iter()
                        .map(|input| UserInputWrapper::DualAxis(input.clone()))
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

    /// Count the total number of registered input bindings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.buttonlike_map
            .values()
            .map(|inputs| inputs.len())
            .sum()
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

mod tests {
    use bevy::prelude::Reflect;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate as leafwing_input_manager;
    use crate::prelude::*;

    #[derive(
        Actionlike,
        Serialize,
        Deserialize,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
        Debug,
        Reflect,
    )]
    enum Action {
        Run,
        Jump,
        Hide,
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
        use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode};

        let mut input_map = InputMap::default();
        let mut default_keyboard_map = InputMap::default();
        default_keyboard_map.insert(Action::Run, KeyCode::ShiftLeft);
        default_keyboard_map.insert(
            Action::Hide,
            ButtonlikeChord::new([KeyCode::ControlLeft, KeyCode::KeyH]),
        );

        let mut default_gamepad_map = InputMap::default();
        default_gamepad_map.insert(Action::Run, GamepadButtonType::South);
        default_gamepad_map.insert(Action::Hide, GamepadButtonType::East);

        // Merging works
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_keyboard_map);

        // Merging is idempotent
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_keyboard_map);
    }

    #[test]
    fn gamepad_swapping() {
        use bevy::input::gamepad::Gamepad;

        let mut input_map = InputMap::<Action>::default();
        assert_eq!(input_map.gamepad(), None);

        input_map.set_gamepad(Gamepad { id: 0 });
        assert_eq!(input_map.gamepad(), Some(Gamepad { id: 0 }));

        input_map.clear_gamepad();
        assert_eq!(input_map.gamepad(), None);
    }
}
