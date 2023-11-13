//! This module contains [`PressScheduler`] and its supporting methods and impls.
//!
//! The [`PressScheduler`] is an optional addition to an [`InputManagerBundle`](crate::InputManagerBundle),
//! which allows for a system that runs in [`Update`] later to correctly press an action.
//! Directly using [`ActionState::press`] would not work, as inputs submitted late would be wiped out by the input manager systems.
//! With this type, the action is pressed on the next time said systems run, making it so other systems can react to it correctly.

use bevy::{prelude::*, utils::HashSet};

use crate::{prelude::ActionState, Actionlike};

/// Allows for scheduling an action to be pressed for the next frame
#[derive(Component, Resource)]
pub struct PressScheduler<A: Actionlike> {
    set: HashSet<A>,
}

// Required because of footgun with derives and trait bounds
impl<A: Actionlike> Default for PressScheduler<A> {
    fn default() -> Self {
        Self {
            set: HashSet::default(),
        }
    }
}

impl<A: Actionlike> PressScheduler<A> {
    /// Schedule a press for this action for the next frame
    /// The action will be pressed the next time [`crate::systems::update_action_state`] runs.
    pub fn schedule_press(&mut self, action: A) {
        self.set.insert(action);
    }

    /// Applies the scheduled presses to the given [`ActionState`]
    pub fn apply(&mut self, action_state: &mut ActionState<A>) {
        for action in &self.set {
            action_state.press(action)
        }
        self.set.clear();
    }
}
