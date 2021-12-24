use bevy::input::InputSystem;
use bevy::prelude::*;
use bevy::utils::HashMap;

use core::fmt::Display;
use core::hash::Hash;
use core::marker::PhantomData;
use multimap::MultiMap;
use strum::IntoEnumIterator;

/// [Plugin] that collects [Input] from disparate sources, producing an [ActionState] to consume in game logic
///
/// Resources
/// - various [InputMap] resources ([KeyCode], [GamepadButton] and [MouseButton])
/// - a central [ActionState] resource, which stores the current input state in an source-agnostic fashion
///
/// Systems:
/// - [reset_action_state], which resets the action state each frame
/// - [update_action_state] and [update_action_state_gamepads], which collects the input from the corresponding input type to update the action state
///     - labeled [InputMapSystem]
pub struct InputManagerPlugin<InputAction: InputActionEnum> {
    _phantom: PhantomData<InputAction>,
}

// Manual impl is required as we do not want a Default bound on our generic type
impl<InputAction: InputActionEnum> Default for InputManagerPlugin<InputAction> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }
}

/// A type that can be used to represent input-agnostic action representation
///
/// This trait should be implemented on the `InputAction` type that you want to pass into [InputManagerPlugin]
pub trait InputActionEnum:
    Send + Sync + Copy + Eq + Hash + IntoEnumIterator + Display + 'static
{
}

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub struct InputMapSystem;

impl<InputAction: InputActionEnum> Plugin for InputManagerPlugin<InputAction> {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMap<InputAction, KeyCode>>()
            .init_resource::<InputMap<InputAction, MouseButton>>()
            .init_resource::<InputMap<InputAction, GamepadButton, GamepadButtonType>>()
            .init_resource::<ActionState<InputAction>>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(update_action_state::<InputAction, KeyCode>)
                    .with_system(update_action_state::<InputAction, MouseButton>)
                    .with_system(update_action_state_gamepads::<InputAction>)
                    .label(InputMapSystem)
                    .after(InputSystem),
            );
    }
}

/// Maps from raw inputs to an input-method agnostic representation
///
/// Multiple inputs of the same type can be mapped to the same action.
/// A seperate resource of this type will be required for each input method you wish to support.
///
/// In almost all cases, the `InputType` type parameter (e.g. `Keycode`) will be the same as the
/// `InputVariant` type parameter: gamepads are the only common exception.
pub struct InputMap<InputAction, InputType, InputVariant = InputType>
where
    InputAction: InputActionEnum,
    InputVariant: Copy + Hash + Eq,
{
    mmap: MultiMap<InputAction, InputVariant>,
    _phantom: PhantomData<InputType>,
}

impl<InputAction, InputType, InputVariant> Default
    for InputMap<InputAction, InputType, InputVariant>
where
    InputAction: InputActionEnum,
    InputVariant: Copy + Hash + Eq,
{
    fn default() -> Self {
        Self {
            mmap: MultiMap::default(),
            _phantom: PhantomData::default(),
        }
    }
}

impl<InputAction, InputType, InputVariant> InputMap<InputAction, InputType, InputVariant>
where
    InputAction: InputActionEnum,
    InputVariant: Copy + Hash + Eq,
{
    /// Maps a particular `input` to the provided `action`
    ///
    /// This is commonly used to configure new inputs.
    pub fn insert(&mut self, action: InputAction, input: InputVariant) {
        self.mmap.insert(action, input);
    }

    /// Removes an 'action' from the map, returning the vector of 'input' at the key if the key was previously in the map.
    ///
    /// This can be used to reset keybindings in a granular fashion.
    pub fn remove(&mut self, action: InputAction) {
        self.mmap.remove(&action);
    }
}

// This handles the simple case, where InputVariant == InputType
// See https://github.com/bevyengine/bevy/issues/3224 for why these aren't always the same
impl<InputAction, InputType> InputMap<InputAction, InputType>
where
    InputAction: InputActionEnum,
    InputType: Copy + Hash + Eq,
{
    pub fn pressed(&self, action: InputAction, input: &Input<InputType>) -> bool {
        let presses = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for press in presses {
            if input.pressed(*press) {
                return true;
            }
        }
        false
    }

    pub fn just_pressed(&self, action: InputAction, input: &Input<InputType>) -> bool {
        let presses = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for press in presses {
            if input.just_pressed(*press) {
                return true;
            }
        }
        false
    }
}

