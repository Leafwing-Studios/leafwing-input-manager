//! Handles clashing inputs into a [`InputMap`](crate::input_map::InputMap) in a configurable fashion.

use crate::action_state::ActionData;
use crate::axislike::VirtualDPad;
use crate::input_map::InputMap;
use crate::input_streams::InputStreams;
use crate::user_input::{InputKind, UserInput};
use crate::Actionlike;

use itertools::Itertools;
use petitset::PetitSet;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::marker::PhantomData;

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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ClashStrategy {
    /// All matching inputs will always be pressed
    PressAll,
    /// Only press the action that corresponds to the longest chord
    ///
    /// This is the default strategy.
    PrioritizeLongest,
    /// Use the order in which actions are defined in the enum to resolve clashing inputs
    ///
    /// Uses the iteration order returned by [`Actionlike::variants()`],
    /// which is generated in order of the enum items by the `#[derive(Actionlike)]` macro.
    UseActionOrder,
}

impl Default for ClashStrategy {
    fn default() -> Self {
        ClashStrategy::PrioritizeLongest
    }
}

impl UserInput {
    /// Does `self` clash with `other`?
    #[must_use]
    fn clashes(&self, other: &UserInput) -> bool {
        use UserInput::*;

        match self {
            Single(self_button) => match other {
                Single(_) => false,
                Chord(other_chord) => button_chord_clash(self_button, other_chord),
                VirtualDPad(other_dpad) => dpad_button_clash(other_dpad, self_button),
            },
            Chord(self_chord) => match other {
                Single(other_button) => button_chord_clash(other_button, self_chord),
                Chord(other_chord) => chord_chord_clash(self_chord, other_chord),
                VirtualDPad(other_dpad) => dpad_chord_clash(other_dpad, self_chord),
            },
            VirtualDPad(self_dpad) => match other {
                Single(other_button) => dpad_button_clash(self_dpad, other_button),
                Chord(other_chord) => dpad_chord_clash(self_dpad, other_chord),
                VirtualDPad(other_dpad) => dpad_dpad_clash(self_dpad, other_dpad),
            },
        }
    }
}

impl<A: Actionlike> InputMap<A> {
    /// Resolve clashing inputs, removing action presses that have been overruled
    ///
    /// The `usize` stored in `pressed_actions` corresponds to `Actionlike::index`
    pub fn handle_clashes(
        &self,
        action_data: &mut [ActionData],
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) {
        for clash in self.get_clashes(action_data, input_streams) {
            // Remove the action in the pair that was overruled, if any
            if let Some(culled_action) = resolve_clash(&clash, clash_strategy, input_streams) {
                action_data[culled_action.index()] = ActionData::default();
            }
        }
    }

