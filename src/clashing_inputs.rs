//! Handles clashing inputs into a [`InputMap`] in a configurable fashion.
//!
//! [`Buttonlike`] actions can clash, if one is a strict subset of the other.
//! For example, the user might have bound `Ctrl + S` to save, and `S` to move down.
//! If the user presses `Ctrl + S`, the input manager should not also trigger the `S` action.

use std::cmp::Ordering;

use bevy::prelude::Resource;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use crate::input_map::InputMap;
use crate::input_streams::InputStreams;
use crate::user_input::Buttonlike;
use crate::{Actionlike, InputControlKind};

/// How should clashing inputs by handled by an [`InputMap`]?
///
/// Inputs "clash" if and only if the [`Buttonlike`] components of one user input is a strict subset of the other.
/// For example:
///
/// - `S` and `W`: does not clash
/// - `ControlLeft + S` and `S`: clashes
/// - `S` and `S`: does not clash
/// - `ControlLeft + S` and ` AltLeft + S`: clashes
/// - `ControlLeft + S`, `AltLeft + S` and `ControlLeft + AltLeft + S`: clashes
///
/// This strategy is only used when assessing the actions and input holistically,
/// in [`InputMap::process_actions`], using [`InputMap::handle_clashes`].
#[non_exhaustive]
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum ClashStrategy {
    /// All matching inputs will always be pressed
    PressAll,
    /// Only press the action that corresponds to the longest chord
    ///
    /// This is the default strategy.
    #[default]
    PrioritizeLongest,
}

impl ClashStrategy {
    /// Returns the list of all possible clash strategies.
    pub fn variants() -> &'static [ClashStrategy] {
        use ClashStrategy::*;

        &[PressAll, PrioritizeLongest]
    }
}

/// A flat list of the [`Buttonlike`] inputs that make up a [`UserInput`](crate::user_input::UserInput).
///
/// This is used to check for potential clashes between actions,
/// where one action is a strict subset of another.
#[derive(Debug, Clone)]
#[must_use]
pub enum BasicInputs {
    /// No buttonlike inputs are involved.
    ///
    /// This might be used for things like a joystick axis.
    None,

    /// The input consists of a single, fundamental [`Buttonlike`] [`UserInput`](crate::user_input::UserInput).
    ///
    /// For example, a single key press.
    Simple(Box<dyn Buttonlike>),

    /// The input can be triggered by multiple independent [`Buttonlike`] [`UserInput`](crate::user_input::UserInput)s,
    /// but is still fundamentally considered a single input.
    ///
    /// For example, a virtual D-Pad is only one input, but can be triggered by multiple keys.
    Composite(Vec<Box<dyn Buttonlike>>),

    /// The input represents one or more independent [`Buttonlike`] [`UserInput`](crate::user_input::UserInput) types.
    ///
    /// For example, a chorded input is a group of multiple keys that must be pressed together.
    Chord(Vec<Box<dyn Buttonlike>>),
}

impl BasicInputs {
    /// Returns a list of the underlying [`Buttonlike`] [`UserInput`](crate::user_input::UserInput)s.
    ///
    /// # Warning
    ///
    /// When checking for clashes, do not use this method to compute the length of the input.
    /// Instead, use [`BasicInputs::len`], as these do not always agree.
    #[inline]
    pub fn inputs(&self) -> Vec<Box<dyn Buttonlike>> {
        match self.clone() {
            Self::None => Vec::default(),
            Self::Simple(input) => vec![input],
            Self::Composite(inputs) => inputs,
            Self::Chord(inputs) => inputs,
        }
    }

    ///  Create a [`BasicInputs::Composite`] from two existing [`BasicInputs`].
    pub fn compose(self, other: BasicInputs) -> Self {
        let combined_inputs = self.inputs().into_iter().chain(other.inputs()).collect();

        BasicInputs::Composite(combined_inputs)
    }

