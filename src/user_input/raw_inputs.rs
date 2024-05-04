//! Raw input events that make up a user input.

use std::hash::Hash;

use bevy::prelude::{GamepadAxisType, GamepadButtonType, KeyCode, MouseButton};
use itertools::Itertools;

use crate::axislike::DualAxisType;
use crate::user_input::{GamepadControlDirection, MouseMoveDirection, MouseScrollDirection};

/// The basic input events that make up a [`UserInput`](crate::user_input::UserInput).
///
/// Typically obtained by calling [`UserInput::raw_inputs`](crate::user_input::UserInput::raw_inputs).
#[derive(Default, Debug, Clone, PartialEq)]
#[must_use]
pub struct RawInputs {
    /// Gamepad buttons, independent of a [`Gamepad`](bevy::input::gamepad::Gamepad)
    pub gamepad_buttons: Vec<GamepadButtonType>,

    /// Gamepad axes, independent of a [`Gamepad`](bevy::input::gamepad::Gamepad)
    pub gamepad_axes: Vec<GamepadAxisType>,

    /// Gamepad stick directions, independent of a [`Gamepad`](bevy::input::gamepad::Gamepad)
    pub gamepad_control_directions: Vec<GamepadControlDirection>,

    /// Physical key locations.
    pub keycodes: Vec<KeyCode>,

    /// Mouse buttons
    pub mouse_buttons: Vec<MouseButton>,

    /// Continuous mouse wheel inputs.
    pub mouse_scroll_axes: Vec<DualAxisType>,

    /// Discrete mouse wheel inputs.
    pub mouse_scroll_directions: Vec<MouseScrollDirection>,

    /// Continuous mouse move inputs.
    pub mouse_move_axes: Vec<DualAxisType>,

    /// Discrete mouse move inputs.
    pub mouse_move_directions: Vec<MouseMoveDirection>,
}

/// Create a list that contains only the unique elements in the given lists.
#[inline]
fn merge_unique<T: Copy + Eq + Hash>(list_a: &[T], list_b: &[T]) -> Vec<T> {
    list_a.iter().chain(list_b).unique().copied().collect()
}

impl RawInputs {
    /// Merges the raw input events from the [`RawInputs`] of the given `other` input into this one.
    /// This is useful for accumulating input events from multiple sources.
    #[inline]
    pub fn merge_input(mut self, other: &Self) -> Self {
        self.gamepad_buttons = merge_unique(&self.gamepad_buttons, &other.gamepad_buttons);
        self.gamepad_axes = merge_unique(&self.gamepad_axes, &other.gamepad_axes);
        self.gamepad_control_directions = merge_unique(
            &self.gamepad_control_directions,
            &other.gamepad_control_directions,
        );
        self.keycodes = merge_unique(&self.keycodes, &other.keycodes);
        self.mouse_buttons = merge_unique(&self.mouse_buttons, &other.mouse_buttons);
        self.mouse_scroll_axes = merge_unique(&self.mouse_scroll_axes, &other.mouse_scroll_axes);
        self.mouse_scroll_directions = merge_unique(
            &self.mouse_scroll_directions,
            &other.mouse_scroll_directions,
        );
        self.mouse_move_axes = merge_unique(&self.mouse_move_axes, &other.mouse_move_axes);
        self.mouse_move_directions =
            merge_unique(&self.mouse_move_directions, &other.mouse_move_directions);
        self
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `buttons` of type [`GamepadButtonType`].
    #[inline]
    pub fn from_gamepad_buttons(buttons: impl IntoIterator<Item = GamepadButtonType>) -> Self {
        Self {
            gamepad_buttons: buttons.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `axes` of type [`GamepadAxisType`].
    #[inline]
    pub fn from_gamepad_axes(axes: impl IntoIterator<Item = GamepadAxisType>) -> Self {
        Self {
            gamepad_axes: axes.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `directions` of type [`GamepadControlDirection`].
    #[inline]
    pub fn from_gamepad_control_directions(
        directions: impl IntoIterator<Item = GamepadControlDirection>,
    ) -> Self {
        Self {
            gamepad_control_directions: directions.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `keys` of type [`KeyCode`].
    #[inline]
    pub fn from_keycodes(keys: impl IntoIterator<Item = KeyCode>) -> Self {
        Self {
            keycodes: keys.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `buttons` of type [`MouseButton`].
    #[inline]
    pub fn from_mouse_buttons(buttons: impl IntoIterator<Item = MouseButton>) -> Self {
        Self {
            mouse_buttons: buttons.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `axes` of type [`DualAxisType`]
    /// for mouse scrolling.
    #[inline]
    pub fn from_mouse_scroll_axes(axes: impl IntoIterator<Item = DualAxisType>) -> Self {
        Self {
            mouse_scroll_axes: axes.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `directions` of type [`MouseScrollDirection`].
    #[inline]
    pub fn from_mouse_scroll_directions(
        directions: impl IntoIterator<Item = MouseScrollDirection>,
    ) -> Self {
        Self {
            mouse_scroll_directions: directions.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `axes` of type [`DualAxisType`]
    /// for mouse movement.
    #[inline]
    pub fn from_mouse_move_axes(axes: impl IntoIterator<Item = DualAxisType>) -> Self {
        Self {
            mouse_move_axes: axes.into_iter().collect(),
            ..Default::default()
        }
    }

    /// Creates a [`RawInputs`] from the given iterator that yields `directions` of type [`MouseMoveDirection`].
    #[inline]
    pub fn from_mouse_move_directions(
        directions: impl IntoIterator<Item = MouseMoveDirection>,
    ) -> Self {
        Self {
            mouse_move_directions: directions.into_iter().collect(),
            ..Default::default()
        }
    }
}
