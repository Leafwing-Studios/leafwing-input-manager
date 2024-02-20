//! Handles conflicting inputs from outside

use bevy::prelude::*;
#[cfg(feature = "egui")]
use bevy_egui::EguiContext;

/// Flags to enable specific input type tracking.
///
/// They can be disabled by setting their fields to `false`.
/// If you are dealing with conflicting input from other crates, this might be useful.
///
/// # Examples
///
/// ```
/// use bevy::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// let mut app = App::new();
///
/// // Always disable the gamepad input tracking
/// pub fn disable_gamepad(mut tracking_input: ResMut<TrackingInputType>) {
///     tracking_input.gamepad = TrackingState::Disabled;
/// }
/// app.add_systems(Startup, disable_gamepad);
///
/// // Ignore the tracked keyboard input until next tick
/// pub fn temporarily_ignore_keyboard(mut tracking_input: ResMut<TrackingInputType>) {
///     tracking_input.keyboard = TrackingState::IgnoredOnce;
/// }
/// app.add_systems(PreUpdate, temporarily_ignore_keyboard);
/// ```
#[derive(Resource, Default)]
pub struct TrackingInputType {
    /// Is tracking gamepad input?
    pub gamepad: TrackingState,

    /// Is tracking keyboard input?
    pub keyboard: TrackingState,

    /// Is tracking mouse input?
    pub mouse: TrackingState,
}

impl TrackingInputType {
    /// Causes all [`TrackingState::IgnoredOnce`] fields becomes [`TrackingState::Enabled`]
    pub(crate) fn tick(&mut self) {
        self.gamepad.tick();
        self.keyboard.tick();
        self.mouse.tick();
    }
}

/// The current state for input tracking.
#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub enum TrackingState {
    /// Tracking enabled
    #[default]
    Enabled,
    /// Tracking disabled
    Disabled,
    /// Ignore the tracked inputs temporarily until the next tick.
    IgnoredOnce,
}

impl TrackingState {
    /// Causes [`TrackingState::IgnoredOnce`] becomes [`TrackingState::Enabled`]
    pub(crate) fn tick(&mut self) {
        if *self == Self::IgnoredOnce {
            *self = Self::Enabled
        }
    }

    /// Returns `Some(f())` if the state is [`TrackingState::Enabled`],
    /// or `None` otherwise.
    pub fn then<T, F: FnOnce() -> T>(&self, f: F) -> Option<T> {
        (*self == Self::Enabled).then(f)
    }
}

/// Allow `bevy::ui` to take priority over actions when processing inputs.
#[cfg(all(feature = "ui", not(feature = "no_ui_priority")))]
pub fn prioritize_ui_inputs(
    query_interactions: Query<&Interaction>,
    mut tracking_input: ResMut<TrackingInputType>,
) {
    for interaction in query_interactions.iter() {
        // If use clicks on a button, do not apply them to the game state
        if *interaction != Interaction::None {
            tracking_input.mouse = TrackingState::IgnoredOnce;
            return;
        }
    }
}

/// Allow `egui` to take priority over actions when processing inputs.
#[cfg(feature = "egui")]
pub fn prioritize_egui_inputs(
    mut query_egui_context: Query<(Entity, &'static mut EguiContext)>,
    mut tracking_input: ResMut<TrackingInputType>,
) {
    for (_, mut egui_context) in query_egui_context.iter_mut() {
        let context = egui_context.get_mut();

        // If egui wants to own inputs, don't also apply them to the game state
        if context.wants_keyboard_input() {
            tracking_input.keyboard = TrackingState::IgnoredOnce;

            if tracking_input.mouse != TrackingState::Enabled {
                return;
            }
        }

        // `wants_pointer_input` sometimes returns `false` after clicking or holding a button over a widget,
        // so `is_pointer_over_area` is also needed.
        if context.is_pointer_over_area() || context.wants_pointer_input() {
            tracking_input.mouse = TrackingState::IgnoredOnce;

            if tracking_input.keyboard != TrackingState::Enabled {
                return;
            }
        }
    }
}
