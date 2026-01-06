//! Run conditions for actions.

use crate::{Actionlike, prelude::ActionState};
use bevy::{
    ecs::system::{Single, SystemParam},
    log::warn,
    prelude::Res,
};

/// A system parameter that fetches the [`ActionState`] either from a resource or from a component. If both exist, the resource takes precedence.
#[derive(SystemParam)]
pub struct ActionStateParam<'w, 's, A>
where
    A: Actionlike + Clone,
{
    action_state_resource: Option<Res<'w, ActionState<A>>>,
    action_state_component: Option<Single<'w, 's, &'static ActionState<A>>>,
}
impl<'w, 's, A> ActionStateParam<'w, 's, A>
where
    A: Actionlike + Clone,
{
    fn as_ref(&self) -> Option<&ActionState<A>> {
        if self.action_state_resource.is_some() {
            self.action_state_resource.as_deref()
        } else if self.action_state_component.is_some() {
            Some(self.action_state_component.as_ref().unwrap())
        } else {
            let type_name = std::any::type_name::<A>();
            warn!(
                "No ActionState found for {type_name}. Please ensure that an ActionState resource is added, or that an InputMap component exists to provide an ActionState."
            );
            None
        }
    }
}

/// Stateful run condition that can be toggled via an action press using [`ActionState::just_pressed`].
pub fn action_toggle_active<A>(
    default: bool,
    action: A,
) -> impl for<'w, 's> FnMut(ActionStateParam<'w, 's, A>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state_params| {
        action_state_params
            .as_ref()
            .is_some_and(|state| state.pressed(&action))
            || default
    }
}

/// Run condition that is active if [`ActionState::pressed`] is true for the given action.
pub fn action_pressed<A>(action: A) -> impl for<'w, 's> FnMut(ActionStateParam<'w, 's, A>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state_params| {
        action_state_params
            .as_ref()
            .is_some_and(|state| state.pressed(&action))
    }
}

/// Run condition that is active if [`ActionState::just_pressed`] is true for the given action.
pub fn action_just_pressed<A>(
    action: A,
) -> impl for<'w, 's> FnMut(ActionStateParam<'w, 's, A>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state_params| {
        action_state_params
            .as_ref()
            .is_some_and(|state| state.just_pressed(&action))
    }
}

/// Run condition that is active if [`ActionState::just_released`] is true for the given action.
pub fn action_just_released<A>(
    action: A,
) -> impl for<'w, 's> FnMut(ActionStateParam<'w, 's, A>) -> bool
where
    A: Actionlike + Clone,
{
    move |action_state_params| {
        action_state_params
            .as_ref()
            .is_some_and(|state| state.just_released(&action))
    }
}
