//! This module contains [`InputMap`] and its supporting methods and impls.

use crate::action_state::ActionData;
use crate::buttonlike::ButtonState;
use crate::clashing_inputs::ClashStrategy;
use crate::input_streams::InputStreams;
use crate::user_input::{InputKind, Modifier, UserInput};
use crate::Actionlike;

#[cfg(feature = "asset")]
use bevy::asset::Asset;
use bevy::ecs::component::Component;
use bevy::ecs::system::Resource;
use bevy::input::gamepad::Gamepad;
use bevy::reflect::Reflect;
use bevy::utils::{Entry, HashMap};
use serde::{Deserialize, Serialize};

use core::fmt::Debug;

/**
Maps from raw inputs to an input-method agnostic representation

Multiple inputs can be mapped to the same action,
and each input can be mapped to multiple actions.

The provided input types must be able to be converted into a [`UserInput`].

By default, if two actions would be triggered by a combination of buttons,
and one combination is a strict subset of the other, only the larger input is registered.
For example, pressing both `S` and `Ctrl + S` in your text editor app would save your file,
but not enter the letters `s`.
Set the [`ClashStrategy`] resource
to configure this behavior.

# Example
```rust
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind;

// You can Run!
// But you can't Hide :(
#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
enum Action {
    Run,
    Hide,
}

// Construction
let mut input_map = InputMap::new([
   // Note that the type of your iterators must be homogenous;
   // you can use `InputKind` or `UserInput` if needed
   // as unifying types
  (Action::Run, GamepadButtonType::South),
  (Action::Hide, GamepadButtonType::LeftTrigger),
  (Action::Hide, GamepadButtonType::RightTrigger),
]);

// Insertion
input_map.insert(Action::Run, MouseButton::Left)
.insert(Action::Run, KeyCode::ShiftLeft)
// Chords
.insert_modified(Action::Run, Modifier::Control, KeyCode::R)
.insert_chord(Action::Run,
              [InputKind::Keyboard(KeyCode::H),
               InputKind::GamepadButton(GamepadButtonType::South),
               InputKind::Mouse(MouseButton::Middle)],
           );

// Removal
input_map.clear_action(&Action::Hide);
```
**/
#[derive(Resource, Component, Debug, Clone, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[cfg_attr(feature = "asset", derive(Asset))]
pub struct InputMap<A: Actionlike> {
    /// The usize stored here is the index of the input in the Actionlike iterator
    map: HashMap<A, Vec<UserInput>>,
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
    /// Creates a new [`InputMap`] from an iterator of `(user_input, action)` pairs
    ///
    /// To create an empty input map, use the [`Default::default`] method instead.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::input_map::InputMap;
    /// use leafwing_input_manager::Actionlike;
    /// use bevy::input::keyboard::KeyCode;
    /// use bevy::prelude::Reflect;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let input_map = InputMap::new([
    ///     (Action::Run, KeyCode::ShiftLeft),
    ///     (Action::Jump, KeyCode::Space),
    /// ]);
    ///
    /// assert_eq!(input_map.len(), 2);
    /// ```
    #[must_use]
    pub fn new(bindings: impl IntoIterator<Item = (A, impl Into<UserInput>)>) -> Self {
        let mut input_map = InputMap::default();
        input_map.insert_multiple(bindings);

        input_map
    }

    /// Constructs a new [`InputMap`] from a `&mut InputMap`, allowing you to insert or otherwise use it
    ///
    /// This is helpful when constructing input maps using the "builder pattern":
    ///  1. Create a new [`InputMap`] struct using [`InputMap::default`] or [`InputMap::new`].
    ///  2. Add bindings and configure the struct using a chain of method calls directly on this struct.
    ///  3. Finish building your struct by calling `.build()`, receiving a concrete struct you can insert as a component.
    ///
    /// Note that this is not the *original* input map, as we do not have ownership of the struct.
    /// Under the hood, this is just a more-readable call to `.clone()`.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;

    /// use bevy::input::keyboard::KeyCode;
    /// use bevy::prelude::Reflect;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let input_map: InputMap<Action> = InputMap::default()
    ///   .insert(Action::Jump, KeyCode::Space).build();
    /// ```
    #[inline]
    #[must_use]
    pub fn build(&mut self) -> Self {
        self.clone()
    }
}

