//! Handles clashing inputs into a [`InputMap`](crate::input_map::InputMap) in a configurable fashion.

use crate::input_map::InputMap;
use crate::user_input::{InputButton, InputStreams, UserInput};
use crate::Actionlike;
use bevy::utils::HashSet;
use itertools::Itertools;
use petitset::PetitSet;
use std::cmp::Ordering;

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
    fn clashes(&self, other: &UserInput) -> bool {
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
    pub fn handle_clashes(&self, pressed_actions: &mut HashSet<A>, input_streams: &InputStreams) {
        for clash in self.get_clashes(pressed_actions, input_streams) {
            // Remove the action in the pair that was overruled, if any
            if let Some(culled_action) = resolve_clash(
                &clash,
                &self.clash_strategy,
                input_streams,
                &self.modifier_buttons,
            ) {
                pressed_actions.remove(&culled_action);
            }
        }
    }

    /// Updates the cache of possible input clashes
    pub fn cache_possible_clashes(&mut self) {
        let mut clashes = Vec::default();

        for action_pair in A::iter().combinations(2) {
            let action_a = *action_pair.get(0).unwrap();
            let action_b = *action_pair.get(1).unwrap();

            if let Some(clash) = self.possible_clash(&action_a, &action_b) {
                clashes.push(clash);
            }
        }

        self.possible_clashes = clashes;
    }

    /// Gets the set of clashing action-input pairs
    ///
    /// Returns both the action and [`UserInput`]s for each clashing set
    fn get_clashes(
        &self,
        pressed_actions: &HashSet<A>,
        input_streams: &InputStreams,
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
            if let Some(clash) = check_clash(clash, input_streams) {
                clashes.push(clash)
            }
        }

        clashes
    }

    /// If the pair of actions could clash, how?
    fn possible_clash(&self, action_a: &A, action_b: &A) -> Option<Clash<A>> {
        let mut clash = Clash::new(*action_a, *action_b);

        for input_a in self.get(*action_a, None) {
            for input_b in self.get(*action_b, None) {
                if input_a.clashes(&input_b) {
                    clash.inputs_a.push(input_a.clone());
                    clash.inputs_b.push(input_b.clone());
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
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Clash<A: Actionlike> {
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

/// Given the `input_streams`, does the provided clash actually occur?
///
/// Returns `Some(clash)` if they are clashing, and `None` if they are not.
fn check_clash<A: Actionlike>(clash: &Clash<A>, input_streams: &InputStreams) -> Option<Clash<A>> {
    let mut actual_clash = Clash::new(clash.action_a, clash.action_b);

    // For all inputs that were actually pressed that match action A
    for input_a in clash
        .inputs_a
        .iter()
        .filter(|&input| input_streams.input_pressed(input))
    {
        // For all inputs that were actually pressed that match action B
        for input_b in clash
            .inputs_b
            .iter()
            .filter(|&input| input_streams.input_pressed(input))
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
fn resolve_clash<A: Actionlike>(
    clash: &Clash<A>,
    clash_strategy: &ClashStrategy,
    input_streams: &InputStreams,
    modifiers: &HashSet<InputButton>,
) -> Option<A> {
    // Figure out why the actions are pressed
    let reasons_a_is_pressed: Vec<&UserInput> = clash
        .inputs_a
        .iter()
        .filter(|&input| input_streams.input_pressed(input))
        .collect();

    let reasons_b_is_pressed: Vec<&UserInput> = clash
        .inputs_b
        .iter()
        .filter(|&input| input_streams.input_pressed(input))
        .collect();

    // Clashes are spurious if the virtual buttons are pressed for any non-clashing reason
    for reason_a in reasons_a_is_pressed.iter() {
        for reason_b in reasons_b_is_pressed.iter() {
            // If there is at least one non-clashing reason why these buttons should both be pressed,
            // we can avoid resolving the clash completely
            if !reason_a.clashes(reason_b) {
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

            match longest_a.cmp(&longest_b) {
                Ordering::Greater => Some(clash.action_b),
                Ordering::Less => Some(clash.action_a),
                Ordering::Equal => None,
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

            match most_modifiers_a.cmp(&most_modifiers_b) {
                Ordering::Greater => Some(clash.action_b),
                Ordering::Less => Some(clash.action_a),
                Ordering::Equal => None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Actionlike;
    use bevy::input::keyboard::KeyCode::*;
    use strum::EnumIter;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
    enum Action {
        One,
        Two,
        OneAndTwo,
        TwoAndThree,
        OneAndTwoAndThree,
        CtrlOne,
        AltOne,
        CtrlAltOne,
    }

    fn test_input_map() -> InputMap<Action> {
        use Action::*;

        let mut input_map = InputMap::default();

        input_map.insert(One, Key1);
        input_map.insert(Two, Key2);
        input_map.insert_chord(OneAndTwo, [Key1, Key2]);
        input_map.insert_chord(TwoAndThree, [Key2, Key3]);
        input_map.insert_chord(OneAndTwoAndThree, [Key1, Key2, Key3]);
        input_map.insert_chord(CtrlOne, [LControl, Key1]);
        input_map.insert_chord(AltOne, [LAlt, Key1]);
        input_map.insert_chord(CtrlAltOne, [LControl, LAlt, Key1]);

        input_map
    }

    #[test]
    fn clash_detection() {
        let a: UserInput = A.into();
        let b: UserInput = B.into();
        let c: UserInput = C.into();
        let ab = UserInput::chord([A, B]);
        let bc = UserInput::chord([B, C]);
        let abc = UserInput::chord([A, B, C]);

        assert!(!a.clashes(&b));
        assert!(a.clashes(&ab));
        assert!(!c.clashes(&ab));
        assert!(!ab.clashes(&bc));
        assert!(ab.clashes(&abc))
    }

    #[test]
    fn button_chord_clash_construction() {
        use Action::*;

        let input_map = test_input_map();

        let observed_clash = input_map.possible_clash(&One, &OneAndTwo).unwrap();
        let correct_clash = Clash {
            action_a: One,
            action_b: OneAndTwo,
            inputs_a: vec![Key1.into()],
            inputs_b: vec![UserInput::chord([Key1, Key2])],
        };

        assert_eq!(observed_clash, correct_clash);
    }

    #[test]
    fn chord_chord_clash_construction() {
        use Action::*;

        let input_map = test_input_map();

        let observed_clash = input_map
            .possible_clash(&OneAndTwoAndThree, &OneAndTwo)
            .unwrap();
        let correct_clash = Clash {
            action_a: OneAndTwoAndThree,
            action_b: OneAndTwo,
            inputs_a: vec![UserInput::chord([Key1, Key2, Key3])],
            inputs_b: vec![UserInput::chord([Key1, Key2])],
        };

        assert_eq!(observed_clash, correct_clash);
    }

    #[test]
    fn can_clash() {
        use Action::*;

        let input_map = test_input_map();

        assert!(input_map.possible_clash(&One, &Two).is_none());
        assert!(input_map.possible_clash(&One, &OneAndTwo).is_some());
        assert!(input_map.possible_clash(&One, &OneAndTwoAndThree).is_some());
        assert!(input_map.possible_clash(&One, &TwoAndThree).is_none());
        assert!(input_map
            .possible_clash(&OneAndTwo, &OneAndTwoAndThree)
            .is_some());
    }

    #[test]
    fn clash_caching() {
        use crate::user_input::InputMode;

        let mut input_map = test_input_map();
        // Possible clashes are cached upon initialization
        assert_eq!(input_map.possible_clashes.len(), 12);

        // Possible clashes are cached upon binding insertion
        input_map.insert(Action::Two, UserInput::chord([LControl, LAlt, Key1]));
        assert_eq!(input_map.possible_clashes.len(), 15);

        // Possible clashes are cached upon binding removal
        input_map.clear_action(Action::One, None);
        assert_eq!(input_map.possible_clashes.len(), 9);

        input_map.clear_action(Action::Two, Some(InputMode::Keyboard));
        assert_eq!(input_map.possible_clashes.len(), 4);
    }
}
