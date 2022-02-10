//! This module contains [`InputMap`] and its supporting methods and impls.

use crate::buttonlike_user_input::{InputButton, InputMode, InputStreams, UserInput};
use crate::clashing_inputs::{Clash, ClashStrategy};
use crate::Actionlike;
use bevy::prelude::*;
use bevy::utils::HashSet;
use core::fmt::Debug;
use petitset::PetitSet;
use serde::{Deserialize, Serialize};

/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs can be mapped to the same action,
/// and each input can be mapped to multiple actions.
///
/// The provided input types must be one of [`GamepadButtonType`], [`KeyCode`] or [`MouseButton`].
///
/// The maximum number of bindings (total) that can be stored for each action is 16.
/// Insertions will silently fail if you have reached this cap.
///
/// In addition, you can configure the per-mode cap for each [`InputMode`] using [`InputMap::new`] or [`InputMap::set_per_mode_cap`].
/// This can be useful if your UI can only display one or two possible keybindings for each input mode.
///
/// By default, if two actions would be triggered by a combination of buttons,
/// and one combination is a strict subset of the other, only the larger input is registered.
/// For example, pressing both `S` and `Ctrl + S` in your text editor app would save your file,
/// but not enter the letters `s`.
/// Set the `clashing_inputs` field of this struct with the [`ClashingInputs`] enum
/// to configure this behavior.
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::buttonlike_user_input::InputButton;

///
/// // You can Run!
/// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
/// enum Action {
///     Run,
///     Hide,
/// }
///
/// // Construction
/// let mut input_map = InputMap::new([
///    // Note that the type of your iterators must be homogenous;
///    // you can use `InputButton` or `UserInput` if needed
///    // as unifiying types
///   (Action::Run, GamepadButtonType::South),
///   (Action::Hide, GamepadButtonType::LeftTrigger),
///   (Action::Hide, GamepadButtonType::RightTrigger),
/// ])
/// // Insertion
/// .insert(Action::Run, MouseButton::Left)
/// .insert(Action::Run, KeyCode::LShift)
/// // Chords
/// .insert_chord(Action::Run, [KeyCode::LControl, KeyCode::R])
/// .insert_chord(Action::Hide, [InputButton::Keyboard(KeyCode::H),
///                              InputButton::Gamepad(GamepadButtonType::South),
///                              InputButton::Mouse(MouseButton::Middle)])
/// // Configuration
/// .set_clash_strategy(ClashStrategy::PressAll)
/// // Converting from a `&mut T` into the `T` that we need
/// .build();
///
///
/// // But you can't Hide :(
/// input_map.clear_action(Action::Hide, None);
///```
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputMap<A: Actionlike> {
    /// The raw vector of [PetitSet]s used to store the input mapping,
    /// indexed by the `Actionlike::id` of `A`
    map: Vec<PetitSet<UserInput, 16>>,
    per_mode_cap: Option<u8>,
    associated_gamepad: Option<Gamepad>,
    /// How should clashing (overlapping) inputs be handled?
    pub clash_strategy: ClashStrategy,
    /// A cached list of all pairs of actions that could potentially clash
    pub(crate) possible_clashes: Vec<Clash<A>>,
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        InputMap {
            map: A::iter().map(|_| PetitSet::default()).collect(),
            associated_gamepad: None,
            per_mode_cap: None,
            // This is the most commonly useful behavior.
            clash_strategy: ClashStrategy::PrioritizeLongest,
            // Empty input maps cannot have any clashes
            possible_clashes: Vec::default(),
        }
    }
}

// Constructors
impl<A: Actionlike> InputMap<A> {
    /// Creates a new [`InputMap`] from an iterator of `(action, user_input)` pairs
    ///
    /// To create an empty input map, use the [`Default::default`] method instead.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::input_map::InputMap;
    /// use leafwing_input_manager::Actionlike;

