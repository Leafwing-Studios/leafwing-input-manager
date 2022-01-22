//! This module contains [`InputMap`] and its supporting methods and impls.

use crate::Actionlike;
use bevy::prelude::*;
use bevy::utils::HashMap;
use core::fmt::Debug;
use petitset::PetitSet;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

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
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
/// use leafwing_input_manager::input_map::Button;
/// use strum_macros::EnumIter;
///
/// // You can Run!
/// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
/// enum Action {
///     Run,
///     Hide,
/// }
///
/// let mut input_map: InputMap<Action> = InputMap::default();
///
/// // Basic insertion
/// input_map.insert(Action::Run, GamepadButtonType::South);
/// input_map.insert(Action::Run, MouseButton::Left);
/// input_map.insert(Action::Run, KeyCode::LShift);
/// input_map.insert_multiple(Action::Hide, [GamepadButtonType::LeftTrigger, GamepadButtonType::RightTrigger]);
///
/// // Combinations!
/// input_map.insert_chord(Action::Run, [KeyCode::LControl, KeyCode::R]);
/// input_map.insert_chord(Action::Hide, [Button::Keyboard(KeyCode::H), Button::Gamepad(GamepadButtonType::South), Button::Mouse(MouseButton::Middle)]);
///
/// // But you can't Hide :(
/// input_map.clear_action(Action::Hide, None);
///```
#[derive(Component, Debug, Clone, PartialEq)]
pub struct InputMap<A: Actionlike> {
    /// The raw [HashMap] of [PetitSet]s used to store the input mapping
    pub map: HashMap<A, PetitSet<UserInput, 16>>,
    per_mode_cap: Option<usize>,
    associated_gamepad: Option<Gamepad>,
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            associated_gamepad: None,
            per_mode_cap: None,
        }
    }
}

// Constructors
impl<A: Actionlike> InputMap<A> {
    /// Creates a new empty [`InputMap`]
    ///
    /// The `per_mode_cap` controls the maximum number of inputs of each [`InputMode`] that can be stored.
    /// If a value of 0 is supplied, no cap will be provided (although the global `CAP` must still be obeyed).
    ///
    /// PANICS: `3 * per_mode_cap` cannot exceed the global `CAP`, as we need space to store all mappings.
    pub fn new(per_mode_cap: usize) -> Self {
        if per_mode_cap == 0 {
            Self::default()
        } else {
            let mut input_map = Self::default();
            input_map.set_per_mode_cap(per_mode_cap);
            input_map
        }
    }
}

// Check whether buttons are pressed
impl<A: Actionlike> InputMap<A> {
    /// Is at least one of the corresponding inputs for `action` found in the provided `input` stream?
    #[must_use]
    pub fn pressed(
        &self,
        action: A,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        if let Some(matching_inputs) = self.map.get(&action) {
            self.any_pressed(
                matching_inputs,
                gamepad_input_stream,
                keyboard_input_stream,
                mouse_input_stream,
            )
        } else {
            // No matches can be found if no inputs are registred for that action
            false
        }
    }

    /// Is at least one of the `inputs` pressed?
    #[must_use]
    pub fn any_pressed(
        &self,
        inputs: &PetitSet<UserInput, 16>,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        for input in inputs.iter() {
            if match input {
                UserInput::Single(button) => self.button_pressed(
                    *button,
                    gamepad_input_stream,
                    keyboard_input_stream,
                    mouse_input_stream,
                ),
                UserInput::Chord(buttons) => self.all_buttons_pressed(
                    buttons,
                    gamepad_input_stream,
                    keyboard_input_stream,
                    mouse_input_stream,
                ),
                UserInput::Null => false,
            } {
                // If any of the appropriate inputs match, the action is considered pressed
                return true;
            }
        }
        // If none of the inputs matched, return false
        false
    }

    /// Is the `button` pressed?
    #[must_use]
    pub fn button_pressed(
        &self,
        button: Button,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        match button {
            Button::Gamepad(gamepad_button) => {
                // If no gamepad is registered, we know for sure that no match was found
                if let Some(gamepad) = self.associated_gamepad {
                    gamepad_input_stream.pressed(GamepadButton(gamepad, gamepad_button))
                } else {
                    false
                }
            }
            Button::Keyboard(keycode) => keyboard_input_stream.pressed(keycode),
            Button::Mouse(mouse_button) => mouse_input_stream.pressed(mouse_button),
        }
    }

