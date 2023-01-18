//! This module contains [`PressScheduler`] and its supporting methods and impls.
//!
//! The [`PressScheduler`] is an optional addition to an [`InputManagerBundle`](crate::InputManagerBundle),
//! which allows for a system that runs in [`CoreStage::Update`] later to correctly press an action.
//! Directly using [`ActionState::press`] would not work, as inputs submitted late would be wiped out by the input manager systems.
//! With this type, the action is pressed on the next time said systems run, making it so other systems can react to it correctly.

use std::marker::PhantomData;

use bevy::prelude::*;
use fixedbitset::FixedBitSet;

use crate::{prelude::ActionState, Actionlike};

/// Allows for scheduling an action to be pressed for the next frame
#[derive(Component, Resource)]
pub struct PressScheduler<A: Actionlike> {
    bitset: FixedBitSet,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Default for PressScheduler<A> {
    fn default() -> Self {
        Self {
            bitset: FixedBitSet::with_capacity(A::n_variants()),
            _phantom: Default::default(),
        }
    }
}

impl<A: Actionlike> PressScheduler<A> {
    /// Schedule a press for this action for the next frame
    /// The action will be pressed the next time [`systems::`]
    pub fn schedule_press(&mut self, action: A) {
        self.bitset.set(action.index(), true);
    }

    /// Applies the scheduled presses to the given [`ActionState`]
    pub fn apply(&mut self, action_state: &mut ActionState<A>) {
        for i in self.bitset.ones() {
            action_state.press(A::get_at(i).unwrap())
        }
        self.bitset.clear();
    }
}
