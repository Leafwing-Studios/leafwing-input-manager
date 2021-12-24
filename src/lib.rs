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
/// - [tick_action_state], which resets the pressed and just_pressed fields of the [ActionState] each frame
///     - labeled [InputMapSystem::Reset]
/// - [update_action_state] and [update_action_state_gamepads], which collects the [Input] from the corresponding input type to update the [ActionState]
///     - labeled [InputMapSystem::Read]
/// - [release_action_state], which releases all actions which are not currently pressed by any system
///     - labeled [InputMapSystem::Release]
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
pub enum InputMapSystem {
    Reset,
    Read,
    Release,
}

impl<InputAction: InputActionEnum> Plugin for InputManagerPlugin<InputAction> {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputMap<InputAction, KeyCode>>()
            .init_resource::<InputMap<InputAction, MouseButton>>()
            .init_resource::<InputMap<InputAction, GamepadButton, GamepadButtonType>>()
            .init_resource::<ActionState<InputAction>>()
            .add_system(
                tick_action_state::<InputAction>
                    .label(InputMapSystem::Reset)
                    .before(InputMapSystem::Read),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(update_action_state::<InputAction, KeyCode>)
                    .with_system(update_action_state::<InputAction, MouseButton>)
                    .with_system(update_action_state_gamepads::<InputAction>)
                    .label(InputMapSystem::Read)
                    .after(InputSystem),
            )
            .add_system(
                release_action_state::<InputAction>
                    .label(InputMapSystem::Release)
                    .after(InputMapSystem::Read),
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
#[derive(Component)]
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
#[derive(Component)]
pub struct ActionState<InputAction: InputActionEnum> {
    pressed: HashMap<InputAction, bool>,
    pressed_this_tick: HashMap<InputAction, bool>,
    just_pressed: HashMap<InputAction, bool>,
    just_released: HashMap<InputAction, bool>,
}

impl<InputAction: InputActionEnum> ActionState<InputAction> {
    /// Clears all just_pressed and just_released state
    ///
    /// Also clears the internal field `pressed_this_tick`, so inputs can be correctly released
    pub fn tick(&mut self) {
        self.just_pressed = Self::default_map();
        self.just_released = Self::default_map();
        self.pressed_this_tick = Self::default_map();
    }

    /// Press the `action`
    pub fn press(&mut self, action: InputAction) {
        self.pressed.insert(action, true);
        self.pressed_this_tick.insert(action, true);
        if !self.pressed(action) {
            self.just_pressed.insert(action, true);
        }
    }

    /// Release the `action`
    pub fn release(&mut self, action: InputAction) {
        self.pressed.insert(action, false);

        if self.pressed(action) {
            self.just_released.insert(action, true);
        }
    }

    /// Releases all actions
    pub fn release_all(&mut self) {
        for action in InputAction::iter() {
            self.release(action);
        }
    }

    /// Is an action currently pressed?
    pub fn pressed(&self, action: InputAction) -> bool {
        *self.pressed.get(&action).unwrap()
    }

    /// Was this action pressed since the last time [tick](ActionState::tick) was called?
    pub fn just_pressed(&self, action: InputAction) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }

    /// Is an action currently released?
    ///
    /// This is always the logical negation of [pressed](ActionState::pressed)
    pub fn released(&self, action: InputAction) -> bool {
        !*self.pressed.get(&action).unwrap()
    }

    /// Was this action pressed since the last time [tick](ActionState::tick) was called?
    pub fn just_released(&self, action: InputAction) -> bool {
        *self.just_pressed.get(&action).unwrap()
    }

    /// Release all actions that were not pressed this tick
    pub fn release_unpressed(&mut self) {
        for action in InputAction::iter() {
            if !*self.pressed_this_tick.get(&action).unwrap() {
                self.release(action);
            }
        }
    }

    /// Creates a Hashmap with all of the possible InputAction variants as keys, and false as the values
    fn default_map() -> HashMap<InputAction, bool> {
        // PERF: optimize construction through pre-allocation or constification
        let mut map = HashMap::default();

        for action in InputAction::iter() {
            map.insert(action, false);
        }
        map
    }
}

impl<InputAction: InputActionEnum> Default for ActionState<InputAction> {
    fn default() -> Self {
        Self {
            pressed: Self::default_map(),
            pressed_this_tick: Self::default_map(),
            just_pressed: Self::default_map(),
            just_released: Self::default_map(),
        }
    }
}

/// Clears the just-pressed and just-released values of the [ActionState]
///
/// Also resets the internal `pressed_this_tick` field, used to track whether or not to release an action
pub fn tick_action_state<InputAction: InputActionEnum>(
    mut action_state: ResMut<ActionState<InputAction>>,
) {
    action_state.tick();
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
            action_state.press(action);
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
        }
    }
}

/// Releases all [ActionState] actions that were not pressed since the last time [tick_action_state] ran
pub fn release_action_state<InputAction: InputActionEnum>(
    mut action_state: ResMut<ActionState<InputAction>>,
) {
    action_state.release_unpressed();
}