    /// Are all of the `buttons` pressed?
    #[must_use]
    pub fn all_buttons_pressed(
        &self,
        buttons: &PetitSet<Button, 8>,
        gamepad_input_stream: &Input<GamepadButton>,
        keyboard_input_stream: &Input<KeyCode>,
        mouse_input_stream: &Input<MouseButton>,
    ) -> bool {
        for &button in buttons.iter() {
            // If any of the appropriate inputs failed to match, the action is considered pressed
            if !self.button_pressed(
                button,
                gamepad_input_stream,
                keyboard_input_stream,
                mouse_input_stream,
            ) {
                return false;
            }
        }
        // If none of the inputs failed to match, return true
        true
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
        if let Some(full_set) = self.map.get(&action) {
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
                full_set.clone()
            }
        } else {
            PetitSet::default()
        }
    }

    /// Returns how many bindings are currently registered for the provided action with the provided [`InputMode`]
    ///
    /// If `None` is provided, a total across all input modes will be provided.
    ///
    /// A maximum of `CAP` bindings across all input modes can be stored for each action,
    /// and insert operations will silently fail if used when `CAP` bindings already exist.
    #[must_use]
    pub fn n_registered(&self, action: A, input_mode: Option<InputMode>) -> usize {
        self.get(action, input_mode).len()
    }
}

// Insertion
impl<A: Actionlike> InputMap<A> {
    /// Insert a mapping between `action` and `input`
    ///
    /// Existing mappings for that action will not be overwritten.
    /// If the set for this action is already full, this insertion will silently fail.
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) {
        let input = input.into();

        // Don't insert Null inputs into the map
        if input == UserInput::Null {
            return;
        }

        // Don't overflow the set!
        if self.n_registered(action, None) >= 16 {
            return;
        }

        // Respect any per-input-mode caps that have been set
        if let Some(per_mode_cap) = self.per_mode_cap {
            for input_mode in input.input_modes() {
                if self.n_registered(action, Some(input_mode)) >= per_mode_cap {
                    return;
                }
            }
        }

        if let Some(existing_set) = self.map.get_mut(&action) {
            existing_set.insert(input);
        } else {
            let mut new_set = PetitSet::default();
            new_set.insert(input);
            self.map.insert(action, new_set);
        }
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
        action: A,
        inputs: impl IntoIterator<Item = impl Into<UserInput>>,
    ) {
        for input in inputs {
            self.insert(action, input);
        }
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
        buttons: impl IntoIterator<Item = impl Into<Button>>,
    ) {
        self.insert(action, UserInput::chord(buttons));
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
            if let Some(removed_inputs) = self.clear_action(action, Some(input_mode)) {
                for removed_input in removed_inputs {
                    old_inputs.insert(removed_input);
                }
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
        index: usize,
    ) -> Option<UserInput> {
        let input = input.into();
        let input_modes = input.input_modes();

        if input_modes.len() != 1 {
            return None;
        }

        // We know that the input belongs to exactly one mode
        let input_mode = input_modes.into_iter().next().unwrap();
        let removed = self.clear_at(action, input_mode, index);
        self.insert(action, input);

        removed
    }

    /// Merges the provided [`InputMap`] into the [`InputMap`] this method was called on
    ///
    /// This adds both of their bindings to the resulting [`InputMap`].
    /// Like usual, any duplicate bindings are ignored.
    ///
    /// If the associated gamepads do not match, the resulting associated gamepad will be set to `None`.
    pub fn merge(&mut self, other: &InputMap<A>) {
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
            new_map.insert_multiple(action, self.get(action, None));
            new_map.insert_multiple(action, other.get(action, None));
        }

        *self = new_map;
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
    /// Returns all previously registered inputs, if any
    pub fn clear_action(
        &mut self,
        action: A,
        input_mode: Option<InputMode>,
    ) -> Option<PetitSet<UserInput, 16>> {
        if let Some(input_mode) = input_mode {
            // Pull out all the matching inputs
            if let Some(bindings) = self.map.remove(&action) {
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
                self.insert_multiple(action, retained_set);

                // Return the items that matched
                if removed_set.is_empty() {
                    None
                } else {
                    Some(removed_set)
                }
            } else {
                None
            }
        } else {
            self.map.remove(&action)
        }
    }

    /// Clears the input for the `action` with the specified [`InputMode`] at the provided index
    ///
    /// Returns the removed input, if any
    pub fn clear_at(
        &mut self,
        action: A,
        input_mode: InputMode,
        index: usize,
    ) -> Option<UserInput> {
        let mut bindings = self.get(action, Some(input_mode));
        if bindings.len() < index {
            // Not enough matching bindings were found
            return None;
        }

        // Clear out existing mappings for that input mode
        self.clear_action(action, Some(input_mode));

        // Remove the binding at the provided index
        let removed = bindings.take_at(index);

        // Reinsert the other bindings
        self.insert_multiple(action, bindings);

        removed
    }

    /// Clears all inputs that use the supplied `input_mode`
    ///
    /// If `input_mode` is `None`, all inputs will be cleared regardless of input mode.
    ///
    /// For chords, an input will be removed if any of the contained buttons use that input mode.
    ///
    /// Returns the subset of the action map that was removed
    #[allow(clippy::return_self_not_must_use)]
    pub fn clear_input_mode(&mut self, input_mode: Option<InputMode>) -> InputMap<A> {
        let mut cleared_input_map = InputMap {
            associated_gamepad: self.associated_gamepad,
            ..Default::default()
        };

        for action in A::iter() {
            if let Some(removed_inputs) = self.clear_action(action, input_mode) {
                cleared_input_map.insert_multiple(action, removed_inputs);
            }
        }

        cleared_input_map
    }
}