// Insertion
impl<A: Actionlike> InputMap<A> {
    /// Insert a mapping between `input` and `action`
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) -> &mut Self {
        let input = input.into();

        // Check for existing copies of the input: insertion should be idempotent
        if let Some(vec) = self.map.get(&action) {
            if vec.contains(&input) {
                return self;
            }
        }

        match self.map.entry(action) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(input);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![input]);
            }
        };

        self
    }

    /// Insert a mapping between many `input`'s and one `action`
    #[inline(always)]
    pub fn insert_one_to_many(
        &mut self,
        action: A,
        input: impl IntoIterator<Item = impl Into<UserInput>>,
    ) -> &mut Self {
        for input in input {
            self.insert(action.clone(), input);
        }
        self
    }

    /// Insert a mapping between the provided `input_action_pairs`
    ///
    /// This method creates multiple distinct bindings.
    /// If you want to require multiple buttons to be pressed at once, use [`insert_chord`](Self::insert_chord).
    /// Any iterator that can be converted into a [`UserInput`] can be supplied.
    pub fn insert_multiple(
        &mut self,
        input_action_pairs: impl IntoIterator<Item = (A, impl Into<UserInput>)>,
    ) -> &mut Self {
        for (action, input) in input_action_pairs {
            self.insert(action, input);
        }

        self
    }

    /// Insert a mapping between the simultaneous combination of `buttons` and the `action` provided
    ///
    /// Any iterator that can be converted into a [`InputKind`] can be supplied, but will be converted into a [`HashSet`](bevy::utils::HashSet) for storage and use.
    /// Chords can also be added with the [insert](Self::insert) method, if the [`UserInput::Chord`] variant is constructed explicitly.
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

    /// Merges the provided [`InputMap`] into the [`InputMap`] this method was called on
    ///
    /// This adds both of their bindings to the resulting [`InputMap`].
    /// Like usual, any duplicate bindings are ignored.
    ///
    /// If the associated gamepads do not match, the resulting associated gamepad will be set to `None`.
    pub fn merge(&mut self, other: &InputMap<A>) -> &mut Self {
        if self.associated_gamepad != other.associated_gamepad {
            self.associated_gamepad = None;
        }

        for other_action in other.map.iter() {
            for input in other_action.1.iter() {
                self.insert(other_action.0.clone(), input.clone());
            }
        }

        self
    }
}

// Configuration
impl<A: Actionlike> InputMap<A> {
    /// Fetches the [Gamepad] associated with the entity controlled by this entity map
    ///
    /// If this is [`None`], input from any connected gamepad will be used.
    #[must_use]
    pub fn gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
    }

    /// Assigns a particular [`Gamepad`] to the entity controlled by this input map
    ///
    /// If this is not called, input from any connected gamepad will be used.
    /// The first matching non-zero input will be accepted,
    /// as determined by gamepad registration order.
    ///
    /// Because of this robust fallback behavior,
    /// this method can typically be ignored when writing single-player games.
    pub fn set_gamepad(&mut self, gamepad: Gamepad) -> &mut Self {
        self.associated_gamepad = Some(gamepad);
        self
    }

    /// Clears any [Gamepad] associated with the entity controlled by this input map
    pub fn clear_gamepad(&mut self) -> &mut Self {
        self.associated_gamepad = None;
        self
    }
}

