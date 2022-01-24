# Release Notes

## Version 0.1.2

### Usability

- added `set_state` method, allowing users to transfer `VirtualButtonState` between `ActionState` without losing `Timing` information

### Bugs

- fixed minor mistakes in documentation

## Version 0.1.1

### Bugs

- fix failed `strum` re-export; users will need to pull in the derive macro `EnumIter` themselves
  - thanks to @Shatur for noticing this

## Version 0.1

- Released!
