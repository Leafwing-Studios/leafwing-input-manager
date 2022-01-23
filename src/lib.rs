// I am deeply sorry for the false advertising;
// we are currently blocked on the 0.24 release of `strum`, which fixes the issue with the `EnumIter` macro :(
// However! CI will fail if any warnings are detected, so full documentation is in fact enforced.
#![deny(missing_docs)]
#![warn(clippy::doc_markdown)]

//! # About
//!
//! A simple but robust input-action manager for Bevy: intended to be useful both as a plugin and a helpful library.
//!
//! Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
//! which can be conveniently used in your game logic.
//!
//! The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on your player entity.
//! A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
//! and a single input can result in multiple actions being triggered, which can be handled contextually.
//!
//! This library seamlessly supports both single-player and local multiplayer games!
//! Simply add the `InputManagerBundle` to each controllable entity, and customize the `InputMap` appropriately.
//!
//! ## Features
//!
//! - Full keyboard, mouse and joystick support for button-like inputs.
//! - Effortlessly wire UI buttons to game state with one simple component!
//!   - When clicked, your button will send a virtual button press to the corresponding entity.
//! - Store all your input mappings in a single `InputMap` component
//!   - No more bespoke `Keybindings<KeyCode>`, `Keybindings<Gamepad>` headaches
//! - Look up your current input state in a single `ActionState` component
//!   - Easily check player statistics while reading input
//!   - That pesky maximum of 16 system parameters got you down? Say goodbye to that input handling mega-system
//! - Ergonomic insertion API that seamlessly blends multiple input types for you
//!   - `input_map.insert(Action::Jump, KeyCode::Space)` XOR `input_map.insert(Action::Jump, C)`? Why not both?
//! - Full support for arbitrary button combinations: chord your heart out.
//!   - `input_map.insert_chord(Action::Console, [KeyCode::LCtrl, KeyCode::Shift, KeyCode::C])`
//! - Create an arbitrary number of strongly typed disjoint action sets: decouple your camera and player state.
//! - Local multiplayer support: freely bind keys to distinct entities, rather than worrying about singular global state
//! - Leafwing Studio's trademark `#![forbid(missing_docs)]`
//!
//! ## Limitations
//!
//! - The `Button` enum only includes `KeyCode`, `MouseButton` and `GamepadButtonType`.
//!   - This is due to object-safety limitations on the types stored in `bevy::input::Input`
//!   - Please file an issue if you would like something more exotic!
//! - No built-in support for non-button input types (e.g. gestures or analog sticks).
//!   - All methods on `ActionState` are `pub`: it's designed to be hooked into and extended.
//! - Gamepads must be associated with each player by the end game: read from the `Gamepads` resource and use `InputMap::set_gamepad`.
use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::system::Resource;
use bevy::input::InputSystem;
use bevy::prelude::*;

use crate::action_state::ActionState;
use crate::input_map::InputMap;
use core::any::TypeId;
use core::hash::Hash;
use core::marker::PhantomData;

pub mod action_state;
mod display_impl;
pub mod input_map;
pub mod systems;
pub mod user_input;

// Importing the derive macro
pub use leafwing_input_manager_macros::Actionlike;

// Re-exporting the relevant strum macros and traits
pub use strum::IntoEnumIterator;
pub use strum_macros::EnumIter;

/// Everything you need to get started
pub mod prelude {
    pub use crate::action_state::{ActionState, ActionStateDriver};
    pub use crate::input_map::InputMap;
    pub use crate::user_input::UserInput;
    pub use crate::{EnumIter, IntoEnumIterator};

    pub use crate::{Actionlike, InputManagerBundle, InputManagerPlugin};
}

/// A [`Plugin`] that collects [`Input`] from disparate sources, producing an [`ActionState`] to consume in game logic
///
/// This plugin needs to be passed in an [`Actionlike`] enum type that you've created for your game,
/// which acts as a "virtual button" that can be comfortably consumed
///
/// Each [`InputManagerBundle`] contains:
///  - an [`InputMap`] component, which stores an entity-specific mapping between the assorted input streams and an internal repesentation of "actions"
///  - an [`ActionState`] component, which stores the current input state for that entity in an source-agnostic fashion
///
/// ## Systems
/// - [`tick_action_state`](systems::tick_action_state), which resets the `pressed` and `just_pressed` fields of the [`ActionState`] each frame
///     - labeled [`InputManagerSystem::Reset`]
/// - [`update_action_state`](systems::update_action_state) which collects [`Input`] resources to update the [`ActionState`]
///     - labeled [`InputManagerSystem::Update`]
/// - [`update_action_state_from_interaction`](systems::update_action_state_from_interaction), for triggering actions from buttons
///    - powers the [`ActionStateDriver`](crate::action_state::ActionStateDriver) component baseod on an [`Interaction`] component
///    - labeled [`InputManagerSystem::Update`]
pub struct InputManagerPlugin<A: Actionlike, UserState: Resource + PartialEq + Clone = ()> {
    _phantom: PhantomData<(A, UserState)>,
    state_variant: UserState,
}

// Deriving default induces an undesired bound on the generic
impl<A: Actionlike> Default for InputManagerPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData::default(),
            state_variant: (),
        }
    }
}

