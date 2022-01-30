# Release Notes

## Version 0.2 (Unreleased)

### Enhancements

- configure how "clashing" inputs should be handled with the `ClashStrategy` field of your `InputMap`
  - very useful for working with modifier keys
  - if two actions are triggered
- ergonomic input mocking API at both the `App` and `World` level using the `MockInputs` trait

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

### Bug fixes

- the `ActionState` component is no longer marked as `Changed` every frame
- `InputManagerPlugin::run_in_state` now actually works!

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
