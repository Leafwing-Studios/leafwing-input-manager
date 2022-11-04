# Release Notes

## Unreleased

### Usability

- Added custom implementation of the `Serialize` and `Deserialize` traits for `InputMap` to make the format more human readable.

## Version 0.7.1

### Bugs

- `egui` feature now works correctly and more robustly if an `EguiPlugin` is not actually enabled.

## Version 0.7

### Enhancements

- Added `VirtualAxis` struct that can be supplied to an `InputMap` to trigger on two direction-representing inputs. 1-dimensional equivalent to `VirtualDPad`.

### Usability

- Added `egui` feature to not take specific input sources into account when egui is using them. For example, when the user clicks on a widget, the actions associated with the mouse will not be taken into account.
- `InputStreams` no longer stores an `Option` to an input stream type: all fields other than `associated_gamepad` are now required. This was not useful in practice and added significant complexity.

## Version 0.6.1

### Bugs

- no longer print "real clash" due to a missed debugging statement

## Version 0.6

### Enhancements

- Added the `Modifier` enum, to ergonomically capture the notion of "either control/alt/shift/windows key".
  - The corresponding `InputKind::Modifier` variant was added to match.
  - You can conveniently construct these using the `InputKind::modified` or `InputMap::insert_modified` methods.

### Usability

- Implemented `Eq` for `Timing` and `InputMap`.
- Held `ActionState` inputs will now be released when an `InputMap` is removed.
- Improve `ToggleActions`.
  - Make `_phantom` field public and rename into `phantom`.
  - Add `ToggleActions::ENABLED` and `ToggleActions::DISABLED`.
- Added `SingleAxis::negative_only` and `SingleAxis::positive_only` for triggering separate actions for each direction of an axis.
- `ActionData::action_data` now returns a reference, rather than a clone, for consistency and explicitness
- added `with_deadzone` methods to configure the deadzones for both `SingleAxis` and `DualAxis` inputs

## Version 0.5.2

### Bug fixes

- Fixed gamepad axes not filtering out inputs outside of the axis deadzone.
- Fixed `DualAxis::right_stick()` returning the y axis for the left stick.

## Version 0.5.1

### Bug fixes

- removed a missed `println` statement spamming "real conflict" that had been missed

## Version 0.5

### Enhancements

- Added gamepad axis support.
  - Use the new `SingleAxis` and `DualAxis` types / variants.
- Added mousewheel and mouse motion support.
  - Use the new `SingleAxis` and `DualAxis` types / variants when you care about the continous values.
  - Use the new `MouseWheelDirection` enum as an `InputKind`.
- Added `SingleAxis` and `DualAxis` structs that can be supplied to an `InputMap` to trigger on axis inputs.
- Added `VirtualDPad` struct that can be supplied to an `InputMap` to trigger on four direction-representing inputs.
- Added `ActionState::action_axis_pair()` which can return an `AxisPair` containing the analog values of a `SingleAxis`, `DualAxis`, or `VirtualDPad`.
- Added `ActionState::action_value()` which represents the floating point value of any action:
  - `1.0` or `0.0` for pressed or unpressed button-like inputs
  - a value (typically) in the range `-1.0..=1.0` for a single axis representing its analog input
  - or a value (typically) in the range `0.0..=1.0` for a dual axis representing the magnitude (length) of its vector.

### Usability

- If no gamepad is registered to a specific `InputMap`, inputs from any gamepad in the `Gamepads` resource will be used.
- Removed the `ActionState::reasons_pressed` API.
  - This API was quite complex, not terribly useful and had nontrivial performance overhead.
  - This was not needed for axislike inputs in the end.
- Added `Direction::try_new()` to fallibly create a new `Direction` struct (which cannot be created from the zero vector).
- Removed the `InputMode` enum.
  - This was poorly motivated and had no internal usages.
  - This could not accurately represent more complex compound input types.
- `ButtonKind` was renamed to `InputKind` to reflect the new non-button input types.
- Renamed `AxisPair` to `DualAxisData`.
  - `DualAxisData::new` now takes two `f32` values for ergonomic reasons.
  - Use `DualAxisData::from_xy` to construct this directly from a `Vec2` as before.
- Rotation is now measured from the positive x axis in a counterclockwise direction. This applies to both `Rotation` and `Direction`.
  - This increases consistency with `glam` and makes trigonometry easier.
- Added `Direction::try_from` which never panics; consider using this in place of `Direction::new`.
- Converting from a `Direction` (which uses a `Vec2` of `f32`'s internally) to a `Rotation` (which uses exact decidegrees) now has special cases to ensure all eight cardinal directions result in exact degrees.
  - For example, a unit vector pointing to the Northeast now always converts to a `Direction` with exactly 1350 decidegrees.
  - Rounding errors may still occur when converting from arbitrary directions to the other 3592 discrete decidegrees.
- `InputStreams` and `MutableInputStreams` no longer store e.g. `Option<Res<Input<MouseButton>>>`, and instead simply store `Res<Input<MouseButton>>`
  - This makes them much easier to work with and dramatically simplifies internal logic.
- `InputStreams::from_world` no longer requires `&mut World`, as it does not require mutable access to any resources.
- Renamed `InputMocking::send_input_to_gamepad` and `InputMocking::release_input_for_gamepad` to `InputMocking::send_input_as_gamepad` and `InputMocking::send_input_as_gamepad`.
- Added the `guess_gamepad` method to `InputStreams` and `MutableInputStreams`, which attempts to find an appropriate gamepad to use.
- `InputMocking::pressed` and `pressed_for_gamepad` no longer require `&mut self`.
- `UserInput::raw_inputs` now returns a `RawInputs` struct, rather than a tuple struct.
- The `mouse` and `keyboard` fields on the two `InputStreams` types are now named `mouse_button` and `keycode` respectively.

## Bug fixes

- mocked inputs are now sent at the low-level `Events` form, rather than in their `Input` format.
  - this ensures that user code that is reading these events directly can be tested accurately.

## Version 0.4.1

### Bug fixes

- fixed a compilation error caused by mistakenly renaming the macros crate

## Version 0.4

### Usability

- reduced required `derive_more` features
- removed `thiserror` dependency
- the order of all methods on `InputMap` is now `(input, action)`, rather than `(action, input`) to better match user mental models
  - this is a map-like struct: one presses `KeyCode::F` to `Actions::PayRespects`, not the other way around!
  - this includes the order of all paired tuples, including the returned values

### Bug fixes

- fixed serious bug that broke all functionality relating to durations that buttons were pressed or released for
  - `ActionState::tick` now takes the `Instant` of both the current and previous frame, rather than just the current
- `InputManagerPlugin` no longer panics when time does not have a previous update
  - this is useful as it ensures `bevy_inspector_egui` compatibility!

### Docs

- properly documented the `ToggleActions` functionality, for dynamically enabling and disabling actions
- added doc examples to `ActionStateDriver`, which allows you to trigger actions based on entity properties
- document the need to add system ordering when you have other functionality running during `CoreStage::PreUpdate`
- hint to users that they may want to use multiple `Actionlike` enums

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
