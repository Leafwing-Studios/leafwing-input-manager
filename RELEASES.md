# Release Notes

## Version 0.1.2

### Usability

- added `set_state` method, allowing users to transfer `VirtualButtonState` between `ActionState` without losing `Timing` information

### Bugs

- fixed minor mistakes in documentation

## Version 0.2 (Unreleased)

### Enhancements

- added `ActionState::set_state`, to allow users to transfer `VirtualButtonState` between `ActionState` components without losing `Timing` information

### Usability

### Bugs

- better decoupled `InputMap` and `ActionState`, ensuring that the `ActionState` component is no longer marked as `Changed` every frame

## Version 0.1.1

### Bugs

- fix failed `strum` re-export; users will need to pull in the derive macro `EnumIter` themselves
  - thanks to @Shatur for noticing this

## Version 0.1

- Released!
