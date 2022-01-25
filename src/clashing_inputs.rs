//! Handles clashing inputs into a [`InputMap`](crate::input_map::InputMap) in a configurable fashion.

use crate::user_input::{InputButton, UserInput};
use bevy::input::keyboard::KeyCode;
use bevy::utils::HashSet;

/// How should clashing inputs by handled by an [`InputMap`](crate::input_map::InputMap)?
///
/// Inputs "clash" if and only if one [`UserInput`] is a strict subset of the other.
/// By example:
///
/// - `S` and `W`: does not clash
/// - `LControl + S` and `S`: clashes
/// - `S` and `S`: does not clash
/// - `LControl + S` and `LAlt + S`: clashes
/// - `LControl + S`, `LAlt + S` and `LControl + LAlt + S`: clashes
#[non_exhaustive]
#[derive(Clone, PartialEq, Debug)]
pub enum ClashStrategy {
    /// All matching inputs will always be pressed
    PressAll,
    /// Only press the action that corresponds to the longest chord
    ///
    /// In the case of a tie, all tied actions will be pressed.
    PrioritizeLongest,
    /// If the [`UserInput`] contains a modifier key, press that action over any unmodified action.
    ///
    /// If more than one matching action uses a modifier, break ties based on number of modifiers.
    /// If a tie persists, press all of them.
    PrioritizeModified(HashSet<InputButton>),
    /// Use the order in which actions are defined in the enum to break ties
    ///
    /// Uses the iteration order returned by [IntoEnumIterator](crate::IntoEnumIterator),
    /// which is generated in order of the enum items by the `#[derive(EnumIter)]` macro.
    UseActionOrder,
}

impl ClashStrategy {
    /// Creates a `ClashStrategy::PrioritizeModified` variant with the standard keyboard modifiers
    ///
    /// The list added is `[LAlt, RAlt, LControl, RControl, LShift, RShift, LWin, RWin]`
    pub fn default_modifiers() -> ClashStrategy {
        use KeyCode::*;

        Self::custom_modifiers([LAlt, RAlt, LControl, RControl, LShift, RShift, LWin, RWin])
    }

    /// Creates a `ClashStrategy::PrioritizeModified` variant with a custom set of modifiers
    ///
    /// These do not need to all be keyboard modifiers,
    /// although the iterator passed in must have a homogenous item type.
    ///
    /// # Example
    /// ```rust
    /// use leafwing_input_manager::user_input::InputButton;
    /// use leafwing_input_manger::clashing_inputs::ClashStrategy;
    ///
    /// let clash_strategy = ClashStrategy::custom_modifiers(
    /// 	[InputButton::Keyboard(KeyCode::LControl),
    /// 	 InputButton::Mouse(MouseButton::Left),
    ///      InputButton::Gamepad(GamepadButtonType::LeftTrigger),
    /// 	]
    /// )
    /// ```
    pub fn custom_modifiers(
        modifiers: impl IntoIterator<Item = impl Into<InputButton>>,
    ) -> ClashStrategy {
        let hash_set: HashSet<InputButton> =
            HashSet::from_iter(modifiers.into_iter().map(|buttonlike| buttonlike.into()));

        ClashStrategy::PrioritizeModified(hash_set)
    }
}

impl Default for ClashStrategy {
    fn default() -> Self {
        ClashStrategy::PressAll
    }
}

impl UserInput {
    /// Is `self` a strict superset of `other`?
    pub fn contains(&self, other: &UserInput) -> bool {
        if self == other {
            false
        } else {
            !self.contained_by(other)
        }
    }

    /// Is `self` a strict subset of `other`?
    pub fn contained_by(&self, other: &UserInput) -> bool {
        use UserInput::*;

        match self {
            Null => true,
            Single(button) => match other {
                Null => false,
                Single(_) => false,
                Chord(button_set) => button_set.contains(button) && button_set.len() > 1,
            },
            Chord(self_set) => match other {
                Null => false,
                Single(_) => false,
                Chord(other_set) => self_set.is_subset(other_set) && self_set != other_set,
            },
        }
    }
}