// Per-mode cap
impl<A: Actionlike> InputMap<A> {
    /// Returns the per-[`InputMode`] cap on input bindings for every action
    ///
    /// Each individual action can have at most this many bindings, making them easier to display and configure.
    pub fn per_mode_cap(&self) -> usize {
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
    #[allow(clippy::return_self_not_must_use)]
    pub fn set_per_mode_cap(&mut self, per_mode_cap: usize) -> InputMap<A> {
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
                let n_registered = self.n_registered(action, Some(input_mode));
                if n_registered > per_mode_cap {
                    for i in per_mode_cap..n_registered {
                        let removed_input = self.clear_at(action, input_mode, i);
                        if let Some(input) = removed_input {
                            removed_actions.insert(action, input);
                        }
                    }
                }
            }
        }

        removed_actions
    }
}

// Gamepads
impl<A: Actionlike> InputMap<A> {
    /// Assigns a particular [`Gamepad`] to the entity controlled by this input map
    pub fn assign_gamepad(&mut self, gamepad: Gamepad) {
        self.associated_gamepad = Some(gamepad);
    }

    /// Clears any [Gamepad] associated with the entity controlled by this input map
    pub fn clear_gamepad(&mut self) {
        self.associated_gamepad = None;
    }

    /// Fetches the [Gamepad] associated with the entity controlled by this entity map
    #[must_use]
    pub fn gamepad(&self) -> Option<Gamepad> {
        self.associated_gamepad
    }
}

/// Some combination of user input, which may cross [`Input`] boundaries
///
/// Suitable for use in an [`InputMap`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInput {
    /// A single button
    Single(Button),
    /// A combination of buttons, pressed simultaneously
    ///
    /// Up to 8 (!!) buttons can be chorded together at once.
    /// Chords are considered to belong to all of the [InputMode]s of their constituent buttons.
    Chord(PetitSet<Button, 8>),
    /// A null user input, used for a safe default and error-handling
    ///
    /// This input can never be pressed.
    Null,
}

impl Default for UserInput {
    fn default() -> Self {
        UserInput::Null
    }
}

impl UserInput {
    /// Creates a [`UserInput::Chord`] from an iterator of [`Button`]s
    ///
    /// If `buttons` has a length of 1, a [`UserInput::Single`] variant will be returned instead.
    /// If `buttons` has a length of 0, a [`UserInput::Null`] variant will be returned instead.
    pub fn chord(buttons: impl IntoIterator<Item = impl Into<Button>>) -> Self {
        // We can't just check the length unless we add an ExactSizeIterator bound :(
        let mut length: u8 = 0;

        let mut set: PetitSet<Button, 8> = PetitSet::default();
        for button in buttons {
            length += 1;
            set.insert(button.into());
        }

        match length {
            0 => UserInput::Null,
            1 => UserInput::Single(set.into_iter().next().unwrap()),
            _ => UserInput::Chord(set),
        }
    }

    /// Which [`InputMode`]s does this input contain?
    pub fn input_modes(&self) -> PetitSet<InputMode, 3> {
        let mut set = PetitSet::default();
        match self {
            UserInput::Null => (),
            UserInput::Single(button) => {
                set.insert((*button).into());
            }
            UserInput::Chord(buttons) => {
                for &button in buttons.iter() {
                    set.insert(button.into());
                }
            }
        }
        set
    }

