/// Input handling uses the following architecture:
/// - input handling and ability determination is all handled in CoreStage::PreUpdate, to make sure entities are spawned in time
/// - inputs are stored in a global `ActionState` resource, which is a big collection of booleans
/// - input handling is done holistically, by examining `ActionState`
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

use core::hash::Hash;
use derive_more::{Deref, DerefMut, Display};
use multimap::MultiMap;
use std::marker::PhantomData;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::movement::Direction;

pub struct InputPlugin;

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputLabel {
    Processing,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamepadLobby>()
            .init_resource::<InputMap<KeyCode>>()
            .init_resource::<InputMap<GamepadButton, GamepadButtonType>>()
            .init_resource::<ActionState>()
            .add_system_to_stage(
                CoreStage::PreUpdate,
                gamepad_conection_manager.before(InputLabel::Processing),
            )
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_action_state.label(InputLabel::Processing),
            );
    }
}

#[derive(Default, Deref, DerefMut)]
pub struct GamepadLobby {
    gamepads: HashSet<Gamepad>,
}

fn gamepad_conection_manager(
    mut lobby: ResMut<GamepadLobby>,
    mut gamepad_event: EventReader<GamepadEvent>,
) {
    for event in gamepad_event.iter() {
        match &event {
            GamepadEvent(gamepad, GamepadEventType::Connected) => {
                lobby.gamepads.insert(*gamepad);
                info!("{:?} Connected", gamepad);
            }
            GamepadEvent(gamepad, GamepadEventType::Disconnected) => {
                lobby.gamepads.remove(gamepad);
                info!("{:?} Disconnected", gamepad);
            }
            _ => (),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Display, EnumIter)]
pub enum InputAction {
    // Movement
    Up,
    Down,
    Left,
    Right,
    // Abilities
    Ability1,
    Ability2,
    Ability3,
    Ability4,
    Ultimate,
    // Utilities
    AimLock,
    Emote,
    Interact,
}

impl InputAction {
    pub const MOVEMENT: [(InputAction, Direction); 4] = [
        (InputAction::Up, Direction::UP),
        (InputAction::Down, Direction::DOWN),
        (InputAction::Left, Direction::LEFT),
        (InputAction::Right, Direction::RIGHT),
    ];

    pub const ABILITIES: [InputAction; 5] = [
        InputAction::Ability1,
        InputAction::Ability2,
        InputAction::Ability3,
        InputAction::Ability4,
        InputAction::Ultimate,
    ];
}

/// Input mapping resource
struct InputMap<InputType, InputVariant = InputType> {
    mmap: MultiMap<InputAction, InputVariant>,
    _phantom: PhantomData<InputType>,
}

impl Default for InputMap<KeyCode> {
    fn default() -> Self {
        let mut mmap = MultiMap::default();

        // Movement
        mmap.insert(InputAction::Up, KeyCode::Up);
        mmap.insert(InputAction::Down, KeyCode::Down);
        mmap.insert(InputAction::Left, KeyCode::Left);
        mmap.insert(InputAction::Right, KeyCode::Right);

        // Abilities
        mmap.insert(InputAction::Ability1, KeyCode::Q);
        mmap.insert(InputAction::Ability2, KeyCode::W);
        mmap.insert(InputAction::Ability3, KeyCode::E);
        mmap.insert(InputAction::Ability4, KeyCode::Space); // movement ability
        mmap.insert(InputAction::Ultimate, KeyCode::R);

        // Utilities
        mmap.insert(InputAction::AimLock, KeyCode::Grave);
        mmap.insert(InputAction::Emote, KeyCode::LShift);
        mmap.insert(InputAction::Interact, KeyCode::F);

        Self {
            mmap,
            _phantom: PhantomData::default(),
        }
    }
}

impl Default for InputMap<GamepadButton, GamepadButtonType> {
    fn default() -> Self {
        let mut mmap = MultiMap::default();

        // Movement
        mmap.insert(InputAction::Up, GamepadButtonType::DPadUp);
        mmap.insert(InputAction::Down, GamepadButtonType::DPadDown);
        mmap.insert(InputAction::Left, GamepadButtonType::DPadLeft);
        mmap.insert(InputAction::Right, GamepadButtonType::DPadRight);

        // Abilities
        mmap.insert(InputAction::Ability1, GamepadButtonType::West);
        mmap.insert(InputAction::Ability2, GamepadButtonType::North);
        mmap.insert(InputAction::Ability3, GamepadButtonType::East);
        mmap.insert(InputAction::Ability4, GamepadButtonType::South); // movement ability
        mmap.insert(InputAction::Ultimate, GamepadButtonType::LeftTrigger2);

        // Utilities
        mmap.insert(InputAction::AimLock, GamepadButtonType::RightTrigger);
        mmap.insert(InputAction::Emote, GamepadButtonType::LeftTrigger);
        mmap.insert(InputAction::Interact, GamepadButtonType::RightTrigger2);

        Self {
            mmap,
            _phantom: PhantomData::default(),
        }
    }
}

impl<T: Copy + Hash + Eq> InputMap<T> {
    pub fn pressed(&self, action: InputAction, input: &Input<T>) -> bool {
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

    pub fn just_pressed(&self, action: InputAction, input: &Input<T>) -> bool {
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
impl InputMap<GamepadButton, GamepadButtonType> {
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
pub struct ActionState {
    pressed: HashMap<InputAction, bool>,
    just_pressed: HashMap<InputAction, bool>,
}

impl ActionState {
    pub fn pressed(&self, action: InputAction) -> bool {
        *self.pressed.get(&action).unwrap()
    }

    pub fn just_pressed(&self, action: InputAction) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }
}

impl Default for ActionState {
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

fn update_action_state(
    keyboard_input: Res<Input<KeyCode>>,
    keyboard_map: Res<InputMap<KeyCode>>,
    gamepad_lobby: Res<GamepadLobby>,
    gamepad_map: Res<InputMap<GamepadButton, GamepadButtonType>>,
    gamepad_input: Res<Input<GamepadButton>>,
    mut action_state: ResMut<ActionState>,
) {
    for action in InputAction::iter() {
        let keyboard_pressed = keyboard_map.pressed(action, &*keyboard_input);
        let keyboard_just_pressed = keyboard_map.just_pressed(action, &*keyboard_input);
        let mut gamepad_pressed = false;
        let mut gamepad_just_pressed = false;

        for &gamepad in gamepad_lobby.iter() {
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
