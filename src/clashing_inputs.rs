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
/// - `LControl + S` and ` LAlt + S`: clashes
/// - `LControl + S`, `LAlt + S` and `LControl + LAlt + S`: clashes
///
/// This strategy is only used when assessing the actions and input holistically,
/// in [`InputMap::which_pressed`], using [`InputMap::handle_clashes`].
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
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
        pressed_actions: &mut HashSet<A>,
        pressed_inputs: &HashSet<UserInput>,
    ) {
        for clash in self.get_clashes(pressed_actions, pressed_inputs) {
            // Remove the action in the pair that was overruled, if any
            if let Some(culled_action) = resolve_clash(
                &clash,
                &self.clash_strategy,
                pressed_inputs,
                &self.modifier_buttons,
            ) {
                pressed_actions.remove(&culled_action);
            }
        }
    }

    /// Gets the set of clashing action-input pairs
    ///
    /// Returns both the action and [`UserInput`] for each clashing set
    pub fn get_clashes(
        &self,
        pressed_actions: &HashSet<A>,
        pressed_inputs: &HashSet<UserInput>,
    ) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        // We can limit our search to the cached set of possibly clashing actions
        for clash in &self.possible_clashes {
            // Clashes can only occur if both actions were triggered
            // This is not strictly necessary, but saves work
            if !pressed_actions.contains(&clash.action_a)
                || !pressed_actions.contains(&clash.action_b)
            {
                continue;
            }

            // Check if the potential clash occured based on the pressed inputs
            if let Some(clash) = check_clash(clash, pressed_inputs) {
                clashes.push(clash)
            }
        }

        clashes
    }

    /// Updates the cache of possible input clashes
    pub fn cache_possible_clashes(&mut self) {
        let mut clashes = Vec::default();

        for action_pair in A::iter().combinations(2) {
            let action_a = *action_pair.get(0).unwrap();
            let action_b = *action_pair.get(0).unwrap();

            if let Some(clash) = self.can_clash(&action_a, &action_b) {
                clashes.push(clash);
            }
        }

        self.possible_clashes = clashes;
    }

    /// Is it possible for a pair of actions to clash given the provided input map?
    pub fn can_clash(&self, action_a: &A, action_b: &A) -> Option<Clash<A>> {
        let mut clash = Clash::new(*action_a, *action_b);

        for input_a in self.get(*action_a, None) {
            for input_b in self.get(*action_b, None) {
                if input_a.clashes(&input_b) {
                    clash.inputs_a.push(input_a.clone());
                    clash.inputs_b.push(input_a.clone());
                }
            }
        }

        if !clash.inputs_a.is_empty() {
            Some(clash)
        } else {
            None
        }
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

/// Given the `pressed_inputs`, does the provided clash actually occur?
///
/// Returns `Some(clash)` if they are clashing, and `None` if they are not.
pub fn check_clash<A: Actionlike>(
    clash: &Clash<A>,
    pressed_inputs: &HashSet<UserInput>,
) -> Option<Clash<A>> {
    let mut actual_clash = Clash::new(clash.action_a, clash.action_b);

    // For all inputs that were actually pressed that match action A
    for input_a in clash
        .inputs_a
        .iter()
        .filter(|input| pressed_inputs.contains(input))
    {
        // For all inputs that were actually pressed that match action B
        for input_b in clash
            .inputs_b
            .iter()
            .filter(|input| pressed_inputs.contains(input))
        {
            // If a clash was detected,
            if input_a.clashes(input_b) {
                actual_clash.inputs_a.push(input_a.clone());
                actual_clash.inputs_b.push(input_a.clone());
            }
        }
    }

    if !clash.inputs_a.is_empty() {
        Some(actual_clash)
    } else {
        None
    }
}

/// Which (if any) of the actions in the [`Clash`] should be discarded?
pub fn resolve_clash<A: Actionlike>(
    clash: &Clash<A>,
    clash_strategy: &ClashStrategy,
    pressed_inputs: &HashSet<UserInput>,
    modifiers: &HashSet<InputButton>,
) -> Option<A> {
    // Figure out why the actions are pressed
    let reasons_a_is_pressed: Vec<&UserInput> = clash
        .inputs_a
        .iter()
        .filter(|input| pressed_inputs.contains(input))
        .collect();

    let reasons_b_is_pressed: Vec<&UserInput> = clash
        .inputs_b
        .iter()
        .filter(|input| pressed_inputs.contains(input))
        .collect();

    // Clashes are spurious if the virtual buttons are pressed for any non-clashing reason
    for reason_a in reasons_a_is_pressed.iter() {
        for reason_b in reasons_b_is_pressed.iter() {
            // If there is at least one non-clashing reason why these buttons should both be pressed,
            // we can avoid resolving the clash completely
            if !reason_a.clashes(&reason_b) {
                return None;
            }
        }
    }

    // There's a real clash; resolve it according to the `clash_strategy`
    match clash_strategy {
        // Do nothing
        ClashStrategy::PressAll => None,
        // Remove the clashing action with the shorter chord
        ClashStrategy::PrioritizeLongest => {
            let longest_a: u8 = reasons_a_is_pressed
                .iter()
                .map(|input| input.len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            let longest_b: u8 = reasons_b_is_pressed
                .iter()
                .map(|input| input.len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            // A's longest matching input is shorter
            if longest_a < longest_b {
                Some(clash.action_a)
            // B's longest matching input is shorter
            } else if longest_b < longest_a {
                Some(clash.action_b)
            // A tie!
            } else {
                None
            }
        }
        // Remove the clashing action wtih the fewest modifier keys
        ClashStrategy::PrioritizeModified => {
            let most_modifiers_a: u8 = reasons_a_is_pressed
                .iter()
                .map(|input| input.n_matching(modifiers))
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            let most_modifiers_b: u8 = reasons_b_is_pressed
                .iter()
                .map(|input| input.n_matching(modifiers))
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            // A's most modified input is less modified than B's
            if most_modifiers_a < most_modifiers_b {
                Some(clash.action_a)
                // B's most modified input is less modified than B's
            } else if most_modifiers_b < most_modifiers_a {
                Some(clash.action_b)
            // A tie!
            } else {
                None
            }
        }
        // Remove the clashing action that comes later in the action enum
        ClashStrategy::UseActionOrder => {
            let mut action_to_remove = None;
            for action in A::iter() {
                if action == clash.action_a {
                    action_to_remove = Some(clash.action_b);
                    break;
                }

                if action == clash.action_b {
                    action_to_remove = Some(clash.action_a);
                    break;
                }
            }
            action_to_remove
        }
    }
}