    /// Returns the number of the logical [`Buttonlike`] [`UserInput`](crate::user_input::UserInput)s that make up the input.
    ///
    /// A single key press is one input, while a chorded input is multiple inputs.
    /// A composite input is still considered one input, even if it can be triggered by multiple keys,
    /// as only one input need actually be pressed.
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Simple(_) => 1,
            Self::Composite(_) => 1,
            Self::Chord(inputs) => inputs.len(),
        }
    }

    /// Checks if the given two [`BasicInputs`] clash with each other.
    #[inline]
    pub fn clashes_with(&self, other: &BasicInputs) -> bool {
        match (self, other) {
            (Self::None, _) | (_, Self::None) => false,
            (Self::Simple(_), Self::Simple(_)) => false,
            (Self::Simple(self_single), Self::Chord(other_group)) => {
                other_group.len() > 1 && other_group.contains(self_single)
            }
            (Self::Chord(self_group), Self::Simple(other_single)) => {
                self_group.len() > 1 && self_group.contains(other_single)
            }
            (Self::Simple(self_single), Self::Composite(other_composite)) => {
                other_composite.contains(self_single)
            }
            (Self::Composite(self_composite), Self::Simple(other_single)) => {
                self_composite.contains(other_single)
            }
            (Self::Composite(self_composite), Self::Chord(other_group)) => {
                other_group.len() > 1
                    && other_group
                        .iter()
                        .any(|input| self_composite.contains(input))
            }
            (Self::Chord(self_group), Self::Composite(other_composite)) => {
                self_group.len() > 1
                    && self_group
                        .iter()
                        .any(|input| other_composite.contains(input))
            }
            (Self::Chord(self_group), Self::Chord(other_group)) => {
                self_group.len() > 1
                    && other_group.len() > 1
                    && self_group != other_group
                    && (self_group.iter().all(|input| other_group.contains(input))
                        || other_group.iter().all(|input| self_group.contains(input)))
            }
            (Self::Composite(self_composite), Self::Composite(other_composite)) => {
                other_composite
                    .iter()
                    .any(|input| self_composite.contains(input))
                    || self_composite
                        .iter()
                        .any(|input| other_composite.contains(input))
            }
        }
    }
}

impl<A: Actionlike> InputMap<A> {
    /// Resolve clashing button-like inputs, removing action presses that have been overruled
    ///
    /// The `usize` stored in `pressed_actions` corresponds to `Actionlike::index`
    pub fn handle_clashes(
        &self,
        button_data: &mut HashMap<A, bool>,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) {
        for clash in self.get_clashes(button_data, input_streams) {
            // Remove the action in the pair that was overruled, if any
            if let Some(culled_action) = resolve_clash(&clash, clash_strategy, input_streams) {
                button_data.remove(&culled_action);
            }
        }
    }

    /// Updates the cache of possible input clashes
    pub(crate) fn possible_clashes(&self) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        for action_a in self.buttonlike_actions() {
            for action_b in self.buttonlike_actions() {
                if let Some(clash) = self.possible_clash(action_a, action_b) {
                    clashes.push(clash);
                }
            }
        }

        clashes
    }

    /// Gets the set of clashing action-input pairs
    ///
    /// Returns both the action and [`UserInput`](crate::user_input::UserInput)s for each clashing set
    #[must_use]
    fn get_clashes(
        &self,
        action_data: &HashMap<A, bool>,
        input_streams: &InputStreams,
    ) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        // We can limit our search to the cached set of possibly clashing actions
        for clash in self.possible_clashes() {
            let pressed_a = action_data
                .get(&clash.action_a)
                .copied()
                .unwrap_or_default();
            let pressed_b = action_data
                .get(&clash.action_b)
                .copied()
                .unwrap_or_default();

            // Clashes can only occur if both actions were triggered
            // This is not strictly necessary, but saves work
            if pressed_a && pressed_b {
                // Check if the potential clash occurred based on the pressed inputs
                if let Some(clash) = check_clash(&clash, input_streams) {
                    clashes.push(clash)
                }
            }
        }

        clashes
    }

    /// Gets the decomposed [`BasicInputs`] for each binding mapped to the given action.
    pub fn decomposed(&self, action: &A) -> Vec<BasicInputs> {
        match action.input_control_kind() {
            InputControlKind::Button => {
                let Some(buttonlike) = self.get_buttonlike(action) else {
                    return Vec::new();
                };

                buttonlike.iter().map(|input| input.decompose()).collect()
            }
            InputControlKind::Axis => {
                let Some(axislike) = self.get_axislike(action) else {
                    return Vec::new();
                };

                axislike.iter().map(|input| input.decompose()).collect()
            }
            InputControlKind::DualAxis => {
                let Some(dual_axislike) = self.get_dual_axislike(action) else {
                    return Vec::new();
                };

                dual_axislike
                    .iter()
                    .map(|input| input.decompose())
                    .collect()
            }
        }
    }

    /// If the pair of actions could clash, how?
    // FIXME: does not handle axis inputs. Should use the `decomposed` method instead of `get_buttonlike`
    #[must_use]
    fn possible_clash(&self, action_a: &A, action_b: &A) -> Option<Clash<A>> {
        let mut clash = Clash::new(action_a.clone(), action_b.clone());

        for input_a in self.get_buttonlike(action_a)? {
            for input_b in self.get_buttonlike(action_b)? {
                if input_a.decompose().clashes_with(&input_b.decompose()) {
                    clash.inputs_a.push(input_a.clone());
                    clash.inputs_b.push(input_b.clone());
                }
            }
        }

        let clashed = !clash.inputs_a.is_empty();
        clashed.then_some(clash)
    }
}

