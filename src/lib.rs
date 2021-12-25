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

    pub use crate::{Actionlike, InputManagerBundle, InputManagerPlugin};
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
pub struct InputManagerPlugin<A: Actionlike> {
    _phantom: PhantomData<A>,
}

// Deriving default induces an undesired bound on the generic
impl<A: Actionlike> Default for InputManagerPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }
}

/// A type that can be used to represent input-agnostic action representation
///
/// This trait should be implemented on the `A` type that you want to pass into [InputManagerPlugin]
pub trait Actionlike:
    Send + Sync + Copy + Eq + Hash + IntoEnumIterator + Display + 'static
{
}

#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputManagerSystem {
    Reset,
    Read,
    Release,
}

#[derive(Bundle)]
pub struct InputManagerBundle<A: Actionlike> {
    action_state: ActionState<A>,
    input_map: InputMap<A>,
}

// Cannot use derive(Default), as it forces an undesirable bound on our generics
impl<A: Actionlike> Default for InputManagerBundle<A> {
    fn default() -> Self {
        Self {
            action_state: ActionState::default(),
            input_map: InputMap::default(),
        }
    }
}

impl<A: Actionlike> Plugin for InputManagerPlugin<A> {
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        app.add_system(
            tick_action_state::<A>
                .label(InputManagerSystem::Reset)
                .before(InputManagerSystem::Read),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            update_action_state::<A>
                .label(InputManagerSystem::Read)
                .after(InputSystem),
        )
        .add_system(
            release_action_state::<A>
                .label(InputManagerSystem::Release)
                .after(InputManagerSystem::Read),
        );
    }
}
