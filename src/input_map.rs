//! This module contains [`InputMap`] and its supporting methods and impls.

use crate::action_state::{Timing, VirtualButtonState};
use crate::buttonlike_user_input::{InputButton, InputStreams, UserInput};
use crate::clashing_inputs::ClashStrategy;
use crate::Actionlike;
use bevy::prelude::*;
use core::fmt::Debug;
use petitset::PetitSet;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::slice::Iter;

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
/// Set the [`ClashStrategy`](crate::clashing_inputs::ClashStrategy) resource
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
///
/// // But you can't Hide :(
/// input_map.clear_action(Action::Hide);
///```
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputMap<A: Actionlike> {
    /// The raw vector of [PetitSet]s used to store the input mapping,
    /// indexed by the `Actionlike::id` of `A`
    map: Vec<PetitSet<UserInput, 16>>,
    associated_gamepad: Option<Gamepad>,
    #[serde(skip)]
    marker: PhantomData<A>,
}

impl<A: Actionlike> Default for InputMap<A> {
    fn default() -> Self {
        InputMap {
            map: A::variants().map(|_| PetitSet::default()).collect(),
            associated_gamepad: None,
            marker: PhantomData,
        }
    }
}

impl<'a, A: Actionlike> IntoIterator for &'a InputMap<A> {
    type Item = &'a PetitSet<UserInput, 16>;
    type IntoIter = Iter<'a, PetitSet<UserInput, 16>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
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
    /// # Panics
    ///
    /// Panics if the map is full and `input` is not a duplicate.
    pub fn insert(&mut self, action: A, input: impl Into<UserInput>) -> &mut Self {
        let input = input.into();

        self.map[action.index()].insert(input);

        self
    }

    /// Insert a mapping between `action` and `input` at the provided index
    ///
    /// If a matching input already existed in the set, it will be moved to the supplied index. Any input that was previously there will be moved to the matching input’s original index.
    ///
    /// # Panics
    ///
    /// Panics if the map is full and `input` is not a duplicate.
    pub fn insert_at(&mut self, action: A, input: impl Into<UserInput>, index: usize) -> &mut Self {
        let input = input.into();

        self.map[action.index()].insert_at(input, index);

        self
    }

    /// Insert a mapping between `action` and the provided `inputs`
    ///
    /// This method creates multiple distinct bindings.
    /// If you want to require multiple buttons to be pressed at once, use [`insert_chord`](Self::insert_chord).
    /// Any iterator that can be converted into a [`UserInput`] can be supplied.
    ///
    /// # Panics
    ///
    /// Panics if the map is full and any of `inputs` is not a duplicate.
    pub fn insert_multiple(
        &mut self,
        inputs: impl IntoIterator<Item = (A, impl Into<UserInput>)>,
    ) -> &mut Self {
        for (action, input) in inputs {
            self.insert(action, input);
        }

        self
    }

    /// Insert a mapping between `action` and the simultaneous combination of `buttons` provided
    ///
    /// Any iterator that can be converted into a [`Button`] can be supplied, but will be converted into a [`PetitSet`] for storage and use.
    /// Chords can also be added with the [insert](Self::insert) method, if the [`UserInput::Chord`] variant is constructed explicitly.
    ///
    /// # Panics
    ///
    /// Panics if the map is full and `buttons` is not a duplicate.
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

        for action in A::variants() {
            for input in self.get(action.clone()).iter() {
                new_map.insert(action.clone(), input.clone());
            }

            for input in other.get(action.clone()).iter() {
                new_map.insert(action.clone(), input.clone());
            }
        }

        *self = new_map;
        self
    }
}

// Configuration
impl<A: Actionlike> InputMap<A> {
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
        action: A,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> bool {
        let pressed_list = self.which_pressed(input_streams, clash_strategy);
        pressed_list[action.index()].pressed()
    }

    /// Returns a [`HashSet`] of the virtual buttons that are currently pressed
    ///
    /// Accounts for clashing inputs according to the [`ClashStrategy`].
    /// The `usize`s returned correspond to `Actionlike::index()`.
    #[must_use]
    pub fn which_pressed(
        &self,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) -> Vec<VirtualButtonState> {
        let mut pressed_actions = vec![VirtualButtonState::default(); A::N_VARIANTS];

        // Generate the raw action presses
        for action in A::variants() {
            let mut inputs = Vec::new();

            for input in self.get(action.clone()).iter() {
                if input_streams.input_pressed(input) {
                    inputs.push(input.clone());
                }
            }

            if !inputs.is_empty() {
                pressed_actions[action.index()] =
                    VirtualButtonState::Pressed(Timing::default(), inputs);
            }
        }

        // Handle clashing inputs, possibly removing some pressed actions from the list
        self.handle_clashes(&mut pressed_actions, input_streams, clash_strategy);

        pressed_actions
    }
}

