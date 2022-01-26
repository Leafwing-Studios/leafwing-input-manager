# About

A simple but robust input-action manager for Bevy: intended to be useful both as a plugin and a helpful library.

Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
which can be conveniently used in your game logic.

The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on your player entity.
A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
and a single input can result in multiple actions being triggered, which can be handled contextually.

This library seamlessly supports both single-player and local multiplayer games!
Simply add the `InputManagerBundle` to each controllable entity, and customize the `InputMap` appropriately.

Of particular note: this plugin allows users to configure which state (i.e. `GameState::Playing`) it runs in.
No more characters wandering around while your menu is open!

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
  - `input_map.insert(Action::Jump, KeyCode::Space)` XOR `input_map.insert(Action::Jump, C)`? Why not both?
- Full support for arbitrary button combinations: chord your heart out.
  - `input_map.insert_chord(Action::Console, [KeyCode::LCtrl, KeyCode::Shift, KeyCode::C])`
- Sophisticated input disambiguation with the `ClashStrategy` enum: stop triggering individual buttons when you meant to press a chord!
- Create an arbitrary number of strongly typed disjoint action sets: decouple your camera and player state.
- Local multiplayer support: freely bind keys to distinct entities, rather than worrying about singular global state
- Powerful and easy-to-use input mocking API for integration testing your Bevy applications
  - `app.send_input(KeyCode::B)` or `world.send_input(UserInput::chord([KeyCode::B, KeyCode::E, KeyCode::V, KeyCode::Y])`
- Leafwing Studio's trademark `#![forbid(missing_docs)]`

## Limitations

- The `Button` enum only includes `KeyCode`, `MouseButton` and `GamepadButtonType`.
  - This is due to object-safety limitations on the types stored in `bevy::input::Input`
  - Please file an issue if you would like something more exotic!
- No built-in support for non-button input types (e.g. gestures or analog sticks).
  - All methods on `ActionState` are `pub`: it's designed to be hooked into and extended.
- Gamepads must be associated with each player by the app using this plugin: read from the `Gamepads` resource and use `InputMap::set_gamepad`.

## Instructions

### Getting started

1. Add `leafwing-input-manager` to your `Cargo.toml`.
2. Create an enum of the logical actions you want to represent, and implement the `Actionlike` trait for it.
3. Add the `InputManagerPlugin` to your `App`.
4. Add the `InputManagerBundle` to your player entity (or entities!).
5. Configure a mapping between your inputs and your actions by modifying the `InputMap` component on your player entity.
6. Read the `ActionState` component on your player entity to check the collected input state!

### Running your game

Use `cargo run`.
This repo is set up to always build with full optimizations, so there's no need for a `--release` flag in most cases.
Dynamic linking is enabled to ensure build times stay snappy.

To run an example, use `cargo run --example_name`, where `example_name` is the file name of the example without the `.rs` extension.

## Contributing

This repository is open to community contributions!
There are a few options if you'd like to help:

1. File issues for bugs you find or new features you'd like.
2. Read over and discuss issues, then make a PR that fixes them. Use "Fixes #X" in your PR description to automatically close the issue when the PR is merged.
3. Review existing PRs, and leave thoughtful feedback. If you think a PR is ready to merge, hit "Approve" in your review!

Any contributions made are provided under the license(s) listed in this repo at the time of their contribution, and do not require separate attribution.

### Testing

1. Use doc tests aggressively to show how APIs should be used.
You can use `#` to hide setup code from the doc tests, but use this sparingly.
2. Unit test belong near the code they are testing. Use `#[cfg(test)]` on the test module to ignore it during builds, and `#[test]` on the test functions to ensure they are run.
3. Integration tests should be stored in the top level `tests` folder, importing functions from `lib.rs`.

Use `cargo test` to run all tests.

### CI

The CI will:

1. Ensure the code is formatted with `cargo fmt`.
2. Ensure that the code compiles.
3. Ensure that (almost) all `clippy` lints pass.
4. Ensure all tests pass on Windows, MacOS and Ubuntu.

Check this locally with:

1. `cargo run -p ci`
2. `cargo test --workspace`

To manually rerun CI:

1. Navigate to the `Actions` tab.
2. Use the dropdown menu in the CI run of interest and select "View workflow file".
3. In the top-right corner, select "Rerun workflow".

### Documentation

Documentation is handled with standard Rust doc strings.
Use `cargo doc --open` to build and then open the docs locally.
