//! This module contains [`InputMap`] and its supporting methods and impls.

use std::fmt::Debug;

#[cfg(feature = "asset")]
use bevy::asset::Asset;
use bevy::prelude::{Component, Gamepad, Reflect, Resource};
use bevy::utils::HashMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::action_state::ActionData;
use crate::buttonlike::ButtonState;
use crate::clashing_inputs::ClashStrategy;
use crate::input_streams::InputStreams;
use crate::user_input::{InputKind, Modifier, UserInput};
use crate::Actionlike;

/// A Multi-Map that allows you to map actions to multiple [`UserInput`]s.
///
/// # Many-to-One Mapping
///
/// You can associate multiple [`UserInput`]s (e.g., keyboard keys, mouse buttons, gamepad buttons)
/// with a single action, simplifying handling complex input combinations for the same action.
/// Duplicate associations are ignored.
///
/// # One-to-Many Mapping
///
/// A single [`UserInput`] can be mapped to multiple actions simultaneously.
/// This allows flexibility in defining alternative ways to trigger an action.
///
/// # Clash Resolution
///
/// By default, the [`InputMap`] prioritizes larger [`UserInput`] combinations to trigger actions.
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
/// use leafwing_input_manager::user_input::InputKind;
///
/// // Define your actions.
/// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
/// enum Action {
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
/// .with(Action::Move, VirtualDPad::wasd())
/// .with(Action::Move, DualAxis::left_stick())
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
    /// The underlying map that stores action-input mappings.
    map: HashMap<A, Vec<UserInput>>,

    /// The specified [`Gamepad`] from which this map exclusively accepts input.
    associated_gamepad: Option<Gamepad>,
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        InputMap {
            map: HashMap::default(),
            associated_gamepad: None,
        }
    }
}

// Constructors
impl<A: Actionlike> InputMap<A> {
    /// Creates an [`InputMap`] from an iterator over action-input bindings.
    /// Note that the type of your iterators must be homogeneous.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn new(bindings: impl IntoIterator<Item = (A, impl Into<UserInput>)>) -> Self {
        bindings
            .into_iter()
            .fold(Self::default(), |map, (action, input)| {
                map.with(action, input)
            })
    }

    /// Associates an `action` with a specific `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with(mut self, action: A, input: impl Into<UserInput>) -> Self {
        self.insert(action, input);
        self
    }

    /// Associates an `action` with multiple `inputs`.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_one_to_many(
        mut self,
        action: A,
        inputs: impl IntoIterator<Item = impl Into<UserInput>>,
    ) -> Self {
        self.insert_one_to_many(action, inputs);
        self
    }

    /// Adds multiple action-input bindings with the same input type.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn with_multiple(
        mut self,
        bindings: impl IntoIterator<Item = (A, impl Into<UserInput>)>,
    ) -> Self {
        self.insert_multiple(bindings);
        self
    }
}

// Insertion
impl<A: Actionlike> InputMap<A> {
    /// Inserts a binding between an `action` and a specific `input`.
    /// Multiple inputs can be bound to the same action.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) -> &mut Self {
        let input = input.into();

        // Check for existing copies of the input: insertion should be idempotent
        let inputs = self.map.get(&action);
        if !inputs.is_some_and(|inputs| inputs.contains(&input)) {
            self.map.entry(action).or_default().push(input);
        }

