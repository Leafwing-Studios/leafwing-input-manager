# Release Notes

## Version 0.15.0 (Unreleased)

### Enhancements

#### Trait-based input design

- added the `UserInput` trait, which can be divided into three subtraits: `Buttonlike`, `Axislike` and `DualAxislike`
  - the `InputControlKind` for each action can be set via the new `Actionlike::input_control_kind` method. The derive will assume that all actions are buttonlike.
  - many methods such as `get` on `InputMap` and `ActionState` have been split into three variants, one for each kind of input
- there is now a clear division between buttonlike, axislike and dualaxislike data
  - each action in an `Actionlike` enum now has a specific `InputControlKind`, mapping it to one of these three categories
  - if you are storing non-buttonlike actions (e.g. movement) inside of your Actionlike enum, you must manually implement the trait
  - pressed / released state can only be accessed for buttonlike data: invalid requests will always return released
  - `f32` values can only be accessed for axislike data: invalid requests will always return 0.0
  - `ActionData` has been renamed to `ButtonData`, and no longer holds a `value` or `DualAxisData`
  - 2-dimensional `DualAxisData` can only be accessed for dualaxislike data: invalid requests will always return (0.0, 0.0)
  - `Axislike` inputs can no longer be inserted directly into an `InputMap`: instead, use the `insert_axis` method
  - `Axislike` inputs can no longer be inserted directly into an `InputMap`: instead, use the `insert_dual_axis` method

#### More inputs

- added `UserInput` impls for gamepad input events:
  - implemented `UserInput` for Bevy’s `GamepadAxisType`-related inputs.
    - `GamepadStick`: `DualAxislike`, continuous or discrete movement events of the left or right gamepad stick along both X and Y axes.
    - `GamepadControlAxis`: `Axislike`, Continuous or discrete movement events of a `GamepadAxisType`.
    - `GamepadControlDirection`: `Buttonlike`, Discrete movement direction events of a `GamepadAxisType`, treated as a button press.
  - implemented `UserInput` for Bevy’s `GamepadButtonType` directly.
  - added `GamepadVirtualAxis`, which implements `Axislike`, similar to the old `UserInput::VirtualAxis` using two `GamepadButtonType`s.
  - added `GamepadVirtualDPad`, which implements `DualAxislike`, similar to the old `UserInput::VirtualDPad` using four `GamepadButtonType`s.
- added `UserInput` impls for keyboard inputs:
  - implemented `Buttonlike` for `KeyCode` and `ModifierKey`
  - implemented `Buttonlike` for `ModifierKey`.
  - added `KeyboardVirtualAxis`, which implements `Axislike`, similar to the old `UserInput::VirtualAxis` using two `KeyCode`s.
  - added `KeyboardVirtualDPad` which implements `DualAxislike`, similar to the old `UserInput::VirtualDPad` using four `KeyCode`s.
- added `UserInput` impls for mouse inputs:
  - implemented `UserInput` for movement-related inputs.
    - `MouseMove`: `DualAxislike`, continuous or discrete movement events of the mouse both X and Y axes.
    - `MouseMoveAxis`: `Axislike`, continuous or discrete movement events of the mouse on an axis, similar to the old `SingleAxis::mouse_motion_*`.
    - `MouseMoveDirection`: `Buttonlike`, discrete movement direction events of the mouse on an axis, similar to the old `MouseMotionDirection`.
  - implemented `UserInput` for wheel-related inputs.
    - `MouseScroll`: `DualAxislike`, continuous or discrete movement events of the mouse wheel both X and Y axes.
    - `MouseScrollAxis`: `Axislike`, continuous or discrete movement events of the mouse wheel on an axis, similar to the old `SingleAxis::mouse_wheel_*`.
    - `MouseScrollDirection`: `ButtonLike`, discrete movement direction events of the mouse wheel on an axis, similar to the old `MouseWheelDirection`.
- added `ButtonlikeChord`, `AxislikeChord` and `DualAxislikeChord` for combining multiple inputs, similar to the old `UserInput::Chord`.

#### Input Processors

Input processors allow you to create custom logic for axis-like input manipulation.

- added processor enums:
  - `AxisProcessor`: Handles single-axis values.
  - `DualAxisProcessor`: Handles dual-axis values.
