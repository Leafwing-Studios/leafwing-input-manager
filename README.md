# About

[![Crates.io](https://img.shields.io/crates/v/leafwing-input-manager.svg)](https://crates.io/crates/leafwing-input-manager)
[![docs.rs](https://img.shields.io/docsrs/leafwing-input-manager/latest)](https://docs.rs/leafwing-input-manager/latest)

A straightforward but robust input-action manager for Bevy.

Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
which can be conveniently used in your game logic.

The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on each player entity.
A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
and a single input can result in multiple actions being triggered, which can be handled contextually.

## Supported Bevy Versions

| Bevy | leafwing-input-manager |
| ---- | ---------------------- |
| 0.14 | 0.14                   |
| 0.13 | 0.13                   |
| 0.12 | 0.11..0.12             |
| 0.11 | 0.10                   |
| 0.10 | 0.9                    |
| 0.9  | 0.7..0.8               |

## Features

- Full keyboard, mouse and joystick support for button-like and axis inputs
- Dual axis support for analog inputs from gamepads and joysticks
- Bind arbitrary button inputs into virtual D-Pads
- Effortlessly wire UI buttons to game state with one simple component!
  - When clicked, your button will press the appropriate action on the corresponding entity
- Store all your input mappings in a single `InputMap` component
  - No more bespoke `Keybindings<KeyCode>`, `Keybindings<Gamepad>` headaches
- Look up your current input state in a single `ActionState` component
  - That pesky maximum of 16 system parameters got you down? Say goodbye to that input handling mega-system
- Ergonomic insertion API that seamlessly blends multiple input types for you
  - Can't decide between `input_map.insert(Action::Jump, KeyCode::Space)` and `input_map.insert(Action::Jump, GamepadButtonType::South)`? Have both!
- Full support for arbitrary button combinations: chord your heart out.
  - `input_map.insert(Action::Console, InputChord::new([KeyCode::ControlLeft, KeyCode::Shift, KeyCode::KeyC]))`
- Sophisticated input disambiguation with the `ClashStrategy` enum: stop triggering individual buttons when you meant to press a chord!
- Create an arbitrary number of strongly typed disjoint action sets by adding multiple copies of this plugin: decouple your camera and player state
- Local multiplayer support: freely bind keys to distinct entities, rather than worrying about singular global state
- Networked multiplayer support: serializable structs, and a space-conscious `ActionDiff` representation to send on the wire
- Powerful and easy-to-use input mocking API for integration testing your Bevy applications
  - `app.press_input(KeyCode::KeyB)` or `world.press_input(UserInput::chord([KeyCode::KeyB, KeyCode::KeyE, KeyCode::KeyV, KeyCode::KeyY])`
- Control which state this plugin is active in: stop wandering around while in a menu!
- Leafwing Studio's trademark `#![forbid(missing_docs)]`

## Limitations

- Gamepads must be manually assigned to each input map: read from the `Gamepads` resource and use `InputMap::set_gamepad`.

## Getting started

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
        .add_plugins(InputManagerPlugin::<Action>::default())
        // The InputMap and ActionState components will be added to any entity with the Player component
        .add_systems(Startup, spawn_player)
        // Read the ActionState in your systems using queries!
        .add_systems(Update, jump)
        .run();
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
enum Action {
    Run,
    Jump,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Describes how to convert from player inputs into those actions
    let input_map = InputMap::new([(Action::Jump, KeyCode::Space)]);
    commands
        .spawn(InputManagerBundle::with_map(input_map))
        .insert(Player);
}

// Query for the `ActionState` component in your game logic systems!
fn jump(query: Query<&ActionState<Action>, With<Player>>) {
    let action_state = query.single();
    // Each action has a button-like state of its own that you can check
    if action_state.just_pressed(&Action::Jump) {
        println!("I'm jumping!");
    }
}
```

This snippet is the `minimal.rs` example from the [`examples`](https://github.com/Leafwing-Studios/leafwing-input-manager/tree/main/examples) folder: check there for more in-depth learning materials!

## Crate Feature Flags

Please refer to the `[features]` section in the [`Cargo.toml`](https://github.com/Leafwing-Studios/leafwing-input-manager/tree/main/Cargo.toml) for information about the available crate features.