// Check whether buttons are pressed
impl<A: Actionlike> InputMap<A> {
    /// Is at least one of the corresponding inputs for `action` found in the provided `input` streams?
    ///
    /// Accounts for clashing inputs according to the [`ClashStrategy`].
    /// If you need to inspect many inputs at once, prefer [`InputMap::which_pressed`] instead.
    #[must_use]
    pub fn pressed(
        &self,
        action: &A,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> bool {
        let action_data = self.which_pressed(input_streams, clash_strategy);
        let Some(action_datum) = action_data.get(action) else {
            return false;
        };

        action_datum.state.pressed()
    }

    /// Returns the actions that are currently pressed, and the responsible [`UserInput`] for each action
    ///
    /// Accounts for clashing inputs according to the [`ClashStrategy`].
    #[must_use]
    pub fn which_pressed(
        &self,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> HashMap<A, ActionData> {
        let mut action_data = HashMap::new();

        // Generate the raw action presses
        for (action, input_vec) in self.iter() {
            let mut action_datum = ActionData::default();

            for input in input_vec {
                // Merge axis pair into action datum
                if let Some(axis_pair) = input_streams.input_axis_pair(input) {
                    action_datum.axis_pair = action_datum
                        .axis_pair
                        .map_or(Some(axis_pair), |current_axis_pair| {
                            Some(current_axis_pair.merged_with(axis_pair))
                        });
                }

                if input_streams.input_pressed(input) {
                    action_datum.state = ButtonState::JustPressed;
                    action_datum.value += input_streams.input_value(input, true);
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
    /// Returns an iterator over actions with their inputs
    pub fn iter(&self) -> impl Iterator<Item = (&A, &Vec<UserInput>)> {
        self.map.iter()
    }
    /// Returns an iterator over actions
    pub(crate) fn actions(&self) -> impl Iterator<Item = &A> {
        self.map.keys()
    }
    /// Returns a reference to the inputs mapped to `action`
    #[must_use]
    pub fn get(&self, action: &A) -> Option<&Vec<UserInput>> {
        self.map.get(action)
    }

    /// Returns a mutable reference to the inputs mapped to `action`
    #[must_use]
    pub fn get_mut(&mut self, action: &A) -> Option<&mut Vec<UserInput>> {
        self.map.get_mut(action)
    }

    /// How many input bindings are registered total?
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.values().map(|inputs| inputs.len()).sum()
    }

    /// Are any input bindings registered at all?
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the map, removing all action-inputs pairs.
    ///
    /// Keeps the allocated memory for reuse.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

// Removing
impl<A: Actionlike> InputMap<A> {
    /// Clears all inputs registered for the `action`
    pub fn clear_action(&mut self, action: &A) {
        self.map.remove(action);
    }

    /// Removes the input for the `action` at the provided index
    ///
    /// Returns `Some(input)` if found.
    pub fn remove_at(&mut self, action: &A, index: usize) -> Option<UserInput> {
        let input_vec = self.map.get_mut(action)?;
        (input_vec.len() > index).then(|| input_vec.remove(index))
    }

    /// Removes the input for the `action`, if it exists
    ///
    /// Returns [`Some`] with index if the input was found, or [`None`] if no matching input was found.
    pub fn remove(&mut self, action: &A, input: impl Into<UserInput>) -> Option<usize> {
        let input_vec = self.map.get_mut(action)?;
        let user_input = input.into();
        let index = input_vec.iter().position(|i| i == &user_input)?;
        input_vec.remove(index);
        Some(index)
    }
}

impl<A: Actionlike> From<HashMap<A, Vec<UserInput>>> for InputMap<A> {
    /// Create `InputMap<A>` from `HashMap<A, Vec<UserInput>>`
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::input_map::InputMap;
    /// use leafwing_input_manager::user_input::UserInput;
    /// use leafwing_input_manager::Actionlike;
    /// use bevy::input::keyboard::KeyCode;
    /// use bevy::reflect::Reflect;
    ///
    /// use bevy::utils::HashMap;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    /// let mut map: HashMap<Action, Vec<UserInput>> = HashMap::default();
    /// map.insert(
    ///     Action::Run,
    ///     vec![KeyCode::ShiftLeft.into(), KeyCode::ShiftRight.into()],
    /// );
    /// let input_map = InputMap::<Action>::from(map);
    /// ```
    fn from(map: HashMap<A, Vec<UserInput>>) -> Self {
        map.iter()
            .flat_map(|(action, inputs)| inputs.iter().map(|input| (action.clone(), input.clone())))
            .collect()
    }
}

impl<A: Actionlike> FromIterator<(A, UserInput)> for InputMap<A> {
    /// Create `InputMap<A>` from iterator with item type `(A, UserInput)`
    fn from_iter<T: IntoIterator<Item = (A, UserInput)>>(iter: T) -> Self {
        InputMap::new(iter)
    }
}

mod tests {
    use bevy::prelude::Reflect;
    use serde::{Deserialize, Serialize};

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
    fn insertion_idempotency() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::<Action>::default();
        input_map.insert(Action::Run, KeyCode::Space);

        assert_eq!(
            input_map.get(&Action::Run),
            Some(&vec![KeyCode::Space.into()])
        );

        // Duplicate insertions should not change anything
        input_map.insert(Action::Run, KeyCode::Space);
        assert_eq!(
            input_map.get(&Action::Run),
            Some(&vec![KeyCode::Space.into()])
        );
    }

    #[test]
    fn multiple_insertion() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map_1 = InputMap::<Action>::default();
        input_map_1.insert(Action::Run, KeyCode::Space);
        input_map_1.insert(Action::Run, KeyCode::Return);

        assert_eq!(
            input_map_1.get(&Action::Run),
            Some(&vec![KeyCode::Space.into(), KeyCode::Return.into()])
        );

        let input_map_2 = InputMap::<Action>::new([
            (Action::Run, KeyCode::Space),
            (Action::Run, KeyCode::Return),
        ]);

        assert_eq!(input_map_1, input_map_2);
    }

    #[test]
    fn chord_singleton_coercion() {
        use crate::input_map::UserInput;
        use bevy::input::keyboard::KeyCode;

        // Single items in a chord should be coerced to a singleton
        let mut input_map_1 = InputMap::<Action>::default();
        input_map_1.insert(Action::Run, KeyCode::Space);

        let mut input_map_2 = InputMap::<Action>::default();
        input_map_2.insert(Action::Run, UserInput::chord([KeyCode::Space]));

        assert_eq!(input_map_1, input_map_2);
    }

    #[test]
    fn input_clearing() {
        use bevy::input::keyboard::KeyCode;

        let mut input_map = InputMap::<Action>::default();
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
        default_keyboard_map.insert_chord(Action::Hide, [KeyCode::ControlLeft, KeyCode::H]);
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
