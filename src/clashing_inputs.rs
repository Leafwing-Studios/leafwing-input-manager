//! Handles clashing inputs into a [`InputMap`](crate::input_map::InputMap) in a configurable fashion.

use crate::input_map::InputMap;
use crate::user_input::{InputButton, UserInput};
use crate::Actionlike;
use bevy::utils::HashSet;
use itertools::Itertools;
use petitset::PetitSet;

/// How should clashing inputs by handled by an [`InputMap`]?
///
/// Inputs "clash" if and only if one [`UserInput`] is a strict subset of the other.
/// By example:
///
/// - `S` and `W`: does not clash
/// - `LControl + S` and `S`: clashes
/// - `S` and `S`: does not clash
/// - `LControl + S` and `LAlt + S`: clashes
/// - `LControl + S`, `LAlt + S` and `LControl + LAlt + S`: clashes
///
/// This strategy is only used when assessing the actions and input holistically,
/// in [`InputMap::which_pressed`], using [`InputMap::handle_clashes`].
#[non_exhaustive]
#[derive(Clone, PartialEq, Debug)]
pub enum ClashStrategy {
    /// All matching inputs will always be pressed
    PressAll,
    /// Only press the action that corresponds to the longest chord
    PrioritizeLongest,
    /// If the [`UserInput`] contains a modifier key (defined at the input map level), press that action over any unmodified action.
    ///
    /// If more than one matching action uses a modifier, break ties based on number of modifiers.
    /// Further ties are broken using the `PrioritizeLongest` rule.
    PrioritizeModified,
    /// Use the order in which actions are defined in the enum to resolve clashing inputs
    ///
    /// Uses the iteration order returned by [IntoEnumIterator](crate::IntoEnumIterator),
    /// which is generated in order of the enum items by the `#[derive(EnumIter)]` macro.
    UseActionOrder,
}

impl Default for ClashStrategy {
    fn default() -> Self {
        ClashStrategy::PressAll
    }
}

impl UserInput {
    /// Does `self` clash with `other`?
    pub fn clashes(&self, other: &UserInput) -> bool {
        use UserInput::*;

        match self {
            Null => false,
            Single(self_button) => match other {
                Null => false,
                Single(_) => false,
                Chord(other_set) => button_chord_clash(self_button, other_set),
            },
            Chord(self_set) => match other {
                Null => false,
                Single(other_button) => button_chord_clash(other_button, self_set),
                Chord(other_set) => chord_chord_clash(self_set, other_set),
            },
        }
    }
}

impl<A: Actionlike> InputMap<A> {
    /// Resolve clashing inputs, removing action presses that have been overruled
    pub fn handle_clashes(
        &self,
        _pressed_actions: &mut HashSet<A>,
        _pressed_inputs: PetitSet<UserInput, 500>,
    ) {
        match self.clash_strategy {
            ClashStrategy::PressAll => (),
            ClashStrategy::PrioritizeLongest => (),
            ClashStrategy::PrioritizeModified => (),
            ClashStrategy::UseActionOrder => (),
        };
    }

    /// Gets the set of clashing action-input pairs
    ///
    /// Returns both the action and [`UserInput`] for each clashing set
    pub fn get_clashes(
        &self,
        pressed_actions: &HashSet<A>,
        pressed_inputs: &PetitSet<UserInput, 500>,
    ) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        for action_pair in pressed_actions.iter().combinations(2) {
            let action_a = *action_pair.get(0).unwrap();
            let action_b = *action_pair.get(0).unwrap();

            if let Some(clash) = self.clashes(action_a, action_b, pressed_inputs) {
                clashes.push(clash);
            }
        }

        clashes
    }

    /// Is it possible for a pair of actions to clash given the provided input map?
    // TODO: use this to cache
    pub fn can_clash(&self, action_a: A, action_b: A) -> bool {
        for input_a in self.get(action_a, None) {
            for input_b in self.get(action_b, None) {
                if input_a.clashes(&input_b) {
                    return true;
                }
            }
        }
        false
    }

    /// Given the `pressed_inputs`, are there any clashes between the two actions?
    ///
    /// Returns `Some(clash)` if they are clashing, and `None` if they are not.
    pub fn clashes(
        &self,
        action_a: &A,
        action_b: &A,
        pressed_inputs: &PetitSet<UserInput, 500>,
    ) -> Option<Clash<A>> {
        let mut clash = Clash::new(*action_a, *action_b);

        for input_a in self
            .get(*action_a, None)
            .iter()
            .filter(|input| pressed_inputs.contains(input))
        {
            for input_b in self
                .get(*action_b, None)
                .iter()
                .filter(|input| pressed_inputs.contains(input))
            {
                if input_a.clashes(input_b) {
                    clash.inputs_a.push(input_a.clone());
                    clash.inputs_b.push(input_a.clone());
                }
            }
        }
        None
    }
}

/// A user-input clash, which stores the actions that are being clashed on,
/// as well as the corresponding user inputs
#[derive(Debug, Clone)]
pub struct Clash<A: Actionlike> {
    action_a: A,
    action_b: A,
    inputs_a: Vec<UserInput>,
    inputs_b: Vec<UserInput>,
}

impl<A: Actionlike> Clash<A> {
    /// Creates a new clash between the two actions
    fn new(action_a: A, action_b: A) -> Self {
        Self {
            action_a,
            action_b,
            inputs_a: Vec::default(),
            inputs_b: Vec::default(),
        }
    }

    /// Provides references to the actions that are clashing
    pub fn actions(&self) -> (&A, &A) {
        (&self.action_a, &self.action_b)
    }

    /// Provides references to the inputs that are clashing
    pub fn inputs(&self) -> (&Vec<UserInput>, &Vec<UserInput>) {
        (&self.inputs_a, &self.inputs_b)
    }
}

/// Does the `button` clash with the `chord`?
fn button_chord_clash(button: &InputButton, chord: &PetitSet<InputButton, 8>) -> bool {
    if chord.len() <= 1 {
        return false;
    }

    chord.contains(button)
}

/// Does the `chord_a` clash with `chord_b`?
fn chord_chord_clash(
    chord_a: &PetitSet<InputButton, 8>,
    chord_b: &PetitSet<InputButton, 8>,
) -> bool {
    if chord_a.len() <= 1 || chord_b.len() <= 1 {
        return false;
    }

    if chord_a == chord_b {
        return false;
    }

    chord_a.is_subset(chord_b) || chord_b.is_subset(chord_a)
}