- added processor traits for defining custom processors:
  - `CustomAxisProcessor`: Handles single-axis values.
  - `CustomDualAxisProcessor`: Handles dual-axis values.
  - added App extensions for registration of custom processors:
    - `register_axis_processor` for `CustomAxisProcessor`.
    - `register_dual_axis_processor` for `CustomDualAxisProcessor`.
- added built-in processors (variants of processor enums and `Into<Processor>` implementors):
  - Digital Conversion: Discretizes values, returning `-1.0`. `0.0` or `1.0`:
    - `AxisProcessor::Digital`: Single-axis digital conversion.
    - `DualAxisProcessor::Digital`: Dual-axis digital conversion.
  - Inversion: Reverses control (positive becomes negative, etc.)
    - `AxisProcessor::Inverted`: Single-axis inversion.
    - `DualAxisInverted`: Dual-axis inversion, implemented `Into<DualAxisProcessor>`.
  - Sensitivity: Adjusts control responsiveness (doubling, halving, etc.).
    - `AxisProcessor::Sensitivity`: Single-axis scaling.
    - `DualAxisSensitivity`: Dual-axis scaling, implemented `Into<DualAxisProcessor>`.
  - Value Bounds: Define the boundaries for constraining input values.
    - `AxisBounds`: Restricts single-axis values to a range, implemented `Into<AxisProcessor>` and `Into<DualAxisProcessor>`.
    - `DualAxisBounds`: Restricts single-axis values to a range along each axis, implemented `Into<DualAxisProcessor>`.
    - `CircleBounds`: Limits dual-axis values to a maximum magnitude, implemented `Into<DualAxisProcessor>`.
  - Deadzones: Ignores near-zero values, treating them as zero.
    - Unscaled versions:
      - `AxisExclusion`: Excludes small single-axis values, implemented `Into<AxisProcessor>` and `Into<DualAxisProcessor>`.
      - `DualAxisExclusion`: Excludes small dual-axis values along each axis, implemented `Into<DualAxisProcessor>`.
      - `CircleExclusion`: Excludes dual-axis values below a specified magnitude threshold, implemented `Into<DualAxisProcessor>`.
    - Scaled versions:
      - `AxisDeadZone`: Normalizes single-axis values based on `AxisExclusion` and `AxisBounds::default`, implemented `Into<AxisProcessor>` and `Into<DualAxisProcessor>`.
      - `DualAxisDeadZone`: Normalizes dual-axis values based on `DualAxisExclusion` and `DualAxisBounds::default`, implemented `Into<DualAxisProcessor>`.
      - `CircleDeadZone`: Normalizes dual-axis values based on `CircleExclusion` and `CircleBounds::default`, implemented `Into<DualAxisProcessor>`.
- implemented `WithAxisProcessingPipelineExt` to manage processors for `SingleAxis` and `VirtualAxis`, integrating the common processing configuration.
- implemented `WithDualAxisProcessingPipelineExt` to manage processors for `DualAxis` and `VirtualDpad`, integrating the common processing configuration.

### Usability

#### InputMap

- added new fluent builders for creating a new `InputMap<A>` with short configurations:
  - `fn with(mut self, action: A, input: impl UserInput)`.
  - `fn with_one_to_many(mut self, action: A, inputs: impl IntoIterator<Item = impl UserInput>)`.
  - `fn with_multiple(mut self, bindings: impl IntoIterator<Item = (A, impl UserInput)>) -> Self`.
  - `fn with_gamepad(mut self, gamepad: Gamepad) -> Self`.
- added new iterators over `InputMap<A>`:
  - `actions(&self) -> impl Iterator<Item = &A>` for iterating over all registered actions.
  - `bindings(&self) -> impl Iterator<Item = (&A, &dyn UserInput)>` for iterating over all registered action-input bindings.

#### ActionState

- removed `ToggleActions` resource in favor of new methods on `ActionState`: `disable_all`, `disable(action)`, `enable_all`, `enable(action)`, and `disabled(action)`.

### MockInput

- added new methods for the `MockInput` trait.
  - `fn press_input(&self, input: impl UserInput)` for simulating button and key presses.
  - `fn send_axis_values(&self, input: impl UserInput, values: impl IntoIterator<Item = f32>)` for sending value changed events to each axis represented by the input.
  - as well as methods for a specific gamepad.