    /// use bevy::input::keyboard::KeyCode;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let input_map = InputMap::new([
    ///     (Action::Run, KeyCode::LShift),
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
    /// Note that this is not the *orginal* input map, as we do not have ownership of the struct.
    /// Under the hood, this is just a more-readable call to `.clone()`.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::prelude::*;

    /// use bevy::input::keyboard::KeyCode;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// Insert a mapping between `action` and `input`
    ///
    /// Existing mappings for that action will not be overwritten.
    /// If the set for this action is already full, this insertion will silently fail.
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) -> &mut Self {
        let input = input.into();

        // Don't insert Null inputs into the map
        if input == UserInput::Null {
            return self;
        }

        // Don't overflow the set!
        if self.n_registered(action.clone(), None) >= 16 {
            return self;
        }

        // Respect any per-input-mode caps that have been set
        if let Some(per_mode_cap) = self.per_mode_cap {
            for input_mode in input.input_modes() {
                if self.n_registered(action.clone(), Some(input_mode)) >= per_mode_cap {
                    return self;
                }
            }
        }

        self.map[action.index()].insert(input);

        // Cache clashes now, to ensure a clean state
        self.cache_possible_clashes();

        self
    }

    /// Insert a mapping between `action` and the provided `inputs`
    ///
    /// This method creates multiple distinct bindings.
    /// If you want to require multiple buttons to be pressed at once, use [`insert_chord`](Self::insert_chord).
    /// Any iterator that can be converted into a [`UserInput`] can be supplied.
    ///
    /// Existing mappings for that action will not be overwritten.
    pub fn insert_multiple(
        &mut self,
        bindings: impl IntoIterator<Item = (A, impl Into<UserInput>)>,
    ) -> &mut Self {
        for (action, input) in bindings {
            self.insert(action, input);
        }

        self
    }

    /// Insert a mapping between `action` and the simultaneous combination of `buttons` provided
    ///
    /// Any iterator that can be converted into a [`Button`] can be supplied, but will be converted into a [`PetitSet`] for storage and use.
    /// Chords can also be added with the [insert](Self::insert) method, if the [`UserInput::Chord`] variant is constructed explicitly.
    ///
    /// Existing mappings for that action will not be overwritten.
    pub fn insert_chord(
        &mut self,
        action: A,
        buttons: impl IntoIterator<Item = impl Into<InputButton>>,
    ) -> &mut Self {
        self.insert(action, UserInput::chord(buttons));
        self
    }

    /// Merges the provided [`InputMap`] into the [`InputMap`] this method was called on
    ///
    /// This adds both of their bindings to the resulting [`InputMap`].
    /// Like usual, any duplicate bindings are ignored.
    ///
    /// If the associated gamepads do not match, the resulting associated gamepad will be set to `None`.
    pub fn merge(&mut self, other: &InputMap<A>) -> &mut Self {
        let associated_gamepad = if self.associated_gamepad == other.associated_gamepad {
            self.associated_gamepad
        } else {
            None
        };

        let mut new_map = InputMap {
            associated_gamepad,
            ..Default::default()
        };

        for action in A::iter() {
            for input in self.get(action.clone(), None) {
                new_map.insert(action.clone(), input);
            }

            for input in other.get(action.clone(), None) {
                new_map.insert(action.clone(), input);
            }
        }

        new_map.cache_possible_clashes();

        *self = new_map;
        self
    }

    /// Replaces any existing inputs for the `action` of the same [`InputMode`] with the provided `input`
    ///
    /// Returns all previously registered inputs, if any
    pub fn replace(
        &mut self,
        action: A,
        input: impl Into<UserInput>,
    ) -> Option<PetitSet<UserInput, 16>> {
        let input = input.into();

        let mut old_inputs: PetitSet<UserInput, 16> = PetitSet::default();
        for input_mode in input.input_modes() {
            for removed_input in self.clear_action(action.clone(), Some(input_mode)) {
                old_inputs.insert(removed_input);
            }
        }

        self.insert(action, input);

        Some(old_inputs)
    }

    /// Replaces the input for the `action`of the same [`InputMode`] at the same index with the provided `input`
    ///
    /// If the input is a [`UserInput::Chord`] that combines multiple input modes or [`UserInput::Null`], this method will silently fail.
    /// Returns the replaced input, if any.
    pub fn replace_at(
        &mut self,
        action: A,
        input: impl Into<UserInput>,
        index: u8,
    ) -> Option<UserInput> {
        let input = input.into();
        let input_modes = input.input_modes();

        if input_modes.len() != 1 {
            return None;
        }

        // We know that the input belongs to exactly one mode
        let input_mode = input_modes.into_iter().next().unwrap();
        let removed = self.clear_at(action.clone(), input_mode, index);
        self.insert(action, input);

        removed
    }
}

