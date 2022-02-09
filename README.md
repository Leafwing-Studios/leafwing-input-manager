# About

A straightforward but robust input-action manager for Bevy.

Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
which can be conveniently used in your game logic.

The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on each player entity.
A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
and a single input can result in multiple actions being triggered, which can be handled contextually.

## Features

- Full keyboard, mouse and joystick support for button-like inputs.
- Effortlessly wire UI buttons to game state with one simple component!
  - When clicked, your button will send a virtual button press to the corresponding entity.
- Store all your input mappings in a single `InputMap` component
  - No more bespoke `Keybindings<KeyCode>`, `Keybindings<Gamepad>` headaches
- Look up your current input state in a single `ActionState` component
  - Easily check player statistics while reading input
  - That pesky maximum of 16 system parameters got you down? Say goodbye to that input handling mega-system
- Ergonomic insertion API that seamlessly blends multiple input types for you
  - `input_map.insert(Action::Jump, KeyCode::Space)` XOR `input_map.insert(Action::Jump, GamepadButtonType::South)`? Have both!
- Full support for arbitrary button combinations: chord your heart out.
  - `input_map.insert_chord(Action::Console, [KeyCode::LCtrl, KeyCode::Shift, KeyCode::C])`
- Sophisticated input disambiguation with the `ClashStrategy` enum: stop triggering individual buttons when you meant to press a chord!
- Create an arbitrary number of strongly typed disjoint action sets: decouple your camera and player state.
- Local multiplayer support: freely bind keys to distinct entities, rather than worrying about singular global state
- Networked multiplayer support: serializable structs, and a space-conscious `ActionDiff` representation to send on the wire
- Powerful and easy-to-use input mocking API for integration testing your Bevy applications
  - `app.send_input(KeyCode::B)` or `world.send_input(UserInput::chord([KeyCode::B, KeyCode::E, KeyCode::V, KeyCode::Y])`
- Control which state this plugin is active in: stop wandering around while in a menu!
- Leafwing Studio's trademark `#![forbid(missing_docs)]`

## Limitations

- The `Button` enum only includes `KeyCode`, `MouseButton` and `GamepadButtonType`.
  - This is due to object-safety limitations on the types stored in `bevy::input::Input`
  - Please file an issue if you would like something more exotic!
- No built-in support for non-button input types (e.g. gestures or analog sticks).
  - All methods on `ActionState` are `pub`: it's designed to be hooked into and extended.
- Gamepads must be associated with each player by the app using this plugin: read from the `Gamepads` resource and use `InputMap::set_gamepad`.

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

### Running your game

Use `cargo run`.
This repo is set up to always build with full optimizations, so there's no need for a `--release` flag in most cases.
Dynamic linking is enabled to ensure build times stay snappy.

To run an example, use `cargo run --example_name`, where `example_name` is the file name of the example without the `.rs` extension.
