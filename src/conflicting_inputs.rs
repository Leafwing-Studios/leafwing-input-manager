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
/// pub fn disable_keyboard(mut tracking_input: ResMut<TrackingInputType>) {
///     tracking_input.keyboard = false;
/// }
///
/// let mut app = App::new();
///
/// app.add_systems(PreUpdate, disable_keyboard);
/// ```
#[derive(Resource)]
pub struct TrackingInputType {
    /// Is tracking gamepad input?
    pub gamepad: bool,

    /// Is tracking keyboard input?
    pub keyboard: bool,

    /// Is tracking mouse input?
    pub mouse: bool,
}

impl Default for TrackingInputType {
    fn default() -> Self {
        Self {
            gamepad: true,
            keyboard: true,
            mouse: true,
        }
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
            tracking_input.mouse = false;
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
            tracking_input.keyboard = false;
        }

        // `wants_pointer_input` sometimes returns `false` after clicking or holding a button over a widget,
        // so `is_pointer_over_area` is also needed.
        if context.is_pointer_over_area() || context.wants_pointer_input() {
            tracking_input.mouse = false;
        }

        if !tracking_input.keyboard && !tracking_input.mouse {
            return;
        }
    }
}
