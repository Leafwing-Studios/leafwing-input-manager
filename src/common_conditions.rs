//! Run conditions for actions.

use crate::{prelude::ActionState, Actionlike};
use bevy::prelude::Res;

/// Stateful run condition that can be toggled via an action press using [`ActionState::just_pressed`].
pub fn action_toggle_active<T>(default: bool, action: T) -> impl FnMut(Res<ActionState<T>>) -> bool
where
    T: Actionlike + Clone,
{
    let mut active = default;
    move |action_state: Res<ActionState<T>>| {
        active ^= action_state.just_pressed(action.clone());
        active
    }
}

/// Run condition that is active if [`ActionState::pressed`] is true for the given action.
pub fn action_pressed<T>(action: T) -> impl FnMut(Res<ActionState<T>>) -> bool
where
    T: Actionlike + Clone,
{
    move |action_state: Res<ActionState<T>>| action_state.pressed(action.clone())
}

/// Run condition that is active if [`ActionState::just_pressed`] is true for the given action.
pub fn action_just_pressed<T>(action: T) -> impl FnMut(Res<ActionState<T>>) -> bool
where
    T: Actionlike + Clone,
{
    move |action_state: Res<ActionState<T>>| action_state.just_pressed(action.clone())
}

/// Run condition that is active if [`ActionState::just_released`] is true for the given action.
pub fn action_just_released<T>(action: T) -> impl FnMut(Res<ActionState<T>>) -> bool
where
    T: Actionlike + Clone,
{
    move |action_state: Res<ActionState<T>>| action_state.just_released(action.clone())
}
