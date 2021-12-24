/// Input handling uses the following architecture:
/// - input handling and ability determination is all handled in CoreStage::PreUpdate, to make sure entities are spawned in time
/// - inputs are stored in a global `ActionState` resource, which is a big collection of booleans
/// - input handling is done holistically, by examining `ActionState`
use bevy::prelude::*;
use bevy::utils::HashMap;

use core::fmt::Display;
use core::hash::Hash;
use core::marker::PhantomData;
use multimap::MultiMap;
use strum::IntoEnumIterator;

pub struct InputPlugin<InputAction: InputActionEnum> {
    _phantom: PhantomData<InputAction>,
}

pub trait InputActionEnum:
    Send + Sync + Copy + Eq + Hash + IntoEnumIterator + Display + 'static
{
}

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputMapLabel {
    Processing,
}

impl<InputAction: InputActionEnum> Plugin for InputPlugin<InputAction> {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMap<InputAction, KeyCode>>()
            .init_resource::<InputMap<InputAction, GamepadButton, GamepadButtonType>>()
            .init_resource::<ActionState<InputAction>>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_action_state::<InputAction>.label(InputMapLabel::Processing),
            );
    }
}

/// Input mapping resource
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

fn update_action_state<InputAction: InputActionEnum>(
    keyboard_input: Res<Input<KeyCode>>,
    keyboard_map: Res<InputMap<InputAction, KeyCode>>,
    gamepads: Res<Gamepads>,
    gamepad_map: Res<InputMap<InputAction, GamepadButton, GamepadButtonType>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut action_state: ResMut<ActionState<InputAction>>,
) {
    for action in InputAction::iter() {
        let keyboard_pressed = keyboard_map.pressed(action, &*keyboard_input);
        let keyboard_just_pressed = keyboard_map.just_pressed(action, &*keyboard_input);
        let mut gamepad_pressed = false;
        let mut gamepad_just_pressed = false;

        for &gamepad in gamepads.iter() {
            if gamepad_map.pressed(action, &*gamepad_input, gamepad) {
                gamepad_pressed = true;
            }

            if gamepad_map.just_pressed(action, &*gamepad_input, gamepad) {
                gamepad_just_pressed = true;
            }
        }

        action_state
            .pressed
            .insert(action, keyboard_pressed | gamepad_pressed);
        action_state
            .just_pressed
            .insert(action, keyboard_just_pressed | gamepad_just_pressed);
    }
}