- implemented the methods for `MutableInputStreams`, `World`, and `App`.

### QueryInput

- added new methods for the `QueryInput` trait
  - `fn read_axis_value` and `read_dual_axis_values`
  - as well as methods for working with a specific gamepad
- implemented the methods for `InputStreams`, `World`, and `App`

### Bugs

- fixed a bug where enabling a pressed action would read as `just_pressed`, and disabling a pressed action would read as `just_released`.
- fixed a bug in `InputStreams::button_pressed()` where unrelated gamepads were not filtered out when an `associated_gamepad` is defined.
- inputs are now handled correctly in the `FixedUpdate` schedule! Previously, the `ActionState`s were only updated in the `PreUpdate` schedule, so you could have situations where an action was marked as `just_pressed` multiple times in a row (if the `FixedUpdate` schedule ran multiple times in a frame) or was missed entirely (if the `FixedUpdate` schedule ran 0 times in a frame).
- Mouse motion and mouse scroll are now computed more efficiently and reliably, through the use of the new `AccumulatedMouseMovement` and `AccumulatedMouseScroll` resources.
- the `timing` field of the `ActionData` is now disabled by default. Timing information will only be collected
  if the `timing` feature is enabled. It is disabled by default because most games don't require timing information.
  (how long a button was pressed for)

### Tech debt

- removed `ActionStateDriver` and `update_action_state_from_interaction`, which allowed actions to be pressed by `bevy_ui` buttons
  - this feature was not widely used and can be easily replicated externally
  - the core pattern is simply calling `action_state.press(MyAction::Variant)` in one of your systems
- removed the `no_ui_priority` feature. To get this behavior, now just turn off the default `ui` feature
- removed the `orientation` module, migrating to `bevy_math::Rot2`
  - use the types provided in `bevy_math` instead

### Migration Guide

- renamed `InputMap::which_pressed` method to `process_actions` to better reflect its current functionality for clarity.
- the old `SingleAxis` is now:
  - `GamepadControlAxis` for gamepad axes.
  - `MouseMoveAxis::X` and `MouseMoveAxis::Y` for continuous mouse movement.
  - `MouseScrollAxis::X` and `MouseScrollAxis::Y` for continuous mouse wheel movement.
- the old `DualAxis` is now:
  - `GamepadStick` for gamepad sticks.
  - `MouseMove::default()` for continuous mouse movement.
  - `MouseScroll::default()` for continuous mouse wheel movement.
- the old `Modifier` is now `ModifierKey`.
- the old `MouseMotionDirection` is now `MouseMoveDirection`.
- the old `MouseWheelDirection` is now `MouseScrollDirection`.
- the old `UserInput::Chord` is now `InputChord`.
- the old `UserInput::VirtualAxis` is now:
  - `GamepadVirtualAxis` for four gamepad buttons.
  - `KeyboardVirtualAxis` for four keys.
  - `MouseMoveAxis::X.digital()` and `MouseMoveAxis::Y.digital()` for discrete mouse movement.
  - `MouseScrollAxis::X.digital()` and `MouseScrollAxis::Y.digital()` for discrete mouse wheel movement.
- the old `UserInput::VirtualDPad` is now:
  - `GamepadVirtualDPad` for four gamepad buttons.
  - `KeyboardVirtualDPad` for four keys.
  - `MouseMove::default().digital()` for discrete mouse movement.
  - `MouseScroll::default().digital()` for discrete mouse wheel movement.
- `ActionDiff::ValueChanged` is now `ActionDiff::AxisChanged`.
- `ActionDiff::AxisPairChanged` is now `ActionDiff::DualAxisChanged`.
- `InputMap::iter` has been split into `iter_buttonlike`, `iter_axislike` and `iter_dual_axislike`.
  - The same split has been done for `InputMap::bindings` and `InputMap::actions`.
- `ActionState::axis_pair` and `AxisState::clamped_axis_pair` now return a plain `Vec2` rather than an `Option<Vec2>` for consistency with their single axis and buttonlike brethren.
- `BasicInputs::clashed` is now `BasicInput::clashes_with` to improve clarity
- `BasicInputs::Group` is now `BasicInputs::Chord` to improve clarity
- `BasicInputs` now only tracks buttonlike user inputs, and a new `None` variant has been added
- Bevy's `bevy_gilrs` feature is now optional.
  - it is still enabled by leafwing-input-manager's default features.
  - if you're using leafwing-input-manager with `default_features = false`, you can readd it by adding `bevy/bevy_gilrs` as a dependency.