/// A user-input clash, which stores the actions that are being clashed on,
/// as well as the corresponding user inputs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct Clash<A: Actionlike> {
    action_a: A,
    action_b: A,
    inputs_a: Vec<Box<dyn Buttonlike>>,
    inputs_b: Vec<Box<dyn Buttonlike>>,
}

impl<A: Actionlike> Clash<A> {
    /// Creates a new clash between the two actions
    #[must_use]
    fn new(action_a: A, action_b: A) -> Self {
        Self {
            action_a,
            action_b,
            inputs_a: Vec::default(),
            inputs_b: Vec::default(),
        }
    }
}

/// Given the `input_streams`, does the provided clash actually occur?
///
/// Returns `Some(clash)` if they are clashing, and `None` if they are not.
#[must_use]
fn check_clash<A: Actionlike>(clash: &Clash<A>, input_streams: &InputStreams) -> Option<Clash<A>> {
    let mut actual_clash: Clash<A> = clash.clone();

    // For all inputs actually pressed that match action A
    for input_a in clash
        .inputs_a
        .iter()
        .filter(|&input| input.pressed(input_streams))
    {
        // For all inputs actually pressed that match action B
        for input_b in clash
            .inputs_b
            .iter()
            .filter(|&input| input.pressed(input_streams))
        {
            // If a clash was detected
            if input_a.decompose().clashes_with(&input_b.decompose()) {
                actual_clash.inputs_a.push(input_a.clone());
                actual_clash.inputs_b.push(input_b.clone());
            }
        }
    }

    let clashed = !clash.inputs_a.is_empty();
    clashed.then_some(actual_clash)
}

