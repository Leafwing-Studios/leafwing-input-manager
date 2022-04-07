# Release Notes

## Version 0.3 (Unreleased)

### Enhancements

- users can now store data in the enum variants of their `Actionlike` enums using `ActionState::action_value` and `set_action_value`
  - this allows you to embed rich information about *how* the action should be done in an input-aware centralized fashion
  - internally, matching is done based on the discriminant of the enum, not their value
- geometric primitives (`Direction` and `Rotation`) for working with rotation in 2 dimensions

### Usability

- the crate name now uses underscores (`leafwing_input_manager`) rather than hyphens (`leafwing-input-manager`) to play nicer with `cargo`
- reverted change from by-reference to by-value APIs for `Actionlike` types.
  - this is more ergonomic (derive `Copy` when you can!), and somewhat faster in the overwhelming majority of uses
- relaxed `Hash` and `Eq` bounds on `Actionlike`
- `ActionState::state` and `set_state` methods renamed to `button_state` and `set_button_state` for clarity
- removed `UserInput::Null`.
- `InputManagerPlugin::run_in_state` was replaced with `InputDisabled<A: Actionlike>` resource. Insert this resource when you want to suppress input collection, and remove it when you're done.
- refactored `InputMap`.
- renamed `InputManagerSystem::Reset` into `InputManagerSystem::Tick`.
  - removed methods that works with specific input mode.
  - removed `n_registered`, use `get(action).len()` instead.
  - added `insert_at` / `remove_at` to insert / remove input at specific index.
  - added `remove` remove input for specific mapping.
  - use `usize` for sizes as in other Rust containers.

### Bug fixes

- the `PartialOrd` implementation of `Timing` (and thus `VirtualButtonState`) now compares on the basis of the current duration that the button has been held / released for

## Version 0.2

### Enhancements

- configure how "clashing" inputs should be handled with the `ClashStrategy` field of your `InputMap`
  - very useful for working with modifier keys
  - if two actions are triggered
- ergonomic input mocking API at both the `App` and `World` level using the `MockInputs` trait
- send `ActionState` across the network in a space-efficient fashion using the `ActionDiff` struct
  - check out (or directly use) the `process_action_diff` and `generate_action_diff` systems to convert these to and from `ActionStates`
  - add `InputManagerPlugin::server()` to your server `App` for a stripped down version of the input management functionality

### Usability

- `InputMap::new()` and `InputMap::insert_multiple` now accept an iterator of `(action, input)` tuples for more natural construction
- better decoupled `InputMap` and `ActionState`, providing an `InputMap::which_pressed` API and allowing `ActionState::update` to operate based on any `HashSet<A: Actionlike>` of pressed virtual buttons that you pass in
- `InputMap` now uses a collected `InputStreams` struct in all of its methods, and input methods are now optional
- `InputManagerPlugin` now works even if some input stream resources are missing
- added the `input_pressed` method to `InputMap`, to check if a single input is pressed
- renamed `InputMap::assign_gamepad` to `InputMap::set_gamepad` for consistency and clarity (it does not uniquely assign a gamepad)
- removed `strum` dependency by reimplementing the funcitonality, allowing users to define actions with only the `Actionlike` trait
- added the `get_at` and `index` methods on the `Actionlike` trait, allowing you to fetch a specific action by its position in the defining enum and vice versa
- `Copy` bound on `Actionlike` trait relaxed to `Clone`, allowing you to store non-copy data in your enum variants
- `Clone`, `PartialEq` and `Debug` trait impls for `ActionState`
- `get_pressed`, `get_just_pressed`, `get_released` and `get_just_released` methods on `ActionState`, for conveniently checking many action states at once

### Bug fixes

- the `ActionState` component is no longer marked as `Changed` every frame
- `InputManagerPlugin::run_in_state` now actually works!
- virtually all methods now take actions and inputs by reference, rather than by ownership, eliminating unneccesary copies

## Version 0.1.2

### Usability

- added `set_state` method, allowing users to transfer `VirtualButtonState` between `ActionState` without losing `Timing` information

### Bug fixes

- fixed minor mistakes in documentation

## Version 0.1.1

### Bug fixes

- fix failed `strum` re-export; users will need to pull in the derive macro `EnumIter` themselves
  - thanks to `@Shatur` for noticing this

## Version 0.1

- Released!