// Utilities
impl<A: Actionlike> InputMap<A> {
    /// Iterate over mapped inputs
    pub fn iter(&self) -> impl Iterator<Item = &PetitSet<UserInput, 16>> {
        self.map.iter()
    }

    /// Returns the `action` mappings
    #[must_use]
    pub fn get(&self, action: A) -> &PetitSet<UserInput, 16> {
        &self.map[action.index()]
    }

    /// How many input bindings are registered total?
    #[must_use]
    pub fn len(&self) -> usize {
        let mut i = 0;
        for action in A::variants() {
            i += self.get(action).len();
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

// Removing
impl<A: Actionlike> InputMap<A> {
    /// Clears all inputs registered for the `action`
    pub fn clear_action(&mut self, action: A) {
        self.map[action.index()].clear();
    }

    /// Removes the input for the `action` at the provided index
    ///
    /// Returns `true` if an element was found.
    pub fn remove_at(&mut self, action: A, index: usize) -> bool {
        self.map[action.index()].remove_at(index)
    }

    /// Removes the input for the `action`, if it exists
    ///
    /// Returns [`Some`] with index if the input was found, or [`None`] if no matching input was found.
    pub fn remove(&mut self, action: A, input: impl Into<UserInput>) -> Option<usize> {
        self.map[action.index()].remove(&input.into())
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
            *input_map.get(Action::Run),
            PetitSet::<UserInput, 16>::from_iter([KeyCode::Space.into()])
        );

        // Duplicate insertions should not change anything
        input_map.insert(Action::Run, KeyCode::Space);
        assert_eq!(
            *input_map.get(Action::Run),
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
            *input_map_1.get(Action::Run),
            PetitSet::<UserInput, 16>::from_iter([KeyCode::Space.into(), KeyCode::Return.into()])
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
        input_map.clear_action(Action::Run);
        assert_eq!(input_map, InputMap::default());

        // Remove input at existing index
        input_map.insert(Action::Run, KeyCode::Space);
        input_map.insert(Action::Run, KeyCode::LShift);
        assert!(input_map.remove_at(Action::Run, 1));
        assert!(
            !input_map.remove_at(Action::Run, 1),
            "Should return false on second removal at the same index"
        );
        assert!(input_map.remove_at(Action::Run, 0));
        assert!(
            !input_map.remove_at(Action::Run, 0),
            "Should return false on second removal at the same index"
        );
    }

    #[test]
    fn merging() {
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
        let mut input_map = InputMap::<Action>::default();
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
        for action in Action::variants() {
            assert!(!input_map.pressed(action, &input_streams, ClashStrategy::PressAll));
        }

        // Pressing the wrong gamepad
        gamepad_input_stream.press(GamepadButton(Gamepad(0), GamepadButtonType::South));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };
        for action in Action::variants() {
            assert!(!input_map.pressed(action, &input_streams, ClashStrategy::PressAll));
        }

        // Pressing the correct gamepad
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::South));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams, ClashStrategy::PressAll));
        assert!(!input_map.pressed(Action::Jump, &input_streams, ClashStrategy::PressAll));

        // Chord
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::South));
        gamepad_input_stream.press(GamepadButton(Gamepad(42), GamepadButtonType::North));

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams, ClashStrategy::PressAll));
        assert!(input_map.pressed(Action::Jump, &input_streams, ClashStrategy::PressAll));

        // Clearing inputs
        gamepad_input_stream = Input::<GamepadButton>::default();
        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        for action in Action::variants() {
            assert!(!input_map.pressed(action, &input_streams, ClashStrategy::PressAll));
        }

        // Keyboard
        keyboard_input_stream.press(KeyCode::LShift);

        let input_streams = InputStreams {
            gamepad: Some(&gamepad_input_stream),
            keyboard: Some(&keyboard_input_stream),
            mouse: Some(&mouse_input_stream),
            associated_gamepad: Some(Gamepad(42)),
        };

        assert!(input_map.pressed(Action::Run, &input_streams, ClashStrategy::PressAll));
        assert!(input_map.pressed(Action::Hide, &input_streams, ClashStrategy::PressAll));

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

        assert!(input_map.pressed(Action::Run, &input_streams, ClashStrategy::PressAll));
        assert!(input_map.pressed(Action::Jump, &input_streams, ClashStrategy::PressAll));

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

        assert!(input_map.pressed(Action::Hide, &input_streams, ClashStrategy::PressAll));
    }
}
