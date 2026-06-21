//! Run conditions for actions.
//!
//! Each run condition reads the [`ActionState<A>`] of a single entity via
//! [`Single`]. If no such entity exists (or more than one does), the
//! condition resolves as if the action were unpressed.

use crate::{Actionlike, prelude::ActionState};
use bevy::prelude::Single;

/// Stateful run condition that can be toggled via an action press using [`ActionState::just_pressed`].
pub fn action_toggle_active<A>(
    default: bool,
    action: A,
) -> impl for<'w, 's> FnMut(Option<Single<'w, 's, &'static ActionState<A>>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state| {
        action_state.is_some_and(|state| state.pressed(&action)) || default
    }
}

/// Run condition that is active if [`ActionState::pressed`] is true for the given action.
pub fn action_pressed<A>(
    action: A,
) -> impl for<'w, 's> FnMut(Option<Single<'w, 's, &'static ActionState<A>>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state| action_state.is_some_and(|state| state.pressed(&action))
}

/// Run condition that is active if [`ActionState::just_pressed`] is true for the given action.
pub fn action_just_pressed<A>(
    action: A,
) -> impl for<'w, 's> FnMut(Option<Single<'w, 's, &'static ActionState<A>>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state| action_state.is_some_and(|state| state.just_pressed(&action))
}

/// Run condition that is active if [`ActionState::just_released`] is true for the given action.
pub fn action_just_released<A>(
    action: A,
) -> impl for<'w, 's> FnMut(Option<Single<'w, 's, &'static ActionState<A>>>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state| action_state.is_some_and(|state| state.just_released(&action))
}
