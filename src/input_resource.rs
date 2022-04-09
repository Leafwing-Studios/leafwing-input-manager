//! This module contains [`InputResource`] and its supporting methods and impls.

use crate::{Actionlike, ActionState, InputMap, UserInput};

/// Resource for Inputs
pub struct InputResource<A: Actionlike> {
    /// An [ActionState] component
    pub action_state: ActionState<A>,
    /// An [InputMap] component
    pub input_map: InputMap<A>,
}

impl<A: Actionlike> InputResource<A> {
    /// Creates a new [`InputResource`] configured with an [`InputMap`] configured from an iterator
    /// of `(action, user_input)` pairs
    ///
    /// To create an [`InputResource`]  with an empty input map, use the [`Default::default`]
    /// method instead.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::input_resource::InputResource;
    /// use leafwing_input_manager::Actionlike;
    /// use bevy_input::keyboard::KeyCode;
    ///
    /// #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    /// }
    ///
    /// let input_resource = InputResource::new([
    ///     (Action::Run, KeyCode::LShift),
    ///     (Action::Jump, KeyCode::Space),
    /// ]);
    ///
    /// assert_eq!(input_resource.input_map.len(), 2);
    /// ```
    #[must_use]
    pub fn new(bindings: impl IntoIterator<Item=(A, impl Into<UserInput>)>) -> Self {
        let mut input_map = InputMap::default();
        input_map.insert_multiple(bindings);

        Self {
            action_state: ActionState::default(),
            input_map,
        }
    }
}

impl<A: Actionlike> Default for InputResource<A> {
    fn default() -> Self {
        Self {
            action_state: ActionState::default(),
            input_map: InputMap::default(),
        }
    }
}