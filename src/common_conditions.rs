//! Run conditions for actions.

use crate::{prelude::ActionState, Actionlike};
use bevy::prelude::Res;

/// Stateful run condition that can be toggled via an action press using [`ActionState::just_pressed`].
pub fn action_toggle_active<A>(default: bool, action: A) -> impl FnMut(Res<ActionState<A>>) -> bool
where
    A: Actionlike + Clone,
{
    let mut active = default;
    move |action_state: Res<ActionState<A>>| {
        active ^= action_state.just_pressed(&action);
        active
    }
}

/// Run condition that is active if [`ActionState::pressed`] is true for the given action.
pub fn action_pressed<A>(action: A) -> impl FnMut(Res<ActionState<A>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state: Res<ActionState<A>>| action_state.pressed(&action)
}

/// Run condition that is active if [`ActionState::just_pressed`] is true for the given action.
pub fn action_just_pressed<A>(action: A) -> impl FnMut(Res<ActionState<A>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state: Res<ActionState<A>>| action_state.just_pressed(&action)
}

/// Run condition that is active if [`ActionState::just_released`] is true for the given action.
pub fn action_just_released<A>(action: A) -> impl FnMut(Res<ActionState<A>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state: Res<ActionState<A>>| action_state.just_released(&action)
}