/// Which (if any) of the actions in the [`Clash`] should be discarded?
#[must_use]
fn resolve_clash<A: Actionlike>(
    clash: &Clash<A>,
    clash_strategy: ClashStrategy,
    input_streams: &InputStreams,
) -> Option<A> {
    // Figure out why the actions are pressed
    let reasons_a_is_pressed: Vec<&dyn Buttonlike> = clash
        .inputs_a
        .iter()
        .filter(|input| input.pressed(input_streams))
        .map(|input| input.as_ref())
        .collect();

    let reasons_b_is_pressed: Vec<&dyn Buttonlike> = clash
        .inputs_b
        .iter()
        .filter(|input| input.pressed(input_streams))
        .map(|input| input.as_ref())
        .collect();

    // Clashes are spurious if the actions are pressed for any non-clashing reason
    for reason_a in reasons_a_is_pressed.iter() {
        for reason_b in reasons_b_is_pressed.iter() {
            // If there is at least one non-clashing reason why these buttons should both be pressed,
            // we can avoid resolving the clash completely
            if !reason_a.decompose().clashes_with(&reason_b.decompose()) {
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
            let longest_a: usize = reasons_a_is_pressed
                .iter()
                .map(|input| input.decompose().len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            let longest_b: usize = reasons_b_is_pressed
                .iter()
                .map(|input| input.decompose().len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            match longest_a.cmp(&longest_b) {
                Ordering::Greater => Some(clash.action_b.clone()),
                Ordering::Less => Some(clash.action_a.clone()),
                Ordering::Equal => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;
    use bevy::input::keyboard::KeyCode::*;
    use bevy::prelude::Reflect;

    use super::*;
    use crate::prelude::{KeyboardVirtualDPad, UserInput};
    use crate::user_input::ButtonlikeChord;

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
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

    impl Actionlike for Action {
        fn input_control_kind(&self) -> crate::InputControlKind {
            match self {
                Self::MoveDPad => crate::InputControlKind::DualAxis,
                _ => crate::InputControlKind::Button,
            }
        }
    }

    fn test_input_map() -> InputMap<Action> {
        use Action::*;

        let mut input_map = InputMap::default();

        input_map.insert(One, Digit1);
        input_map.insert(Two, Digit2);
        input_map.insert(OneAndTwo, ButtonlikeChord::new([Digit1, Digit2]));
        input_map.insert(TwoAndThree, ButtonlikeChord::new([Digit2, Digit3]));
        input_map.insert(
            OneAndTwoAndThree,
            ButtonlikeChord::new([Digit1, Digit2, Digit3]),
        );
        input_map.insert(CtrlOne, ButtonlikeChord::new([ControlLeft, Digit1]));
        input_map.insert(AltOne, ButtonlikeChord::new([AltLeft, Digit1]));
        input_map.insert(
            CtrlAltOne,
            ButtonlikeChord::new([ControlLeft, AltLeft, Digit1]),
        );
        input_map.insert_dual_axis(MoveDPad, KeyboardVirtualDPad::ARROW_KEYS);
        input_map.insert(CtrlUp, ButtonlikeChord::new([ControlLeft, ArrowUp]));

        input_map
    }

    fn inputs_clash(input_a: impl UserInput, input_b: impl UserInput) -> bool {
        let decomposed_a = input_a.decompose();
        println!("{decomposed_a:?}");
        let decomposed_b = input_b.decompose();
        println!("{decomposed_b:?}");
        let do_inputs_clash = decomposed_a.clashes_with(&decomposed_b);
        println!("Clash: {do_inputs_clash}");
        do_inputs_clash
    }

    mod basic_functionality {
        use crate::{input_mocking::MockInput, prelude::ModifierKey};
        use bevy::input::InputPlugin;
        use Action::*;

        use super::*;

        #[test]
        #[ignore = "Figuring out how to handle the length of chords with group inputs is out of scope."]
        fn input_types_have_right_length() {
            let simple = KeyA.decompose();
            assert_eq!(simple.len(), 1);

            let empty_chord = ButtonlikeChord::default().decompose();
            assert_eq!(empty_chord.len(), 0);

            let chord = ButtonlikeChord::new([KeyA, KeyB, KeyC]).decompose();
            assert_eq!(chord.len(), 3);

            let modifier = ModifierKey::Control.decompose();
            assert_eq!(modifier.len(), 1);

            let modified_chord = ButtonlikeChord::modified(ModifierKey::Control, KeyA).decompose();
            assert_eq!(modified_chord.len(), 2);

            let group = KeyboardVirtualDPad::WASD.decompose();
            assert_eq!(group.len(), 1);
        }

        #[test]
        fn clash_detection() {
            let a = KeyA;
            let b = KeyB;
            let c = KeyC;
            let ab = ButtonlikeChord::new([KeyA, KeyB]);
            let bc = ButtonlikeChord::new([KeyB, KeyC]);
            let abc = ButtonlikeChord::new([KeyA, KeyB, KeyC]);
            let axyz_dpad = KeyboardVirtualDPad::new(KeyA, KeyX, KeyY, KeyZ);
            let abcd_dpad = KeyboardVirtualDPad::WASD;

            let ctrl_up = ButtonlikeChord::new([ArrowUp, ControlLeft]);
            let directions_dpad = KeyboardVirtualDPad::ARROW_KEYS;

            assert!(!inputs_clash(a, b));
            assert!(inputs_clash(a, ab.clone()));
            assert!(!inputs_clash(c, ab.clone()));
            assert!(!inputs_clash(ab.clone(), bc.clone()));
            assert!(inputs_clash(ab.clone(), abc.clone()));
            assert!(inputs_clash(axyz_dpad.clone(), a));
            assert!(inputs_clash(axyz_dpad.clone(), ab.clone()));
            assert!(!inputs_clash(axyz_dpad.clone(), bc.clone()));
            assert!(inputs_clash(axyz_dpad.clone(), abcd_dpad.clone()));
            assert!(inputs_clash(ctrl_up.clone(), directions_dpad.clone()));
        }

        #[test]
        fn button_chord_clash_construction() {
            let input_map = test_input_map();

            let observed_clash = input_map.possible_clash(&One, &OneAndTwo).unwrap();

            let correct_clash = Clash {
                action_a: One,
                action_b: OneAndTwo,
                inputs_a: vec![Box::new(Digit1)],
                inputs_b: vec![Box::new(ButtonlikeChord::new([Digit1, Digit2]))],
            };

            assert_eq!(observed_clash, correct_clash);
        }

        #[test]
        fn chord_chord_clash_construction() {
            let input_map = test_input_map();

            let observed_clash = input_map
                .possible_clash(&OneAndTwoAndThree, &OneAndTwo)
                .unwrap();
            let correct_clash = Clash {
                action_a: OneAndTwoAndThree,
                action_b: OneAndTwo,
                inputs_a: vec![Box::new(ButtonlikeChord::new([Digit1, Digit2, Digit3]))],
                inputs_b: vec![Box::new(ButtonlikeChord::new([Digit1, Digit2]))],
            };

            assert_eq!(observed_clash, correct_clash);
        }

        #[test]
        fn can_clash() {
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
        fn resolve_prioritize_longest() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);

            let input_map = test_input_map();
            let simple_clash = input_map.possible_clash(&One, &OneAndTwo).unwrap();
            app.press_input(Digit1);
            app.press_input(Digit2);
            app.update();

            assert_eq!(
                resolve_clash(
                    &simple_clash,
                    ClashStrategy::PrioritizeLongest,
                    &InputStreams::from_world(app.world(), None),
                ),
                Some(One)
            );

            let reversed_clash = input_map.possible_clash(&OneAndTwo, &One).unwrap();
            assert_eq!(
                resolve_clash(
                    &reversed_clash,
                    ClashStrategy::PrioritizeLongest,
                    &InputStreams::from_world(app.world(), None),
                ),
                Some(One)
            );

            let chord_clash = input_map
                .possible_clash(&OneAndTwo, &OneAndTwoAndThree)
                .unwrap();
            app.press_input(Digit3);
            app.update();

            let input_streams = InputStreams::from_world(app.world(), None);

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
        fn handle_clashes() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.press_input(Digit1);
            app.press_input(Digit2);
            app.update();

            let mut button_data = HashMap::new();

            button_data.insert(One, true);
            button_data.insert(Two, true);
            button_data.insert(OneAndTwo, true);

            input_map.handle_clashes(
                &mut button_data,
                &InputStreams::from_world(app.world(), None),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = HashMap::new();
            expected.insert(OneAndTwo, true);

            assert_eq!(button_data, expected);
        }

        // Checks that a clash between a VirtualDPad and a chord chooses the chord
        #[test]
        #[ignore = "Clashing inputs for non-buttonlike inputs is broken."]
        fn handle_clashes_dpad_chord() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.press_input(ControlLeft);
            app.press_input(ArrowUp);
            app.update();

            // Both the DPad and the chord are pressed,
            // because we've sent the inputs for both
            let mut button_data = HashMap::new();
            button_data.insert(CtrlUp, true);
            button_data.insert(MoveDPad, true);

            // Double-check that the two input bindings clash
            let chord_input = input_map.get_buttonlike(&CtrlUp).unwrap().first().unwrap();
            let dpad_input = input_map
                .get_dual_axislike(&MoveDPad)
                .unwrap()
                .first()
                .unwrap();

            assert!(chord_input
                .decompose()
                .clashes_with(&dpad_input.decompose()));

            // Triple check that the inputs are clashing
            input_map
                .possible_clash(&CtrlUp, &MoveDPad)
                .expect("Clash not detected");

            // Double check that the chord is longer than the DPad
            assert!(chord_input.decompose().len() > dpad_input.decompose().len());

            input_map.handle_clashes(
                &mut button_data,
                &InputStreams::from_world(app.world(), None),
                ClashStrategy::PrioritizeLongest,
            );

            // Only the chord should be pressed,
            // because it is longer than the DPad
            let mut expected = HashMap::new();
            expected.insert(CtrlUp, true);

            assert_eq!(button_data, expected);
        }

        #[test]
        fn which_pressed() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.press_input(Digit1);
            app.press_input(Digit2);
            app.press_input(ControlLeft);
            app.update();

            let action_data = input_map.process_actions(
                &InputStreams::from_world(app.world(), None),
                ClashStrategy::PrioritizeLongest,
            );

            for (action, button_pressed) in action_data.button_actions.iter() {
                if *action == CtrlOne || *action == OneAndTwo {
                    assert!(button_pressed);
                } else {
                    assert!(!button_pressed);
                }
            }
        }
    }
}