- removed `InputMap::build` method in favor of new fluent builder pattern (see 'Usability: InputMap' for details).
- removed `DeadZoneShape` in favor of new dead zone processors (see 'Enhancements: Input Processors' for details).
- refactored the fields and methods of `RawInputs` to fit the new input types.
- removed `Direction` type in favor of `bevy::math::primitives::Direction2d`.
- removed `MockInput::send_input` methods, in favor of new input mocking APIs (see 'Usability: MockInput' for details).
- `DualAxisData` has been removed, and replaced with a simple `Vec2` throughout
  - a new type with the `DualAxisData` name has been added, as a parallel to `ButtonData` and `AxisData`

## Version 0.14.0

- updated to Bevy 0.14
- this is strictly a compatibility release to ease migration; you should consider upgrading to version 0.15 when possible

## Version 0.13.3

### Bugs

- fixed a bug where `DualAxis` was being considered pressed even when its data was [0.0, 0.0].

### Usability

- added `InputManagerBundle::with_map(InputMap)` allowing you to create the bundle with the given `InputMap` and default `ActionState`.

## Version 0.13.2

### Usability

- added `with_threshold()` for const `SingleAxis` creation.
- added `horizontal_gamepad_face_buttons()` and `vertical_gamepad_face_buttons()` for `VirtualAxis`, similar to `VirtualDpad::gamepad_face_buttons()`.
- changed various creations of `DualAxis`, `VirtualAxis`, `VirtualDpad` into const functions as they should be:
  - `left_stick()`, `right_stick()` for `DualAxis`.
  - `from_keys()`, `horizontal_arrow_keys()`, `vertical_arrow_keys()`, `ad()`, `ws()`, `horizontal_dpad()`, `vertical_dpad()` for `VirtualAxis`.
  - `arrow_keys()`, `wasd()`, `dpad()`, `gamepad_face_buttons()`, `mouse_wheel()`, `mouse_motion()` for `VirtualDpad`.

## Version 0.13.1

### Breaking Changes

- removed the `block_ui_interactions` feature:
  - by default, this library will prioritize `bevy::ui`.
  - if you want to disable this priority, add the newly added `no_ui_priority` feature to your configuration.

### Bugs

- fixed a bug related to missing handling for `ActionState::consumed`

### Usability

- exported `ActionState::action_data_mut_or_default()`

## Version 0.13.0

### Breaking Changes

- `Modifier::Win` has been renamed to `Modifier::Super`, consistent with `KeyCode::SuperLeft` and `KeyCode::SuperRight`.
- both `KeyCode`-based logical keybindings and `ScanCode`-based physical keybindings are no longer supported; please migrate to:
  - `KeyCode`s are now representing physical keybindings.
  - `InputKind::Keyboard` has been removed.
  - `InputKind::KeyLocation` has been removed; please use `InputKind::PhysicalKey` instead.
  - All `ScanCode`s and `QwertyScanCode`s have been removed; please use `KeyCode` instead:
    - all letter keys now follow the format `KeyCode::Key<Letter>`, e.g., `ScanCode::K` is now `KeyCode::KeyK`.
    - all number keys over letters now follow the format `KeyCode::Digit<Number>`, e.g., `ScanCode::Key1` is now `KeyCode::Digit1`.
    - all arrow keys now follow the format `KeyCode::Arrow<Direction>`, e.g., `ScanCode::Up` is now `KeyCode::ArrowUp`.

### Usability

- `bevy` dependency has been bumped from 0.12 to 0.13.
- `bevy_egui` dependency has been bumped from 0.24 to 0.25.

## Version 0.12.1

### Usability

- added a table detailing supported Bevy versions in the README.md
- added a feature flag `asset` allowing optional `bevy::asset::Asset` derive for the `InputMap`
- exported `InputKind` in `prelude` module

### Bugs

