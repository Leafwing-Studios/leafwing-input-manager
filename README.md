# About

A simple but robust input-action manager for Bevy: intended to be useful both as a plugin and a helpful library.

Inputs from various input sources (keyboard, mouse and gamepad) are collected into a common `ActionState` on your player entity,
which can be conveniently used in your game logic.

The mapping between inputs and actions is many-to-many, and easily configured and extended with the `InputMap` components on your player entity.
A single action can be triggered by multiple inputs (or set directly by UI elements or gameplay logic),
and a single input can result in multiple actions being triggered, which can be handled contextually.

This library is designed to support both single-player and local multiplayer games!
Simply add the `InputManagerBundle` to each controllable entity, and customize the `InputMap` and `AssociatedGamepad` values appropriately.

## Instructions

### Getting started

1. Add `leafwing-input-manager` to your `Cargo.toml`.
2. Create an enum of the logical actions you want to represent, and implement the `Actionlike` for it.
3. Add the `InputManagerPlugin` to your `App`.
4. Add the `InputManagerBundle` to your player entity (or entities!).
5. Configure a mapping between your inputs and your actions by modifying the `InputMap` components on your player entity.
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
You can use `#` to hide a setup line from the doc tests.
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

Reference documentation is handled with standard Rust doc strings.
Use `cargo doc --open` to build and then open the docs.

Design docs (or other book-format documentation) is handled with [mdBook](https://rust-lang.github.io/mdBook/index.html).
Install it with `cargo install mdbook`, then use `mdbook serve --open` to launch the docs.