    /// Does this [`UserInput`] match the provided [`InputMode`]?
    ///
    /// For [`UserInput::Chord`], this will be true if any of the buttons in the combination match.
    pub fn matches_input_mode(&self, input_mode: InputMode) -> bool {
        // This is slightly faster than using Self::input_modes
        // As we can return early
        match self {
            UserInput::Single(button) => {
                let button_mode: InputMode = (*button).into();
                button_mode == input_mode
            }
            UserInput::Chord(set) => {
                for button in set.iter() {
                    let button_mode: InputMode = (*button).into();
                    if button_mode == input_mode {
                        return true;
                    }
                }
                false
            }
            UserInput::Null => false,
        }
    }
}

impl From<Button> for UserInput {
    fn from(input: Button) -> Self {
        UserInput::Single(input)
    }
}

impl From<GamepadButtonType> for UserInput {
    fn from(input: GamepadButtonType) -> Self {
        UserInput::Single(Button::Gamepad(input))
    }
}

impl From<KeyCode> for UserInput {
    fn from(input: KeyCode) -> Self {
        UserInput::Single(Button::Keyboard(input))
    }
}

impl From<MouseButton> for UserInput {
    fn from(input: MouseButton) -> Self {
        UserInput::Single(Button::Mouse(input))
    }
}

/// A button-like input type
///
/// See [`Button`] for the value-ful equivalent.
/// Use the [`From`] or [`Into`] traits to convert from a [`Button`] to a [`InputMode`].
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum InputMode {
    /// A gamepad
    Gamepad,
    /// A keyboard
    Keyboard,
    /// A mouse
    Mouse,
}

impl From<Button> for InputMode {
    fn from(button: Button) -> Self {
        match button {
            Button::Gamepad(_) => InputMode::Gamepad,
            Button::Keyboard(_) => InputMode::Keyboard,
            Button::Mouse(_) => InputMode::Mouse,
        }
    }
}

/// The values of a button-like input type
///
/// See [`InputMode`] for the value-less equivalent. Commonly stored in the [`UserInput`] enum.
///
/// Unfortunately we cannot use a trait object here, as the types used by `Input`
/// require traits that are not object-safe.
///
/// Please contact the maintainers if you need support for another type!
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// A button on a gamepad
    Gamepad(GamepadButtonType),
    /// A button on a keyboard
    Keyboard(KeyCode),
    /// A button on a mouse
    Mouse(MouseButton),
}

impl From<GamepadButtonType> for Button {
    fn from(input: GamepadButtonType) -> Self {
        Button::Gamepad(input)
    }
}

impl From<KeyCode> for Button {
    fn from(input: KeyCode) -> Self {
        Button::Keyboard(input)
    }
}

impl From<MouseButton> for Button {
    fn from(input: MouseButton) -> Self {
        Button::Mouse(input)
    }
}

mod tests {
    use super::UserInput;
    use crate::prelude::*;
    use strum_macros::EnumIter;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
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
        use crate::input_map::{Button, UserInput};
        use bevy::input::keyboard::KeyCode;
        use petitset::PetitSet;

        let mut input_map_1 = InputMap::<Action>::default();
        input_map_1.insert(Action::Run, KeyCode::Space);
        input_map_1.insert(Action::Run, KeyCode::Return);

        assert_eq!(
            input_map_1.get(Action::Run, None),
            PetitSet::<UserInput, 16>::from_iter([KeyCode::Space.into(), KeyCode::Return.into()])
        );

        let mut input_map_2 = InputMap::<Action>::default();
        input_map_2.insert_multiple(Action::Run, [KeyCode::Space, KeyCode::Return]);

        assert_eq!(input_map_1, input_map_2);

        let mut input_map_3 = InputMap::<Action>::default();
        input_map_3.insert_multiple(Action::Run, [KeyCode::Return, KeyCode::Space]);

        assert_eq!(input_map_1, input_map_3);

        let mut input_map_4 = InputMap::<Action>::default();
        input_map_4.insert_multiple(
            Action::Run,
            [
                Button::Keyboard(KeyCode::Space),
                Button::Keyboard(KeyCode::Return),
            ],
        );

        assert_eq!(input_map_1, input_map_4);

