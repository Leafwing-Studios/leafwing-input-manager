# About

[![Crates.io](https://img.shields.io/crates/v/leafwing-input-manager.svg)](https://crates.io/crates/leafwing-input-manager)

A straightforward but robust input-action manager for Bevy.

Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
which can be conveniently used in your game logic.

The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on each player entity.
A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
and a single input can result in multiple actions being triggered, which can be handled contextually.

## Features

- Full keyboard, mouse and joystick support for button-like and axis inputs
- Dual axis support for analog inputs from gamepads and joysticks
- Bind arbitrary button inputs into virtual DPads
- Effortlessly wire UI buttons to game state with one simple component!
  - When clicked, your button will press the appropriate action on the corresponding entity
- Store all your input mappings in a single `InputMap` component
  - No more bespoke `Keybindings<KeyCode>`, `Keybindings<Gamepad>` headaches
- Look up your current input state in a single `ActionState` component
  - That pesky maximum of 16 system parameters got you down? Say goodbye to that input handling mega-system
- Ergonomic insertion API that seamlessly blends multiple input types for you
  - Can't decide between `input_map.insert(Action::Jump, KeyCode::Space)` and `input_map.insert(Action::Jump, GamepadButtonType::South)`? Have both!
- Full support for arbitrary button combinations: chord your heart out.
  - `input_map.insert_chord(Action::Console, [KeyCode::LControl, KeyCode::Shift, KeyCode::C])`
- Sophisticated input disambiguation with the `ClashStrategy` enum: stop triggering individual buttons when you meant to press a chord!
- Create an arbitrary number of strongly typed disjoint action sets by adding multiple copies of this plugin: decouple your camera and player state
- Local multiplayer support: freely bind keys to distinct entities, rather than worrying about singular global state
- Networked multiplayer support: serializable structs, and a space-conscious `ActionDiff` representation to send on the wire
- Powerful and easy-to-use input mocking API for integration testing your Bevy applications
  - `app.send_input(KeyCode::B)` or `world.send_input(UserInput::chord([KeyCode::B, KeyCode::E, KeyCode::V, KeyCode::Y])`
- Control which state this plugin is active in: stop wandering around while in a menu!
- Sophisticated cooldown management for `Actionlike` actions
- Leafwing Studio's trademark `#![forbid(missing_docs)]`

## Limitations

- Gamepads must be manually assigned to each input map: read from the `Gamepads` resource and use `InputMap::set_gamepad`.

## Instructions

**Development occurs on the `dev` branch, which is merged into `main` on each release.**
This ensures the examples are in-sync with the latest release.

### Getting started

1. Add `leafwing-input-manager` to your `Cargo.toml`.
2. Create an enum of the logical actions you want to represent, and derive the `Actionlike` trait for it.
3. Add the `InputManagerPlugin` to your `App`.
4. Add the `InputManagerBundle` to your player entity (or entities!).
5. Configure a mapping between your inputs and your actions by modifying the `InputMap` component on your player entity.
6. Read the `ActionState` component on your player entity to check the collected input state!

```rust, ignore
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // This plugin maps inputs to an input-type agnostic action-state
        // We need to provide it with an enum which stores the possible actions a player could take
        .add_plugin(InputManagerPlugin::<Action>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_startup_system(spawn_player)
        // Read the ActionState in your systems using queries!
        .add_system(jump)
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Run,
    Jump,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert_bundle(InputManagerBundle::<Action> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new([(KeyCode::Space, Action::Jump)]),
        });
}

// Query for the `ActionState` component in your game logic systems!
fn jump(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action has a button-like state of its own that you can check
    if action_state.just_pressed(Action::Jump) {
        println!("I'm jumping!");
    }
}
```

This snippet is the `minimal.rs` example from the [`examples`](./examples) folder: check there for more in-depth learning materials!