// Configuration
impl<A: Actionlike> InputMap<A> {
    /// Returns the per-[`InputMode`] cap on input bindings for every action
    ///
    /// Each individual action can have at most this many bindings, making them easier to display and configure.
    pub fn per_mode_cap(&self) -> u8 {
        if let Some(cap) = self.per_mode_cap {
            cap
        } else {
            0
        }
    }

    /// Sets the per-[`InputMode`] cap on input bindings for every action
    ///
    /// Each individual action can have at most this many bindings, making them easier to display and configure.
    /// Any excess actions will be removed, and returned from this method.
    ///
    /// Supplying a value of 0 removes any per-mode cap.
    ///
    /// PANICS: `3 * per_mode_cap` cannot exceed the global `CAP`, as we need space to store all mappings.
    pub fn set_per_mode_cap(&mut self, per_mode_cap: u8) -> InputMap<A> {
        assert!(3 * per_mode_cap <= 16);

        if per_mode_cap == 0 {
            self.per_mode_cap = None;
            return InputMap::default();
        } else {
            self.per_mode_cap = Some(per_mode_cap);
        }

        // Store the actions that get culled and then return them
        let mut removed_actions = InputMap::default();

        // Cull excess mappings
        for action in A::iter() {
            for input_mode in InputMode::iter() {
                let n_registered = self.n_registered(action.clone(), Some(input_mode));
                if n_registered > per_mode_cap {
                    for i in per_mode_cap..n_registered {
                        let removed_input = self.clear_at(action.clone(), input_mode, i);
                        if let Some(input) = removed_input {
                            removed_actions.insert(action.clone(), input);
                        }
                    }
                }
            }
        }

        removed_actions
    }

    /// Fetches the [Gamepad] associated with the entity controlled by this entity map
    #[must_use]
    pub fn gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
    }

    /// Assigns a particular [`Gamepad`] to the entity controlled by this input map
    pub fn set_gamepad(&mut self, gamepad: Gamepad) -> &mut Self {
        self.associated_gamepad = Some(gamepad);
        self
    }

    /// Clears any [Gamepad] associated with the entity controlled by this input map
    pub fn clear_gamepad(&mut self) -> &mut Self {
        self.associated_gamepad = None;
        self
    }

    /// Sets the [`ClashStrategy`] for this input map
    pub fn set_clash_strategy(&mut self, clash_strategy: ClashStrategy) -> &mut Self {
        self.clash_strategy = clash_strategy;
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
    pub fn pressed(&self, action: A, input_streams: &InputStreams) -> bool {
        let pressed_set = self.which_pressed(input_streams);
        pressed_set.contains(&action.index())
    }

    /// Returns a [`HashSet`] of the virtual buttons that are currently pressed
    ///
    /// Accounts for clashing inputs according to the [`ClashStrategy`].
    /// The `usize`s returned correspond to `Actionlike::index()`.
    #[must_use]
    pub fn which_pressed(&self, input_streams: &InputStreams) -> HashSet<usize> {
        let mut pressed_actions = HashSet::default();

        // Generate the raw action presses
        for action in A::iter() {
            for input in self.get(action.clone(), None) {
                if input_streams.input_pressed(&input) {
                    pressed_actions.insert(action.index());
                    // No need to press more than once
                    break;
                }
            }
        }

        // Handle clashing inputs, possibly removing some pressed actions from the list
        if self.clash_strategy != ClashStrategy::PressAll {
            self.handle_clashes(&mut pressed_actions, input_streams);
        }

        pressed_actions
    }
}