        let mut input_map_5 = InputMap::<Action>::default();
        input_map_5.insert_multiple(
            Action::Run,
            [
                UserInput::Single(Button::Keyboard(KeyCode::Space)),
                UserInput::Single(Button::Keyboard(KeyCode::Return)),
            ],
        );

        assert_eq!(input_map_1, input_map_5);
    }

    #[test]
    pub fn chord_coercion() {
        use crate::input_map::{Button, UserInput};
        use bevy::input::keyboard::KeyCode;

        // Single items in a chord should be coerced to a singleton
        let mut input_map_1 = InputMap::<Action>::default();
        input_map_1.insert(Action::Run, KeyCode::Space);

        let mut input_map_2 = InputMap::<Action>::default();
        input_map_2.insert(Action::Run, UserInput::chord([KeyCode::Space]));

        assert_eq!(input_map_1, input_map_2);

        // Empty chords are converted to UserInput::Null, and then ignored
        let mut input_map_3 = InputMap::<Action>::default();
        let empty_vec: Vec<Button> = Vec::default();
        input_map_3.insert_chord(Action::Run, empty_vec);

        assert_eq!(input_map_3, InputMap::<Action>::default());
    }

    #[test]
    fn input_clearing() {
        use crate::input_map::{Button, InputMode};
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
        input_map.insert_multiple(Action::Run, [KeyCode::Space, KeyCode::A]);
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
                Button::Gamepad(GamepadButtonType::South),
                Button::Keyboard(KeyCode::A),
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
                Button::Gamepad(GamepadButtonType::South),
                Button::Keyboard(KeyCode::A),
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

        input_map.assign_gamepad(Gamepad(0));
        assert_eq!(input_map.gamepad(), Some(Gamepad(0)));

        input_map.clear_gamepad();
        assert_eq!(input_map.gamepad(), None);
    }

    #[test]
    fn mock_inputs() {
        use crate::input_map::Button;
        use bevy::prelude::*;
        use strum::IntoEnumIterator;

        // Setting up the input map
        let mut input_map = InputMap::<Action>::default();
        input_map.assign_gamepad(Gamepad(42));

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
                Button::Keyboard(KeyCode::LControl),
                Button::Mouse(MouseButton::Left),
            ],
        );

        // Input streams
        let mut gamepad_input_stream = Input::<GamepadButton>::default();
        let mut keyboard_input_stream = Input::<KeyCode>::default();
        let mut mouse_input_stream = Input::<MouseButton>::default();

        // With no inputs, nothing should be detected
        for action in Action::iter() {
            assert!(!input_map.pressed(
                action,
                &gamepad_input_stream,
                &keyboard_input_stream,
                &mouse_input_stream,
            ));
        }

        // Pressing the wrong gamepad
        gamepad_input_stream.press(GamepadButton(Gamepad(0), GamepadButtonType::South));
        for action in Action::iter() {
            assert!(!input_map.pressed(
                action,
                &gamepad_input_stream,
                &keyboard_input_stream,
                &mouse_input_stream,
            ));
        }

        // Pressing the correct gamepad
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::South));
        assert!(input_map.pressed(
            Action::Run,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));
        assert!(!input_map.pressed(
            Action::Jump,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));

        // Chord
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::North));
        assert!(input_map.pressed(
            Action::Run,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));
        assert!(input_map.pressed(
            Action::Jump,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));

        // Clearing inputs
        gamepad_input_stream = Input::<GamepadButton>::default();
        for action in Action::iter() {
            assert!(!input_map.pressed(
                action,
                &gamepad_input_stream,
                &keyboard_input_stream,
                &mouse_input_stream,
            ));
        }

        // Keyboard
        keyboard_input_stream.press(KeyCode::LShift);
        assert!(input_map.pressed(
            Action::Run,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));
        assert!(input_map.pressed(
            Action::Hide,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));

        keyboard_input_stream = Input::<KeyCode>::default();

        // Mouse
        mouse_input_stream.press(MouseButton::Left);
        mouse_input_stream.press(MouseButton::Other(42));

        assert!(input_map.pressed(
            Action::Run,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));
        assert!(input_map.pressed(
            Action::Jump,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));

        mouse_input_stream = Input::<MouseButton>::default();

        // Cross-device chording
        keyboard_input_stream.press(KeyCode::LControl);
        mouse_input_stream.press(MouseButton::Left);
        assert!(input_map.pressed(
            Action::Hide,
            &gamepad_input_stream,
            &keyboard_input_stream,
            &mouse_input_stream,
        ));
    }
}