    /// Updates the cache of possible input clashes
    pub(crate) fn possible_clashes(&self) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        for action_pair in A::variants().combinations(2) {
            let action_a = action_pair.get(0).unwrap().clone();
            let action_b = action_pair.get(1).unwrap().clone();

            if let Some(clash) = self.possible_clash(action_a, action_b) {
                clashes.push(clash);
            }
        }
        clashes
    }

    /// Gets the set of clashing action-input pairs
    ///
    /// Returns both the action and [`UserInput`]s for each clashing set
    #[must_use]
    fn get_clashes(
        &self,
        action_data: &[ActionData],
        input_streams: &InputStreams,
    ) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        // We can limit our search to the cached set of possibly clashing actions
        for clash in self.possible_clashes() {
            // Clashes can only occur if both actions were triggered
            // This is not strictly necessary, but saves work
            if action_data[clash.index_a].state.pressed()
                && action_data[clash.index_b].state.pressed()
            {
                // Check if the potential clash occured based on the pressed inputs
                if let Some(clash) = check_clash(&clash, input_streams) {
                    clashes.push(clash)
                }
            }
        }

        clashes
    }

    /// If the pair of actions could clash, how?
    #[must_use]
    fn possible_clash(&self, action_a: A, action_b: A) -> Option<Clash<A>> {
        let mut clash = Clash::new(action_a.clone(), action_b.clone());

        for input_a in self.get(action_a).iter() {
            for input_b in self.get(action_b.clone()).iter() {
                if input_a.clashes(input_b) {
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct Clash<A: Actionlike> {
    /// The `Actionlike::index` value corresponding to `action_a`
    index_a: usize,
    /// The `Actionlike::index` value corresponding to `action_b`
    index_b: usize,
    inputs_a: Vec<UserInput>,
    inputs_b: Vec<UserInput>,
    _phantom: PhantomData<A>,
}

impl<A: Actionlike> Clash<A> {
    /// Creates a new clash between the two actions
    #[must_use]
    fn new(action_a: A, action_b: A) -> Self {
        Self {
            index_a: action_a.index(),
            index_b: action_b.index(),
            inputs_a: Vec::default(),
            inputs_b: Vec::default(),
            _phantom: PhantomData::default(),
        }
    }

    /// Creates a new clash between the two actions based on their `Actionlike::index` indexes
    #[must_use]
    fn from_indexes(index_a: usize, index_b: usize) -> Self {
        Self {
            index_a,
            index_b,
            inputs_a: Vec::default(),
            inputs_b: Vec::default(),
            _phantom: PhantomData::default(),
        }
    }
}

// Does the `button` clash with the `chord`?
#[must_use]
fn button_chord_clash(button: &InputKind, chord: &PetitSet<InputKind, 8>) -> bool {
    if chord.len() <= 1 {
        return false;
    }

    chord.contains(button)
}

// Does the `dpad` clash with the `chord`?
#[must_use]
fn dpad_chord_clash(dpad: &VirtualDPad, chord: &PetitSet<InputKind, 8>) -> bool {
    if chord.len() <= 1 {
        return false;
    }

    for button in &[dpad.up, dpad.down, dpad.left, dpad.right] {
        if chord.contains(button) {
            return true;
        }
    }

    false
}

fn dpad_button_clash(dpad: &VirtualDPad, button: &InputKind) -> bool {
    for dpad_button in &[dpad.up, dpad.down, dpad.left, dpad.right] {
        if button == dpad_button {
            return true;
        }
    }

    false
}

fn dpad_dpad_clash(dpad1: &VirtualDPad, dpad2: &VirtualDPad) -> bool {
    for button1 in &[dpad1.up, dpad1.down, dpad1.left, dpad1.right] {
        for button2 in &[dpad2.up, dpad2.down, dpad2.left, dpad2.right] {
            if button1 == button2 {
                return true;
            }
        }
    }

    false
}

/// Does the `chord_a` clash with `chord_b`?
#[must_use]
fn chord_chord_clash(chord_a: &PetitSet<InputKind, 8>, chord_b: &PetitSet<InputKind, 8>) -> bool {
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
#[must_use]
fn check_clash<A: Actionlike>(clash: &Clash<A>, input_streams: &InputStreams) -> Option<Clash<A>> {
    let mut actual_clash: Clash<A> = Clash::from_indexes(clash.index_a, clash.index_b);

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
                actual_clash.inputs_b.push(input_b.clone());
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
#[must_use]
fn resolve_clash<A: Actionlike>(
    clash: &Clash<A>,
    clash_strategy: ClashStrategy,
    input_streams: &InputStreams,
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

    // Clashes are spurious if the actions are pressed for any non-clashing reason
    for reason_a in reasons_a_is_pressed.iter() {
        for reason_b in reasons_b_is_pressed.iter() {
            // If there is at least one non-clashing reason why these buttons should both be pressed,
            // we can avoid resolving the clash completely
            if !reason_a.clashes(reason_b) {
                return None;
            }
        }
    }

    println!("real clash");

    // There's a real clash; resolve it according to the `clash_strategy`
    match clash_strategy {
        // Do nothing
        ClashStrategy::PressAll => None,
        // Remove the clashing action with the shorter chord
        ClashStrategy::PrioritizeLongest => {
            let longest_a: usize = reasons_a_is_pressed
                .iter()
                .map(|input| input.len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            let longest_b: usize = reasons_b_is_pressed
                .iter()
                .map(|input| input.len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            match longest_a.cmp(&longest_b) {
                Ordering::Greater => Some(A::get_at(clash.index_b).unwrap()),
                Ordering::Less => Some(A::get_at(clash.index_a).unwrap()),
                Ordering::Equal => None,
            }
        } // Remove the clashing action that comes later in the action enum
        ClashStrategy::UseActionOrder => match clash.index_a.cmp(&clash.index_b) {
            Ordering::Greater => Some(A::get_at(clash.index_a).unwrap()),
            Ordering::Less => Some(A::get_at(clash.index_b).unwrap()),
            Ordering::Equal => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as leafwing_input_manager;
    use bevy::app::App;
    use bevy::input::keyboard::KeyCode::*;
    use leafwing_input_manager_macros::Actionlike;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum Action {
        One,
        Two,
        OneAndTwo,
        TwoAndThree,
        OneAndTwoAndThree,
        CtrlOne,
        AltOne,
        CtrlAltOne,
        MoveDPad,
        CtrlUp,
    }

    fn test_input_map() -> InputMap<Action> {
        use Action::*;

        let mut input_map = InputMap::default();

        input_map.insert(Key1, One);
        input_map.insert(Key2, Two);
        input_map.insert_chord([Key1, Key2], OneAndTwo);
        input_map.insert_chord([Key2, Key3], TwoAndThree);
        input_map.insert_chord([Key1, Key2, Key3], OneAndTwoAndThree);
        input_map.insert_chord([LControl, Key1], CtrlOne);
        input_map.insert_chord([LAlt, Key1], AltOne);
        input_map.insert_chord([LControl, LAlt, Key1], CtrlAltOne);
        input_map.insert(
            VirtualDPad {
                up: Up.into(),
                down: Down.into(),
                left: Left.into(),
                right: Right.into(),
            },
            MoveDPad,
        );
        input_map.insert_chord([LControl, Up], CtrlUp);

        input_map
    }

    mod basic_functionality {
        use crate::axislike::VirtualDPad;
        use crate::buttonlike::ButtonState;
        use crate::input_mocking::MockInput;
        use bevy::input::InputPlugin;
        use Action::*;

        use super::*;

        #[test]
        fn clash_detection() {
            let a: UserInput = A.into();
            let b: UserInput = B.into();
            let c: UserInput = C.into();
            let ab = UserInput::chord([A, B]);
            let bc = UserInput::chord([B, C]);
            let abc = UserInput::chord([A, B, C]);
            let axyz_dpad: UserInput = VirtualDPad {
                up: A.into(),
                down: X.into(),
                left: Y.into(),
                right: Z.into(),
            }
            .into();
            let abcd_dpad: UserInput = VirtualDPad {
                up: A.into(),
                down: B.into(),
                left: C.into(),
                right: D.into(),
            }
            .into();

            let ctrl_up: UserInput = UserInput::chord([Up, LControl]);
            let directions_dpad: UserInput = VirtualDPad {
                up: Up.into(),
                down: Down.into(),
                left: Left.into(),
                right: Right.into(),
            }
            .into();

            assert!(!a.clashes(&b));
            assert!(a.clashes(&ab));
            assert!(!c.clashes(&ab));
            assert!(!ab.clashes(&bc));
            assert!(ab.clashes(&abc));
            assert!(axyz_dpad.clashes(&a));
            assert!(axyz_dpad.clashes(&ab));
            assert!(!axyz_dpad.clashes(&bc));
            assert!(axyz_dpad.clashes(&abcd_dpad));
            assert!(ctrl_up.clashes(&directions_dpad));
        }

        #[test]
        fn button_chord_clash_construction() {
            let input_map = test_input_map();

            let observed_clash = input_map.possible_clash(One, OneAndTwo).unwrap();
            let correct_clash = Clash {
                index_a: One.index(),
                index_b: OneAndTwo.index(),
                inputs_a: vec![Key1.into()],
                inputs_b: vec![UserInput::chord([Key1, Key2])],
                _phantom: PhantomData::default(),
            };

            assert_eq!(observed_clash, correct_clash);
        }

        #[test]
        fn chord_chord_clash_construction() {
            let input_map = test_input_map();

            let observed_clash = input_map
                .possible_clash(OneAndTwoAndThree, OneAndTwo)
                .unwrap();
            let correct_clash = Clash {
                index_a: OneAndTwoAndThree.index(),
                index_b: OneAndTwo.index(),
                inputs_a: vec![UserInput::chord([Key1, Key2, Key3])],
                inputs_b: vec![UserInput::chord([Key1, Key2])],
                _phantom: PhantomData::default(),
            };

            assert_eq!(observed_clash, correct_clash);
        }

        #[test]
        fn can_clash() {
            let input_map = test_input_map();

            assert!(input_map.possible_clash(One, Two).is_none());
            assert!(input_map.possible_clash(One, OneAndTwo).is_some());
            assert!(input_map.possible_clash(One, OneAndTwoAndThree).is_some());
            assert!(input_map.possible_clash(One, TwoAndThree).is_none());
            assert!(input_map
                .possible_clash(OneAndTwo, OneAndTwoAndThree)
                .is_some());
        }

        #[test]
        fn clash_caching() {
            let mut input_map = test_input_map();
            // Possible clashes are cached upon initialization
            assert_eq!(input_map.possible_clashes().len(), 13);

            // Possible clashes are cached upon binding insertion
            input_map.insert(UserInput::chord([LControl, LAlt, Key1]), Action::Two);
            assert_eq!(input_map.possible_clashes().len(), 16);

            // Possible clashes are cached upon binding removal
            input_map.clear_action(Action::One);
            assert_eq!(input_map.possible_clashes().len(), 10);
        }

        #[test]
        fn resolve_prioritize_longest() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);

            let input_map = test_input_map();
            let simple_clash = input_map.possible_clash(One, OneAndTwo).unwrap();
            app.send_input(Key1);
            app.send_input(Key2);
            app.update();

            let input_streams = InputStreams::from_world(&app.world, None);

            assert_eq!(
                resolve_clash(
                    &simple_clash,
                    ClashStrategy::PrioritizeLongest,
                    &input_streams,
                ),
                Some(One)
            );

            let reversed_clash = input_map.possible_clash(OneAndTwo, One).unwrap();
            assert_eq!(
                resolve_clash(
                    &reversed_clash,
                    ClashStrategy::PrioritizeLongest,
                    &input_streams,
                ),
                Some(One)
            );

            let chord_clash = input_map
                .possible_clash(OneAndTwo, OneAndTwoAndThree)
                .unwrap();
            app.send_input(Key3);
            app.update();

            let input_streams = InputStreams::from_world(&app.world, None);

            assert_eq!(
                resolve_clash(
                    &chord_clash,
                    ClashStrategy::PrioritizeLongest,
                    &input_streams,
                ),
                Some(OneAndTwo)
            );
        }

        #[test]
        fn resolve_use_action_order() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);

            let input_map = test_input_map();
            let simple_clash = input_map.possible_clash(One, CtrlOne).unwrap();
            let reversed_clash = input_map.possible_clash(CtrlOne, One).unwrap();
            app.send_input(Key1);
            app.send_input(LControl);
            app.update();

            let input_streams = InputStreams::from_world(&app.world, None);

            assert_eq!(
                resolve_clash(&simple_clash, ClashStrategy::UseActionOrder, &input_streams,),
                Some(CtrlOne)
            );

            assert_eq!(
                resolve_clash(
                    &reversed_clash,
                    ClashStrategy::UseActionOrder,
                    &input_streams,
                ),
                Some(CtrlOne)
            );
        }

        #[test]
        fn handle_clashes() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);
            let input_map = test_input_map();

            app.send_input(Key1);
            app.send_input(Key2);
            app.update();

            let mut action_data = vec![ActionData::default(); Action::N_VARIANTS];
            action_data[One.index()].state = ButtonState::JustPressed;
            action_data[Two.index()].state = ButtonState::JustPressed;
            action_data[OneAndTwo.index()].state = ButtonState::JustPressed;

            input_map.handle_clashes(
                &mut action_data,
                &InputStreams::from_world(&app.world, None),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = vec![ActionData::default(); Action::N_VARIANTS];
            expected[OneAndTwo.index()].state = ButtonState::JustPressed;

            assert_eq!(action_data, expected);
        }

        // Checks that a clash between a VirtualDPad and a chord choses the chord
        #[test]
        fn handle_clashes_dpad_chord() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);
            let input_map = test_input_map();

            app.send_input(LControl);
            app.send_input(Up);
            app.update();

            let mut action_data = vec![ActionData::default(); Action::N_VARIANTS];
            action_data[MoveDPad.index()].state = ButtonState::JustPressed;
            action_data[CtrlUp.index()].state = ButtonState::JustPressed;

            input_map.handle_clashes(
                &mut action_data,
                &InputStreams::from_world(&app.world, None),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = vec![ActionData::default(); Action::N_VARIANTS];
            expected[CtrlUp.index()].state = ButtonState::JustPressed;

            assert_eq!(action_data, expected);
        }

        #[test]
        fn which_pressed() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);
            let input_map = test_input_map();

            app.send_input(Key1);
            app.send_input(Key2);
            app.send_input(LControl);
            app.update();

            let action_data = input_map.which_pressed(
                &InputStreams::from_world(&app.world, None),
                ClashStrategy::PrioritizeLongest,
            );

            for (i, action_data) in action_data.iter().enumerate() {
                if i == CtrlOne.index() || i == OneAndTwo.index() {
                    assert!(action_data.state.pressed());
                } else {
                    assert!(action_data.state.released());
                }
            }
        }
    }
}
