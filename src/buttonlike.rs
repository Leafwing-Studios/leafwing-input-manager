//! Tools for working with button-like user inputs (mouse clicks, gamepad button, keyboard inputs and so on)

use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Current values of a button.
#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ButtonValue {
    /// Is the button currently pressed?
    pub pressed: bool,

    /// How far has the button been pressed,
    /// ranging from 0.0 (not pressed) to 1.0 (fully pressed).
    pub value: f32,
}

impl ButtonValue {
    /// Create a new [`ButtonValue`] with the given `pressed` state and `value`.
    #[inline]
    pub fn new(pressed: bool, value: f32) -> Self {
        Self { pressed, value }
    }

    /// Create a new [`ButtonValue`] with the given `pressed` state.
    ///
    /// The value will set to 1.0 if `pressed` is true, and 0.0 otherwise
    #[inline]
    pub fn from_pressed(pressed: bool) -> Self {
        Self::new(pressed, f32::from(pressed))
    }
}

impl From<ButtonState> for ButtonValue {
    fn from(value: ButtonState) -> Self {
        Self::from_pressed(value.pressed())
    }
}

impl From<bevy::input::ButtonState> for ButtonValue {
    fn from(value: bevy::input::ButtonState) -> Self {
        Self::from_pressed(value.is_pressed())
    }
}

/// The current state of a particular button,
/// usually corresponding to a single [`Actionlike`](crate::Actionlike) action.
///
/// By default, buttons are [`ButtonState::Released`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect, Default)]
pub enum ButtonState {
    /// The button has been pressed since the most recent tick
    JustPressed,
    /// This button is currently pressed (and was pressed before the most recent tick)
    Pressed,
    /// The button has been released since the most recent tick
    JustReleased,
    /// This button is currently released (and was released before the most recent tick)
    #[default]
    Released,
}

impl ButtonState {
    /// Causes [`just_pressed`](ButtonState::just_pressed) and [`just_released`](ButtonState::just_released) to become false
    ///
    /// [`JustPressed`](ButtonState::JustPressed) becomes [`Pressed`](ButtonState::Pressed) and
    /// [`JustReleased`](ButtonState::JustReleased) becomes [`Released`](ButtonState::Released)
    pub fn tick(&mut self) {
        use ButtonState::*;
        *self = match self {
            JustPressed => Pressed,
            Pressed => Pressed,
            JustReleased => Released,
            Released => Released,
        }
    }

    /// Presses the button
    ///
    /// It will be [`JustPressed`](ButtonState::JustPressed), unless it was already [`Pressed`](ButtonState::Pressed)
    #[inline]
    pub fn press(&mut self) {
        if *self != ButtonState::Pressed {
            *self = ButtonState::JustPressed;
        }
    }

    /// Releases the button
    ///
    /// It will be [`JustReleased`](ButtonState::JustReleased), unless it was already [`Released`](ButtonState::Released)
    #[inline]
    pub fn release(&mut self) {
        if *self != ButtonState::Released {
            *self = ButtonState::JustReleased;
        }
    }

    /// Is the button currently pressed?
    #[inline]
    #[must_use]
    pub fn pressed(&self) -> bool {
        *self == ButtonState::Pressed || *self == ButtonState::JustPressed
    }

    /// Is the button currently released?
    #[inline]
    #[must_use]
    pub fn released(&self) -> bool {
        *self == ButtonState::Released || *self == ButtonState::JustReleased
    }

    /// Has the button been pressed since the last time [`ActionState::update`](crate::action_state::ActionState::update) was called?
    #[inline]
    #[must_use]
    pub fn just_pressed(&self) -> bool {
        *self == ButtonState::JustPressed
    }

    /// Has the button been released since the last time [`ActionState::update`](crate::action_state::ActionState::update) was called?
    #[inline]
    #[must_use]
    pub fn just_released(&self) -> bool {
        *self == ButtonState::JustReleased
    }
}