        self
    }

    /// Inserts bindings between the same `action` and multiple [`UserInput`]s.
    /// Note that the type of your iterators must be homogeneous.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert_one_to_many(
        &mut self,
        action: A,
        inputs: impl IntoIterator<Item = impl Into<UserInput>>,
    ) -> &mut Self {
        let inputs = inputs.into_iter().map(|input| input.into());
        if let Some(bindings) = self.map.get_mut(&action) {
            for input in inputs {
                if !bindings.contains(&input) {
                    bindings.push(input);
                }
            }
        } else {
            self.map.insert(action, inputs.unique().collect());
        }
        self
    }

    /// Inserts multiple action-input bindings.
    /// Note that the type of your iterators must be homogeneous.
    ///
    /// This method ensures idempotence, meaning that adding the same input
    /// for the same action multiple times will only result in a single binding being created.
    #[inline(always)]
    pub fn insert_multiple(
        &mut self,
        bindings: impl IntoIterator<Item = (A, impl Into<UserInput>)>,
    ) -> &mut Self {
        for (action, input) in bindings.into_iter() {
            self.insert(action, input);
        }
        self
    }

    /// Insert a mapping between the simultaneous combination of `buttons` and the `action` provided
    ///
    /// Any iterator convertible into a [`InputKind`] can be supplied,
    /// but will be converted into a [`HashSet`](bevy::utils::HashSet) for storage and use.
    /// Chords can also be added with the [insert](Self::insert) method if the [`UserInput::Chord`] variant is constructed explicitly.
    ///
    /// When working with keyboard modifier keys, consider using the `insert_modified` method instead.
    pub fn insert_chord(
        &mut self,
        action: A,
        buttons: impl IntoIterator<Item = impl Into<InputKind>>,
    ) -> &mut Self {
        self.insert(action, UserInput::chord(buttons));
        self
    }

    /// Inserts a mapping between the simultaneous combination of the [`Modifier`] plus the `input` and the `action` provided.
    ///
    /// When working with keyboard modifiers, should be preferred over `insert_chord`.
    pub fn insert_modified(
        &mut self,
        action: A,
        modifier: Modifier,
        input: impl Into<InputKind>,
    ) -> &mut Self {
        self.insert(action, UserInput::modified(modifier, input));
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

        for (other_action, other_inputs) in other.map.iter() {
            for other_input in other_inputs.iter().cloned() {
                self.insert(other_action.clone(), other_input);
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
    /// Checks if the `action` are currently pressed by any of the associated [`UserInput`]s.
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
            .get(action)
            .map(|datum| datum.state.pressed())
            .unwrap_or_default()
    }

    /// Processes [`UserInput`] bindings for each action and generates corresponding [`ActionData`].
    ///
    /// Accounts for clashing inputs according to the [`ClashStrategy`] and remove conflicting actions.
    #[must_use]
    pub fn process_actions(
        &self,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> HashMap<A, ActionData> {
        let mut action_data = HashMap::new();

        // Generate the raw action presses
        for (action, input_bindings) in self.iter() {
            let mut action_datum = ActionData::default();

            for input in input_bindings {
                // Merge the axis pair into action datum
                if let Some(axis_pair) = input_streams.input_axis_pair(input) {
                    action_datum.axis_pair = action_datum
                        .axis_pair
                        .map_or(Some(axis_pair), |current_axis_pair| {
                            Some(current_axis_pair.merged_with(axis_pair))
                        });
                }

                if input_streams.input_pressed(input) {
                    action_datum.state = ButtonState::JustPressed;
                    action_datum.value += input_streams.input_value(input);
                }
            }

            action_data.insert(action.clone(), action_datum);
        }

        // Handle clashing inputs, possibly removing some pressed actions from the list
        self.handle_clashes(&mut action_data, input_streams, clash_strategy);

        action_data
    }
}

// Utilities
impl<A: Actionlike> InputMap<A> {
    /// Returns an iterator over all registered actions with their input bindings.
    pub fn iter(&self) -> impl Iterator<Item = (&A, &Vec<UserInput>)> {
        self.map.iter()
    }

    /// Returns an iterator over all registered action-input bindings.
    pub fn bindings(&self) -> impl Iterator<Item = (&A, &UserInput)> {
        self.map
            .iter()
            .flat_map(|(action, inputs)| inputs.iter().map(move |input| (action, input)))
    }

    /// Returns an iterator over all registered actions.
    pub fn actions(&self) -> impl Iterator<Item = &A> {
        self.map.keys()
    }

    /// Returns a reference to the inputs associated with the given `action`.
    #[must_use]
    pub fn get(&self, action: &A) -> Option<&Vec<UserInput>> {
        self.map.get(action)
    }

    /// Returns a mutable reference to the inputs mapped to `action`
    #[must_use]
    pub fn get_mut(&mut self, action: &A) -> Option<&mut Vec<UserInput>> {
        self.map.get_mut(action)
    }

    /// Count the total number of registered input bindings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.values().map(|inputs| inputs.len()).sum()
    }

    /// Returns `true` if the map contains no action-input bindings.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the map, removing all action-input bindings.
    ///
    /// Keeps the allocated memory for reuse.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

// Removing
impl<A: Actionlike> InputMap<A> {
    /// Clears all input bindings associated with the `action`.
    pub fn clear_action(&mut self, action: &A) {
        self.map.remove(action);
    }

    /// Removes the input for the `action` at the provided index.
    ///
    /// Returns `Some(input)` if found.
    pub fn remove_at(&mut self, action: &A, index: usize) -> Option<UserInput> {
        let input_bindings = self.map.get_mut(action)?;
        (input_bindings.len() > index).then(|| input_bindings.remove(index))
    }

    /// Removes the input for the `action` if it exists
    ///
    /// Returns [`Some`] with index if the input was found, or [`None`] if no matching input was found.
    pub fn remove(&mut self, action: &A, input: impl Into<UserInput>) -> Option<usize> {
        let bindings = self.map.get_mut(action)?;
        let user_input = input.into();
        let index = bindings.iter().position(|input| input == &user_input)?;
        bindings.remove(index);
        Some(index)
    }
}

impl<A: Actionlike> From<HashMap<A, Vec<UserInput>>> for InputMap<A> {
    /// Converts a [`HashMap`] mapping actions to multiple [`UserInput`]s into an [`InputMap`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bevy::prelude::*;
    /// use bevy::utils::HashMap;
    /// use leafwing_input_manager::prelude::*;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// // Create an InputMap from a HashMap mapping actions to their input bindings.
    /// let mut map: HashMap<Action, Vec<UserInput>> = HashMap::default();
    ///
    /// // Bind the "run" action to either the left or right shift keys to trigger the action.
    /// map.insert(
    ///     Action::Run,
    ///     vec![KeyCode::ShiftLeft.into(), KeyCode::ShiftRight.into()],
    /// );
    ///
    /// let input_map = InputMap::from(map);
    /// ```
    fn from(raw_map: HashMap<A, Vec<UserInput>>) -> Self {
        let mut input_map = Self::default();
        for (action, inputs) in raw_map.into_iter() {
            input_map.insert_one_to_many(action, inputs);
        }
        input_map
    }
}

impl<A: Actionlike> FromIterator<(A, UserInput)> for InputMap<A> {
    fn from_iter<T: IntoIterator<Item = (A, UserInput)>>(iter: T) -> Self {
        iter.into_iter()
            .fold(Self::default(), |map, (action, input)| {
                map.with(action, input)
            })
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

        let expected_bindings: HashMap<UserInput, Action> = HashMap::from([
            (KeyCode::KeyW.into(), Action::Run),
            (KeyCode::ShiftLeft.into(), Action::Run),
            (KeyCode::KeyR.into(), Action::Run),
            (KeyCode::ShiftRight.into(), Action::Run),
            (KeyCode::Space.into(), Action::Jump),
            (KeyCode::ControlLeft.into(), Action::Hide),
            (KeyCode::ControlRight.into(), Action::Hide),
        ]);

        for (action, input) in input_map.bindings() {
            let expected_action = expected_bindings.get(input).unwrap();
            assert_eq!(expected_action, action);
        }
    }

    #[test]
    fn insertion_idempotency() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::Space);

        let expected: Vec<UserInput> = vec![KeyCode::Space.into()];
        assert_eq!(input_map.get(&Action::Run), Some(&expected));

        // Duplicate insertions should not change anything
        input_map.insert(Action::Run, KeyCode::Space);
        assert_eq!(input_map.get(&Action::Run), Some(&expected));
    }

    #[test]
    fn multiple_insertion() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::default();
        input_map.insert(Action::Run, KeyCode::Space);
        input_map.insert(Action::Run, KeyCode::Enter);

        let expected: Vec<UserInput> = vec![KeyCode::Space.into(), KeyCode::Enter.into()];
        assert_eq!(input_map.get(&Action::Run), Some(&expected));
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
        default_keyboard_map.insert_chord(Action::Hide, [KeyCode::ControlLeft, KeyCode::KeyH]);
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