impl<A: Actionlike, UserState: Resource + PartialEq + Clone> InputManagerPlugin<A, UserState> {
    /// Creates a version of this plugin that will only run in the specified `state_variant`
    ///
    /// # Example
    /// ```rust
    /// use bevy::prelude::*;
    /// use leafwing_input_manager::*;
    /// /// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
    /// enum PlayerAction {
    ///    // Movement
    ///    Up,
    ///    Down,
    ///    Left,
    ///    Right,
    /// }
    ///
    /// #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    /// enum GameState {
    ///     Playing,
    ///     Paused,
    ///     Menu,
    /// }
    ///
    /// App::new().add_plugin(InputManagerPlugin::<PlayerAction, GameState>::run_iff_state(GameState::Playing));
    /// ```
    #[must_use]
    pub fn run_iff_state(state_variant: UserState) -> Self {
        Self {
            _phantom: PhantomData::default(),
            state_variant,
        }
    }
}

impl<A: Actionlike, UserState: Resource + PartialEq + Clone> Plugin
    for InputManagerPlugin<A, UserState>
{
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        if TypeId::of::<UserState>() == TypeId::of::<()>() {
            {
                let input_manager_systems = SystemSet::new()
                    .with_system(
                        tick_action_state::<A>
                            .label(InputManagerSystem::Reset)
                            .before(InputManagerSystem::Update),
                    )
                    .with_system(
                        update_action_state::<A>
                            .label(InputManagerSystem::Update)
                            .after(InputSystem),
                    )
                    .with_system(
                        update_action_state_from_interaction::<A>
                            .label(InputManagerSystem::Update)
                            .after(InputSystem),
                    );

                app.add_system_set_to_stage(CoreStage::PreUpdate, input_manager_systems);
            }
        // If a state has been provided
        // Only run this plugin's systems in the state variant provided
        // Note that this does not perform the standard looping behavior
        // as otherwise we would be limited to the stage that state was added in T_T
        } else {
            // Please forgive me, this whole state-handling API is impossibly janky.
            // https://github.com/bevyengine/rfcs/pull/45 will make special-casing state support unnecessary
            // We can't use a SystemSet, as the run criteria must be reused,
            // and the moved `state_variant` value cannot be shared across the systems

            // Clone it out, so then we're not capturing any part of `Self`
            let state_variant = self.state_variant.clone();

            app.add_system(
                tick_action_state::<A>
                    .label(InputManagerSystem::Reset)
                    .before(InputManagerSystem::Update)
                    .with_run_criteria(move |res: Res<UserState>| {
                        if *res == state_variant {
                            ShouldRun::Yes
                        } else {
                            ShouldRun::No
                        }
                    }),
            );

            // Clone it again, to prevent the use of the moved value!
            let state_variant = self.state_variant.clone();

            app.add_system(
                update_action_state::<A>
                    .label(InputManagerSystem::Update)
                    .after(InputSystem)
                    .with_run_criteria(move |res: Res<UserState>| {
                        if *res == state_variant {
                            ShouldRun::Yes
                        } else {
                            ShouldRun::No
                        }
                    }),
            );

            // One last time, you know the drill T_T
            let state_variant = self.state_variant.clone();

            app.add_system(
                update_action_state_from_interaction::<A>
                    .label(InputManagerSystem::Update)
                    .after(InputSystem)
                    .with_run_criteria(move |res: Res<UserState>| {
                        if *res == state_variant {
                            ShouldRun::Yes
                        } else {
                            ShouldRun::No
                        }
                    }),
            );
        }
    }
}

/// A type that can be used to represent input-agnostic action representation
///
/// Actions serve as "virtual buttons", cleanly abstracting over messy, customizable inputs
/// in a way that can be easily consumed by your game logic.
///
/// This trait should be implemented on the `A` type that you want to pass into [`InputManagerPlugin`]
///
/// # Example
/// ```rust
/// use leafwing_input_manager::{Actionlike, EnumIter};
///
/// #[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
/// enum PlayerAction {
///    // Movement
///    Up,
///    Down,
///    Left,
///    Right,
///    // Abilities
///    Ability1,
///    Ability2,
///    Ability3,
///    Ability4,
///    Ultimate,
/// }
/// ```
pub trait Actionlike: Send + Sync + Copy + Eq + Hash + IntoEnumIterator + 'static {}

/// [`SystemLabel`]s for the [`crate::systems`] used by this crate
///
/// `Reset` must occur before `Update`
#[derive(SystemLabel, Clone, Hash, Debug, PartialEq, Eq)]
pub enum InputManagerSystem {
    /// Cleans up the state of the input manager, clearing `just_pressed` and just_released`
    Reset,
    /// Gathers input data to update the [ActionState]
    Update,
}

/// This [`Bundle`] allows entities to collect and interpret inputs from across input sources
///
/// Use with [`InputManagerPlugin`], providing the same enum type to both.
#[derive(Bundle)]
pub struct InputManagerBundle<A: Actionlike> {
    /// An [ActionState] component
    pub action_state: ActionState<A>,
    /// An [InputMap] component
    pub input_map: InputMap<A>,
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