// Utilities
impl<A: Actionlike> InputMap<A> {
    /// Returns the mapping between the `action` that uses the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be returned regardless of input mode.
    ///
    /// For chords, an input will be returned if any of the contained buttons use that input mode.
    ///
    /// If no matching bindings are found, an empty [`PetitSet`] will be returned.
    ///
    /// A copy of the values are returned, rather than a reference to them.
    /// The order of these values is stable, in a first-in, first-out fashion.
    /// Use `self.map.get` or `self.map.get_mut` if you require a reference.
    #[must_use]
    pub fn get(&self, action: A, input_mode: Option<InputMode>) -> PetitSet<UserInput, 16> {
        let full_set = self.map[action.index()].clone();
        if let Some(input_mode) = input_mode {
            let mut matching_set = PetitSet::default();
            for input in full_set.iter() {
                if input.matches_input_mode(input_mode) {
                    matching_set.insert(input.clone());
                }
            }

            if matching_set.is_empty() {
                PetitSet::default()
            } else {
                matching_set
            }
        } else {
            full_set
        }
    }

    /// Returns how many bindings are currently registered for the provided action with the provided [`InputMode`]
    ///
    /// If `None` is provided, a total across all input modes will be provided.
    ///
    /// A maximum of `CAP` bindings across all input modes can be stored for each action,
    /// and insert operations will silently fail if used when `CAP` bindings already exist.
    #[must_use]
    pub fn n_registered(&self, action: A, input_mode: Option<InputMode>) -> u8 {
        self.get(action, input_mode).len() as u8
    }

    /// How many input bindings are registered total?
    ///
    /// For more granular information, use [`InputMap::n_registered`] instead.
    #[must_use]
    pub fn len(&self) -> usize {
        let mut i = 0;
        for action in A::iter() {
            i += self.n_registered(action, None) as usize;
        }
        i
    }

    /// Are any input bindings registered at all?
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Clearing
impl<A: Actionlike> InputMap<A> {
    /// Clears all inputs registered for the `action` that use the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be cleared regardless of input mode.
    ///
    /// For chords, an input will be removed if any of the contained buttons use that input mode.
    ///
    /// Returns all previously registered inputs
    pub fn clear_action(
        &mut self,
        action: A,
        input_mode: Option<InputMode>,
    ) -> PetitSet<UserInput, 16> {
        // FIXME: does not appear to be working correctly
        if let Some(input_mode) = input_mode {
            // Pull out all the matching inputs
            let bindings = self.map[action.index()].clone();
            self.map[action.index()] = PetitSet::default();

            let mut retained_set: PetitSet<UserInput, 16> = PetitSet::default();
            let mut removed_set: PetitSet<UserInput, 16> = PetitSet::default();

            for input in bindings {
                if input.matches_input_mode(input_mode) {
                    removed_set.insert(input);
                } else {
                    retained_set.insert(input);
                }
            }

            // Put back the ones that didn't match
            for input in retained_set.iter() {
                self.insert(action.clone(), input.clone());
            }

            // Cache clashes now, to ensure a clean state
            self.cache_possible_clashes();

            // Return the items that matched
            removed_set
        } else {
            let previous_bindings = self.map[action.index()].clone();
            self.map[action.index()] = PetitSet::default();
            // Cache clashes now, to ensure a clean state
            self.cache_possible_clashes();
            previous_bindings
        }
    }

