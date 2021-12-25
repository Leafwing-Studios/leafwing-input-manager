use bevy::input::InputSystem;
use bevy::prelude::*;

use crate::action_state::ActionState;
use crate::input_map::InputMap;
use core::fmt::Display;
use core::hash::Hash;
use core::marker::PhantomData;
use strum::IntoEnumIterator;

pub mod action_state;
pub mod input_map;
pub mod systems;

pub mod prelude {
    pub use crate::action_state::ActionState;
    pub use crate::input_map::InputMap;

    pub use crate::{InputActionEnum, InputManagerPlugin};
}

/// A [Plugin] that collects [Input] from disparate sources, producing an [ActionState] to consume in game logic
///
/// For each entity with a [PlayerMarker](Self::PlayerMarker) component,
/// several components are inserted:
///  - one [InputMap] component for each of [KeyCode], [GamepadButton] and [MouseButton]
///  - an [ActionState] component, which stores the current input state for that player in an source-agnostic fashion
///
/// ## Systems
/// - [tick_action_state](systems::tick_action_state), which resets the pressed and just_pressed fields of the [ActionState] each frame
///     - labeled [InputMapSystem::Reset]
/// - [update_action_state] and [update_action_state_gamepads], which collects the [Input] from the corresponding input type to update the [ActionState]
///     - labeled [InputMapSystem::Read]
/// - [release_action_state], which releases all actions which are not currently pressed by any system
///     - labeled [InputMapSystem::Release]
pub struct InputManagerPlugin<InputAction: InputActionEnum, PlayerMarker: Component> {
    _unreachable: (PhantomData<InputAction>, PhantomData<PlayerMarker>),
}

// Cannot use derive(Default), as it forces an undesirable bound on our generics
impl<InputAction: InputActionEnum, PlayerMarker: Component> Default
    for InputManagerPlugin<InputAction, PlayerMarker>
{
    fn default() -> Self {
        Self {
            _unreachable: (PhantomData::default(), PhantomData::default()),
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
pub enum InputManagerSystem {
    Reset,
    Read,
    Release,
}

impl<InputAction: InputActionEnum, PlayerMarker: Component> Plugin
    for InputManagerPlugin<InputAction, PlayerMarker>
{
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        app.init_resource::<InputMap<InputAction, KeyCode>>()
            .init_resource::<InputMap<InputAction, MouseButton>>()
            .init_resource::<InputMap<InputAction, GamepadButton, GamepadButtonType>>()
            .init_resource::<ActionState<InputAction>>()
            .add_system(
                tick_action_state::<InputAction>
                    .label(InputManagerSystem::Reset)
                    .before(InputManagerSystem::Read),
            )
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new()
                    .with_system(update_action_state::<InputAction, KeyCode>)
                    .with_system(update_action_state::<InputAction, MouseButton>)
                    .with_system(update_action_state_gamepads::<InputAction>)
                    .label(InputManagerSystem::Read)
                    .after(InputSystem),
            )
            .add_system(
                release_action_state::<InputAction>
                    .label(InputManagerSystem::Release)
                    .after(InputManagerSystem::Read),
            );
    }
}
