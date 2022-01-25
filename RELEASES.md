# Release Notes

## Version 0.2 (Unreleased)

### Enhancements

### Usability

- better decoupled `InputMap` and `ActionState`, providing an `InputMap::which_pressed` API and allowing `ActionState::update` to operate based on any `HashSet<A: Actionlike>` of pressed virtual buttons that you pass in
- `InputMap` now uses a collected `InputStreams` struct in all of its methods, and input methods are now optional
- `InputManagerPlugin` now works even if some input stream resources are missing

### Bug fixes

- the `ActionState` component is no longer marked as `Changed` every frame

## Version 0.1.2

### Usability

- added `set_state` method, allowing users to transfer `VirtualButtonState` between `ActionState` without losing `Timing` information

### Bugs

- fixed minor mistakes in documentation

## Version 0.1.1

### Bug fixes

- fix failed `strum` re-export; users will need to pull in the derive macro `EnumIter` themselves
  - thanks to @Shatur for noticing this

## Version 0.1

- Released!