    /// Clears the input for the `action` with the specified [`InputMode`] at the provided index
    ///
    /// Returns the removed input, if any
    pub fn clear_at(&mut self, action: A, input_mode: InputMode, index: u8) -> Option<UserInput> {
        let mut bindings = self.get(action.clone(), Some(input_mode));
        if (bindings.len() as u8) < index {
            // Not enough matching bindings were found
            return None;
        }

        // Clear out existing mappings for that input mode
        self.clear_action(action.clone(), Some(input_mode));

        // Remove the binding at the provided index
        let removed = bindings.take_at(index as usize);

        // Reinsert the other bindings
        for input in bindings.iter() {
            self.insert(action.clone(), input.clone());
        }

        // Cache clashes now, to ensure a clean state
        self.cache_possible_clashes();

        removed
    }

    /// Clears all inputs that use the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be cleared regardless of input mode.
    ///
    /// For chords, an input will be removed if any of the contained buttons use that input mode.
    ///
    /// Returns the subset of the action map that was removed
    pub fn clear_input_mode(&mut self, input_mode: Option<InputMode>) -> InputMap<A> {
        let mut cleared_input_map = InputMap {
            associated_gamepad: self.associated_gamepad,
            ..Default::default()
        };

        for action in A::iter() {
            for input in self.clear_action(action.clone(), input_mode).iter() {
                // Put back the ones that didn't match
                cleared_input_map.insert(action.clone(), input.clone());
            }
        }

        cleared_input_map
    }
}

mod tests {
    use crate as leafwing_input_manager;
    use crate::prelude::*;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Action {
        Run,
        Jump,
        Hide,
    }

    #[test]
    fn insertion_idempotency() {
        use bevy::input::keyboard::KeyCode;
        use petitset::PetitSet;

        let mut input_map = InputMap::<Action>::default();
        input_map.insert(Action::Run, KeyCode::Space);

        assert_eq!(
            input_map.get(Action::Run, None),
            PetitSet::<UserInput, 16>::from_iter([KeyCode::Space.into()])
        );

        // Duplicate insertions should not change anything
        input_map.insert(Action::Run, KeyCode::Space);
        assert_eq!(
            input_map.get(Action::Run, None),
            PetitSet::<UserInput, 16>::from_iter([KeyCode::Space.into()])
        );
    }

    #[test]
    fn multiple_insertion() {
        use crate::buttonlike_user_input::UserInput;
        use bevy::input::keyboard::KeyCode;
        use petitset::PetitSet;

        let mut input_map_1 = InputMap::<Action>::default();
        input_map_1.insert(Action::Run, KeyCode::Space);
        input_map_1.insert(Action::Run, KeyCode::Return);

        assert_eq!(
            input_map_1.get(Action::Run, None),
            PetitSet::<UserInput, 16>::from_iter([KeyCode::Space.into(), KeyCode::Return.into()])
        );

        let input_map_2 = InputMap::<Action>::new([
            (Action::Run, KeyCode::Space),
            (Action::Run, KeyCode::Return),
        ]);

        assert_eq!(input_map_1, input_map_2);
    }

    #[test]
    pub fn chord_coercion() {
        use crate::input_map::{InputButton, UserInput};
        use bevy::input::keyboard::KeyCode;

        // Single items in a chord should be coerced to a singleton
        let mut input_map_1 = InputMap::<Action>::default();
        input_map_1.insert(Action::Run, KeyCode::Space);

        let mut input_map_2 = InputMap::<Action>::default();
        input_map_2.insert(Action::Run, UserInput::chord([KeyCode::Space]));

        assert_eq!(input_map_1, input_map_2);

        // Empty chords are converted to UserInput::Null, and then ignored
        let mut input_map_3 = InputMap::<Action>::default();
        let empty_vec: Vec<InputButton> = Vec::default();
        input_map_3.insert_chord(Action::Run, empty_vec);

        assert_eq!(input_map_3, InputMap::<Action>::default());
    }