// Special-cased impl required due to https://github.com/bevyengine/bevy/issues/3224
impl<InputAction: InputActionEnum> InputMap<InputAction, GamepadButton, GamepadButtonType> {
    pub fn pressed(
        &self,
        action: InputAction,
        input: &Input<GamepadButton>,
        gamepad: Gamepad,
    ) -> bool {
        let button_types = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for &button_type in button_types {
            let gamepad_button = GamepadButton(gamepad, button_type);

            if input.pressed(gamepad_button) {
                return true;
            }
        }
        false
    }

    pub fn just_pressed(
        &self,
        action: InputAction,
        input: &Input<GamepadButton>,
        gamepad: Gamepad,
    ) -> bool {
        let button_types = self
            .mmap
            .get_vec(&action)
            .unwrap_or_else(|| panic!("No bindings found for {}", action));

        for &button_type in button_types {
            let gamepad_button = GamepadButton(gamepad, button_type);

            if input.just_pressed(gamepad_button) {
                return true;
            }
        }
        false
    }
}

/// Resource that stores the currently and recently pressed actions
///
/// Abstracts over all of the various input methods and bindings
pub struct ActionState<InputAction: InputActionEnum> {
    pressed: HashMap<InputAction, bool>,
    just_pressed: HashMap<InputAction, bool>,
}

impl<InputAction: InputActionEnum> ActionState<InputAction> {
    pub fn pressed(&self, action: InputAction) -> bool {
        *self.pressed.get(&action).unwrap()
    }

    pub fn just_pressed(&self, action: InputAction) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }
}

impl<InputAction: InputActionEnum> Default for ActionState<InputAction> {
    fn default() -> Self {
        let mut pressed = HashMap::<InputAction, bool>::default();
        let mut just_pressed = HashMap::<InputAction, bool>::default();

        for action in InputAction::iter() {
            pressed.insert(action, false);
            just_pressed.insert(action, false);
        }

        Self {
            pressed,
            just_pressed,
        }
    }
}

/// Resets the action state to "nothing is pressed" each frame
pub fn reset_action_state<InputAction: InputActionEnum>(
    mut action_state: ResMut<ActionState<InputAction>>,
) {
    *action_state = ActionState::default();
}

/// Fetches an [Input] resource to update [ActionState] according to the [InputMap]
pub fn update_action_state<
    InputAction: InputActionEnum,
    InputType: Send + Sync + Copy + Hash + Eq + 'static,
>(
    input: Res<Input<InputType>>,
    input_map: Res<InputMap<InputAction, InputType>>,
    mut action_state: ResMut<ActionState<InputAction>>,
) {
    for action in InputAction::iter() {
        // A particular input type can add to the action state, but cannot revert it
        if input_map.pressed(action, &*input) {
            action_state.pressed.insert(action, true);
        }

        if input_map.just_pressed(action, &*input) {
            action_state.just_pressed.insert(action, true);
        }
    }
}

/// Special-cased version of [update_action_state] for Gamepads
///
/// This system is intended for single-player games;
/// all gamepads are mapped to a single [ActionState].
/// You will want to modify this system if you want to handle multiple players correctly
pub fn update_action_state_gamepads<InputAction: InputActionEnum>(
    gamepads: Res<Gamepads>,
    gamepad_map: Res<InputMap<InputAction, GamepadButton, GamepadButtonType>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut action_state: ResMut<ActionState<InputAction>>,
) {
    for action in InputAction::iter() {
        for &gamepad in gamepads.iter() {
            if gamepad_map.pressed(action, &*gamepad_input, gamepad) {
                action_state.pressed.insert(action, true);
            }

            if gamepad_map.just_pressed(action, &*gamepad_input, gamepad) {
                action_state.just_pressed.insert(action, true);
            }
        }
    }
}