- fixed compilation issues with no-default-features
- fixed [a bug](https://github.com/Leafwing-Studios/leafwing-input-manager/issues/471) related to incorrect updating of `ActionState`.

## Version 0.12

### Enhancements

- improved deadzone handling for both `DualAxis` and `SingleAxis` deadzones
  - all deadzones now scale the input so that it is continuous.
  - `DeadZoneShape::Cross` handles each axis separately, making a per-axis "snapping" effect
  - an input that falls on the exact boundary of a deadzone is now considered inside it
- added support in `ActionDiff` for value and axis_pair changes

### Usability

- `InputMap`s are now constructed with `(Action, Input)` pairs, rather than `(Input, Action)` pairs, which directly matches the underlying data model
- registered types in the reflection system
- added `InputMap::clear`
- added `ActionState::keys`
- exported `VirtualAxis` in `prelude` module

### Bugs

- registered types in the reflection system
- added `InputMap::clear`
- fixed [a bug](https://github.com/Leafwing-Studios/leafwing-input-manager/issues/430) related to incorrect axis data in `Chord` when not all buttons are pressed.

### Code Quality

- all non-insertion methods now take `&A: Actionlike` rather than `A: Actionlike` to avoid pointless cloning
- removed `multimap` dependency in favor of regular `HashMap` which allowed to derive `Reflect` for `InputMap`
- removed widely unused and untested dynamic actions functionality: this should be more feasible to implement directly with the changed architecture
- removed widely unused `PressScheduler` functionality: this can be re-implemented externally
- `ActionState` now stores a `HashMap` internally
  - `ActionState::update` now takes a `HashMap<A, ActionState>` rather than relying on ordering
  - `InputMap::which_pressed` now returns a `HashMap<A, ActionState>`
  - `handle_clashes` now takes a `HashMap<A, ActionState>`
  - `ClashStrategy::UseActionOrder` has been removed
- the `action_state` module has been pared down to something more reasonable in scope:
  - timing-related code now lives in its own `timing` module
  - `ActionStateDriver` code now lives in its own `action_driver` module
  - `ActionDiff`-related code now lives in its own `action_diff` module

## Version 0.11.2

- fixed [a bug](https://github.com/Leafwing-Studios/leafwing-input-manager/issues/285) with mouse motion and mouse wheel events being improperly counted
  - this was pre-existing, but dramatically worsened by the release of Bevy 0.12.1

## Version 0.11.1

- `bevy_egui` integration and the `egui` feature flag have been added back with the release of `bevy_egui` 0.23.

### Bugs

- A disabled `ToggleActions` of one `Action` now does not release other `Action`'s inputs.
- `bevy_egui` integration and the `egui` feature flag have been added back with the release of `bevy_egui` 0.23.

## Version 0.11

### Known Issues

- `bevy_egui` integration and the `egui` feature flag have been temporarily removed to ensure a timely release
- gamepad input mocking is not completely functional due to upstream changes: see [#407](https://github.com/Leafwing-Studios/leafwing-input-manager/issues/407)
  - additional experiments and information would be helpful!

### Breaking Changes

- The `UserInput::insert_at` method has been removed: build this abstraction into your input binding menus if desired.
- `InputMap::iter()` now returns a simple iterator of (action, input) pairs
  - As a result, the `InputMap::iter_inputs` method has been removed.
- The `InputMap::remove_at` API now returns `Some(removed_input)`, rather than just a `bool`.
- The serialization format for `InputMap` has changed. You will need to re-generate your input maps if you were storing these persistently.

### Enhancements

- Added `DeadZoneShape` for `DualAxis` which allows for different deadzones shapes: cross, rectangle, and ellipse.
- Added sensitivity for `SingleAxis` and `DualAxis`, allowing you to scale mouse, keypad and gamepad inputs differently for each action.
- Added a helper `from_keys` to `VirtualAxis` to simplify creating one from two keys

### Usability

- Added `block_ui_interactions` feature flag; when on, mouse input won't be read if any `bevy_ui` element has an active `Interaction`.
- Chords no longer have a max length.
- `InputMap`, `UserInput` and all of the contained types now implement `Reflect`. As a result, the trait bound on `Actionlike` has been changed from `TypePath` to `Reflect`.

### Bugs

- Fixed system order ambiguity between bevy_ui and update_action_state systems
- The input values of axis inputs in a `Chord` are now prioritized over buttons
- Fixed unassigned `InputMaps`s not receiving input from all connected gamepads

### Performance

- Removed the `petitset` dependency in favor of a `MultiMap` to reduce stack size of input types.
  - As a result, the `Actionlike` trait now has the additional `Hash` and `Eq` trait bounds
  - `UserInput::Chord` now stores a simple `Vec` of `InputKind`s

### Docs

- Fixed invalid example code in README
- Added example for setting default controls
- Added example for registering gamepads in a local multiplayer fashion

## Version 0.10

### Usability

- `bevy` dependency has been bumped from 0.10 to 0.11.
- `ActionLike` now requires Bevy's `TypePath` trait. Your actions will now need to derive `Reflect` or `TypePath`. See [bevy#7184](https://github.com/bevyengine/bevy/pull/7184)
- `QwertyScanCode` has had its variants renamed to match bevy's `KeyCode` variants.
  See [bevy#8792](https://github.com/bevyengine/bevy/pull/8792)

### Enhancements

- Changed `entity` field of `ActionStateDriver` to `targets: ActionStateDriverTarget` with variants for 0, 1, or multiple targets, to allow for one driver
  to update multiple entities if needed.
- Added builder-style functions to `SingleAxis`, `DualAxis`, and `VirtualDPad` that invert their output values, allowing, for example, binding inverted camera controls.

### Docs

- Added example for driving cursor position action from another entity.

### Usability

- Makes `run_if_enabled` public.

## Version 0.9.3

### Bugs

- Changed `Rotation` to be stored in millionths of a degree instead of tenths of a degree in order to reduce rounding errors.

### Usability

- Added `VirtualAxis::horizontal_dpad()` and `VirtualAxis::vertical_dpad()`.
- Do not read mouse input if any `bevy_ui` element have active `Interaction`.

## Version 0.9.2

### Bugs

- Fixed `DualAxis` inputs so deadzones apply across both axes, and filter
  out-of-range values correctly.

## Version 0.9.1

### Usability

- Added common run conditions for actions that mirrors input conditions in Bevy.

## Version 0.9.0

### Usability

- Added `ActionState::consume_all()` to consume all actions.
- `bevy_egui` dependency has been bumped from 0.19 to 0.20.
- `bevy` dependency has been bumped from 0.9 to 0.10.

### Enhancements

- Added **scan code** support, which enables you to define keybindings depending on the key position rather than the key output.
  This is useful to make the keybindings layout-independent and is commonly used for the WASD movement controls.
  - Use `ScanCode` to define the raw scan code values.
  - Use `QwertyScanCode` to define the scan code by the name of the key on the US QWERTY keyboard layout.
- The `Actionlike::N_VARIANTS` constant has been changed to a function.
- Added the `DynAction` type and various companions to enable more advanced use cases.

## Version 0.8.0

### Usability

- `bevy_egui` dependency has been bumped from 0.18 to 0.19.

## Version 0.7.2

### Usability

- Added custom implementation of the `Serialize` and `Deserialize` traits for `InputMap` to make the format more human readable.
- Added `TypeUuid` for `InputMap` to be able use it as asset without wrapper
- `ActionState` and its fields now implement `Reflect`. The type is automatically registered when the `InputManagerPlugin` is added.
- Added `PressScheduler`, used to defer action presses until the start of the next frame to ease scheduling.

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
  - Use the new `SingleAxis` and `DualAxis` types / variants when you care about the continuous values.
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
  - `Vec2::new` now takes two `f32` values for ergonomic reasons.
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
- removed a layer of indirection for fetching timing information: simply call `action_state.current_duration(&Action::Jump)`, rather than `action_state.button_state(Action::Jump).current_duration()`
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
- removed `strum` dependency by reimplementing the functionality, allowing users to define actions with only the `Actionlike` trait
- added the `get_at` and `index` methods on the `Actionlike` trait, allowing you to fetch a specific action by its position in the defining enum and vice versa
- `Copy` bound on `Actionlike` trait relaxed to `Clone`, allowing you to store non-copy data in your enum variants
- `Clone`, `PartialEq` and `Debug` trait impls for `ActionState`
- `get_pressed`, `get_just_pressed`, `get_released` and `get_just_released` methods on `ActionState`, for conveniently checking many action states at once

### Bug fixes

- the `ActionState` component is no longer marked as `Changed` every frame
- `InputManagerPlugin::run_in_state` now actually works!
- virtually all methods now take actions and inputs by reference, rather than by ownership, eliminating unnecessary copies

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