    #[test]
    fn input_clearing() {
        use crate::buttonlike_user_input::{InputButton, InputMode};
        use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode, mouse::MouseButton};

        let mut input_map = InputMap::<Action>::default();
        input_map.insert(Action::Run, KeyCode::Space);

        let one_item_input_map = input_map.clone();

        // Clearing without a specified input mode
        input_map.clear_action(Action::Run, None);
        assert_eq!(input_map, InputMap::default());

        // Clearing with the non-matching input mode
        input_map.insert(Action::Run, KeyCode::Space);
        input_map.clear_action(Action::Run, Some(InputMode::Gamepad));
        input_map.clear_action(Action::Run, Some(InputMode::Mouse));
        assert_eq!(input_map, one_item_input_map);

        // Clearing with the matching input mode
        input_map.clear_action(Action::Run, Some(InputMode::Keyboard));
        assert_eq!(input_map, InputMap::default());

        // Clearing an entire input mode
        input_map.insert_multiple([(Action::Run, KeyCode::Space), (Action::Run, KeyCode::A)]);
        input_map.insert(Action::Hide, KeyCode::RBracket);
        input_map.clear_input_mode(Some(InputMode::Keyboard));
        assert_eq!(input_map, InputMap::default());

        // Other stored inputs should be unaffected
        input_map.insert(Action::Run, KeyCode::Space);
        input_map.insert(Action::Hide, GamepadButtonType::South);
        input_map.insert(Action::Run, MouseButton::Left);
        input_map.clear_input_mode(Some(InputMode::Gamepad));
        input_map.clear_action(Action::Run, Some(InputMode::Mouse));
        assert_eq!(input_map, one_item_input_map);

        // Clearing all inputs works
        input_map.insert(Action::Hide, GamepadButtonType::South);
        input_map.insert(Action::Run, MouseButton::Left);
        let big_input_map = input_map.clone();
        let removed_items = input_map.clear_input_mode(None);
        assert_eq!(input_map, InputMap::default());

        // Items are returned on clearing
        assert_eq!(removed_items, big_input_map);

        // Chords are removed if at least one button matches
        input_map.insert_chord(Action::Run, [KeyCode::A, KeyCode::B]);
        input_map.insert_chord(
            Action::Run,
            [GamepadButtonType::South, GamepadButtonType::West],
        );
        input_map.insert_chord(
            Action::Run,
            [
                InputButton::Gamepad(GamepadButtonType::South),
                InputButton::Keyboard(KeyCode::A),
            ],
        );

        let removed_items = input_map.clear_input_mode(Some(InputMode::Gamepad));
        let mut expected_removed_items = InputMap::default();
        expected_removed_items.insert_chord(
            Action::Run,
            [GamepadButtonType::South, GamepadButtonType::West],
        );
        expected_removed_items.insert_chord(
            Action::Run,
            [
                InputButton::Gamepad(GamepadButtonType::South),
                InputButton::Keyboard(KeyCode::A),
            ],
        );

        assert_eq!(removed_items, expected_removed_items);
    }

    #[test]
    fn reset_to_default() {
        use crate::input_map::InputMode;
        use bevy::input::{gamepad::GamepadButtonType, keyboard::KeyCode};

        let mut input_map = InputMap::default();
        let mut default_keyboard_map = InputMap::default();
        default_keyboard_map.insert(Action::Run, KeyCode::LShift);
        default_keyboard_map.insert_chord(Action::Hide, [KeyCode::LControl, KeyCode::H]);
        let mut default_gamepad_map = InputMap::default();
        default_gamepad_map.insert(Action::Run, GamepadButtonType::South);
        default_gamepad_map.insert(Action::Hide, GamepadButtonType::East);

        // Merging works
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_keyboard_map);

