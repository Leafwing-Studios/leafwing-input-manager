# Release Notes

## Version 0.3.1

### Usability

- reduced required `derive_more` features
- removed `thiserror` dependency

### Docs

- properly documented the `ToggleActions` functionality, for dynamically enabling and disabling actions
- added doc examples to `ActionStateDriver`, which allows you to trigger actions based on entity properties
- document the need to add system ordering when you have other functionality running during `CoreStage::PreUpdate`
- hint to users that they may want to use multiple `Actionlike` enums

## Version 0.4

### Usability

- `ToggleActions<A>` is now an enum, rather than storing an internal bool

## Version 0.3

### Enhancements

- added `reasons_pressed` API on `ActionState`, which records the triggering inputs
  - you can use this to extract exact input information from analog inputs (like triggers or joysticks)
- added the ability to release user inputs during input mocking
- added `ActionState::consume(action)`, which allows you to consume a pressed action, ensuring it is not pressed until after it is otherwise released
- added geometric primitives (`Direction` and `Rotation`) for working with rotations in 2 dimensions
  - stay tuned for first-class directional input support!

### Usability

- if desired, users are now able to use the `ActionState` and `InputMap` structs as standalone resources
- reverted change from by-reference to by-value APIs for `Actionlike` types
  - this is more ergonomic (derive `Copy` when you can!), and somewhat faster in the overwhelming majority of uses
- relaxed `Hash` and `Eq` bounds on `Actionlike`
- `InputManagerPlugin::run_in_state` was replaced with `ToggleActions<A: Actionlike>` resource which controls whether or not the [`ActionState`] / [`InputMap`] pairs of type `A` are active.
- `ActionState::state` and `set_state` methods renamed to `button_state` and `set_button_state` for clarity
- simplified `VirtualButtonState` into a trivial enum `ButtonState`
  - other metadata (e.g. timing information and reasons pressed) is stored in the `ActionData` struct
  - users can now access the `ActionData` struct directly for each action in a `ActionState` struct, allowing full manual control for unusual needs
- removed a layer of indirection for fetching timing information: simply call `action_state.current_duration(Action::Jump)`, rather than `action_state.button_state(Action::Jump).current_duration()`
- fleshed out `ButtonState` API for better parity with `ActionState`
- removed `UserInput::Null`: this was never helpful and bloated match statements
  - insert this resource when you want to suppress input collection, and remove it when you're done
- renamed the `InputManagerSystem::Reset` system label to `InputManagerSystem::Tick`.
- refactored `InputMap`
  - removed methods that works with specific input mode.
  - removed `n_registered`, use `get(action).len()` instead.
  - added `insert_at` / `remove_at` to insert / remove input at specific index.
  - added `remove` remove input for specific mapping.
  - use `usize` for sizes as in other Rust containers.
- added `UserInput::raw_inputs`, which breaks down a `UserInput` into the constituent Bevy types (e.g. `KeyCode` and `MouseButton`)

### Bug fixes

- the `PartialOrd` implementation of `Timing` now correctly compares values on the basis of the current duration that the button has been held / released for

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