        // Merging is idempotent
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_keyboard_map);

        // Fully default settings
        input_map.merge(&default_gamepad_map);
        let default_input_map = input_map.clone();

        // Changing from the default
        input_map.replace(Action::Jump, KeyCode::J);

        // Clearing all keyboard bindings works as expected
        input_map.clear_input_mode(Some(InputMode::Keyboard));
        assert_eq!(input_map, default_gamepad_map);

        // Resetting to default works
        input_map.merge(&default_keyboard_map);
        assert_eq!(input_map, default_input_map);
    }

    #[test]
    fn gamepad_swapping() {
        use bevy::input::gamepad::Gamepad;

        let mut input_map = InputMap::<Action>::default();
        assert_eq!(input_map.gamepad(), None);

        input_map.set_gamepad(Gamepad(0));
        assert_eq!(input_map.gamepad(), Some(Gamepad(0)));

        input_map.clear_gamepad();
        assert_eq!(input_map.gamepad(), None);
    }

    #[test]
    fn mock_inputs() {
        use crate::buttonlike_user_input::InputStreams;
        use crate::input_map::InputButton;
        use bevy::prelude::*;

        // Setting up the input map
        let mut input_map = InputMap::<Action> {
            // Ignore clashing to isolate tests
            clash_strategy: ClashStrategy::PressAll,
            ..Default::default()
        };
        input_map.set_gamepad(Gamepad(42));

        // Gamepad
        input_map.insert(Action::Run, GamepadButtonType::South);
        input_map.insert_chord(
            Action::Jump,
            [GamepadButtonType::South, GamepadButtonType::North],
        );

        // Keyboard
        input_map.insert(Action::Run, KeyCode::LShift);
        input_map.insert(Action::Hide, KeyCode::LShift);

        // Mouse
        input_map.insert(Action::Run, MouseButton::Left);
        input_map.insert(Action::Jump, MouseButton::Other(42));

        // Cross-device chords
        input_map.insert_chord(
            Action::Hide,
            [
                InputButton::Keyboard(KeyCode::LControl),
                InputButton::Mouse(MouseButton::Left),
            ],
        );

        // Input streams
        let mut gamepad_input_stream = Input::<GamepadButton>::default();
        let mut keyboard_input_stream = Input::<KeyCode>::default();
        let mut mouse_input_stream = Input::<MouseButton>::default();

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        // With no inputs, nothing should be detected
        for action in Action::iter() {
            assert!(!input_map.pressed(action, &input_streams));
        }

        // Pressing the wrong gamepad
        gamepad_input_stream.press(GamepadButton(Gamepad(0), GamepadButtonType::South));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };
        for action in Action::iter() {
            assert!(!input_map.pressed(action, &input_streams));
        }

        // Pressing the correct gamepad
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::South));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams));
        assert!(!input_map.pressed(Action::Jump, &input_streams));

        // Chord
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::South));
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::North));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams));
        assert!(input_map.pressed(Action::Jump, &input_streams));

        // Clearing inputs
        gamepad_input_stream = Input::<GamepadButton>::default();
        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        for action in Action::iter() {
            assert!(!input_map.pressed(action, &input_streams));
        }

        // Keyboard
        keyboard_input_stream.press(KeyCode::LShift);

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams));
        assert!(input_map.pressed(Action::Hide, &input_streams));

        keyboard_input_stream = Input::<KeyCode>::default();

        // Mouse
        mouse_input_stream.press(MouseButton::Left);
        mouse_input_stream.press(MouseButton::Other(42));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams));
        assert!(input_map.pressed(Action::Jump, &input_streams));

        mouse_input_stream = Input::<MouseButton>::default();

        // Cross-device chording
        keyboard_input_stream.press(KeyCode::LControl);
        mouse_input_stream.press(MouseButton::Left);

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Hide, &input_streams));
    }
}
