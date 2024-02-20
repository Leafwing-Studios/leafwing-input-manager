//! Handles clashing inputs into a [`InputMap`] in a configurable fashion.

use crate::action_state::ActionData;
use crate::axislike::{VirtualAxis, VirtualDPad};
use crate::input_map::InputMap;
use crate::input_streams::InputStreams;
use crate::user_input::{InputKind, UserInput};
use crate::Actionlike;

use bevy::prelude::Resource;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// How should clashing inputs by handled by an [`InputMap`]?
///
/// Inputs "clash" if and only if one [`UserInput`] is a strict subset of the other.
/// By example:
///
/// - `S` and `W`: does not clash
/// - `ControlLeft + S` and `S`: clashes
/// - `S` and `S`: does not clash
/// - `ControlLeft + S` and ` AltLeft + S`: clashes
/// - `ControlLeft + S`, `AltLeft + S` and `ControlLeft + AltLeft + S`: clashes
///
/// This strategy is only used when assessing the actions and input holistically,
/// in [`InputMap::which_pressed`], using [`InputMap::handle_clashes`].
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
                VirtualAxis(other_axis) => virtual_axis_button_clash(other_axis, self_button),
            },
            Chord(self_chord) => match other {
                Single(other_button) => button_chord_clash(other_button, self_chord),
                Chord(other_chord) => chord_chord_clash(self_chord, other_chord),
                VirtualDPad(other_dpad) => dpad_chord_clash(other_dpad, self_chord),
                VirtualAxis(other_axis) => virtual_axis_chord_clash(other_axis, self_chord),
            },
            VirtualDPad(self_dpad) => match other {
                Single(other_button) => dpad_button_clash(self_dpad, other_button),
                Chord(other_chord) => dpad_chord_clash(self_dpad, other_chord),
                VirtualDPad(other_dpad) => dpad_dpad_clash(self_dpad, other_dpad),
                VirtualAxis(other_axis) => virtual_axis_dpad_clash(other_axis, self_dpad),
            },
            VirtualAxis(self_axis) => match other {
                Single(other_button) => virtual_axis_button_clash(self_axis, other_button),
                Chord(other_chord) => virtual_axis_chord_clash(self_axis, other_chord),
                VirtualDPad(other_dpad) => virtual_axis_dpad_clash(self_axis, other_dpad),
                VirtualAxis(other_axis) => virtual_axis_virtual_axis_clash(self_axis, other_axis),
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
        action_data: &mut HashMap<A, ActionData>,
        input_streams: &InputStreams,
        clash_strategy: ClashStrategy,
    ) {
        for clash in self.get_clashes(action_data, input_streams) {
            // Remove the action in the pair that was overruled, if any
            if let Some(culled_action) = resolve_clash(&clash, clash_strategy, input_streams) {
                action_data.remove(&culled_action);
            }
        }
    }

    /// Updates the cache of possible input clashes
    pub(crate) fn possible_clashes(&self) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        for action_a in self.actions() {
            for action_b in self.actions() {
                if let Some(clash) = self.possible_clash(action_a, action_b) {
                    clashes.push(clash);
                }
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
        action_data: &HashMap<A, ActionData>,
        input_streams: &InputStreams,
    ) -> Vec<Clash<A>> {
        let mut clashes = Vec::default();

        // We can limit our search to the cached set of possibly clashing actions
        for clash in self.possible_clashes() {
            let Some(data_a) = action_data.get(&clash.action_a) else {
                continue;
            };

            let Some(data_b) = action_data.get(&clash.action_b) else {
                continue;
            };

            // Clashes can only occur if both actions were triggered
            // This is not strictly necessary, but saves work
            if data_a.state.pressed() && data_b.state.pressed() {
                // Check if the potential clash occurred based on the pressed inputs
                if let Some(clash) = check_clash(&clash, input_streams) {
                    clashes.push(clash)
                }
            }
        }

        clashes
    }

    /// If the pair of actions could clash, how?
    #[must_use]
    fn possible_clash(&self, action_a: &A, action_b: &A) -> Option<Clash<A>> {
        let mut clash = Clash::new(action_a.clone(), action_b.clone());

        for input_a in self.get(action_a)? {
            for input_b in self.get(action_b)? {
                if input_a.clashes(input_b) {
                    clash.inputs_a.push(input_a.clone());
                    clash.inputs_b.push(input_b.clone());
                }
            }
        }

        let not_empty = !clash.inputs_a.is_empty();
        not_empty.then_some(clash)
    }
}

/// A user-input clash, which stores the actions that are being clashed on,
/// as well as the corresponding user inputs
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct Clash<A: Actionlike> {
    action_a: A,
    action_b: A,
    inputs_a: Vec<UserInput>,
    inputs_b: Vec<UserInput>,
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

// Does the `button` clash with the `chord`?
#[must_use]
fn button_chord_clash(button: &InputKind, chord: &[InputKind]) -> bool {
    chord.len() > 1 && chord.contains(button)
}

// Does the `dpad` clash with the `chord`?
#[must_use]
fn dpad_chord_clash(dpad: &VirtualDPad, chord: &[InputKind]) -> bool {
    chord.len() > 1
        && chord
            .iter()
            .any(|button| [dpad.up, dpad.down, dpad.left, dpad.right].contains(button))
}

fn dpad_button_clash(dpad: &VirtualDPad, button: &InputKind) -> bool {
    [dpad.up, dpad.down, dpad.left, dpad.right].contains(button)
}

fn dpad_dpad_clash(dpad1: &VirtualDPad, dpad2: &VirtualDPad) -> bool {
    [dpad1.up, dpad1.down, dpad1.left, dpad1.right]
        .into_iter()
        .any(|button| [dpad2.up, dpad2.down, dpad2.left, dpad2.right].contains(&button))
}

#[must_use]
fn virtual_axis_button_clash(axis: &VirtualAxis, button: &InputKind) -> bool {
    button == &axis.negative || button == &axis.positive
}

#[must_use]
fn virtual_axis_dpad_clash(axis: &VirtualAxis, dpad: &VirtualDPad) -> bool {
    [&dpad.up, &dpad.down, &dpad.left, &dpad.right]
        .iter()
        .any(|button| virtual_axis_button_clash(axis, button))
}

#[must_use]
fn virtual_axis_chord_clash(axis: &VirtualAxis, chord: &[InputKind]) -> bool {
    chord.len() > 1
        && chord
            .iter()
            .any(|button| virtual_axis_button_clash(axis, button))
}

#[must_use]
fn virtual_axis_virtual_axis_clash(axis1: &VirtualAxis, axis2: &VirtualAxis) -> bool {
    virtual_axis_button_clash(axis1, &axis2.negative)
        || virtual_axis_button_clash(axis1, &axis2.positive)
}

/// Does the `chord_a` clash with `chord_b`?
#[must_use]
fn chord_chord_clash(chord_a: &Vec<InputKind>, chord_b: &Vec<InputKind>) -> bool {
    if chord_a.len() <= 1 || chord_b.len() <= 1 {
        return false;
    }

    if chord_a == chord_b {
        return false;
    }

    fn is_subset(slice_a: &[InputKind], slice_b: &[InputKind]) -> bool {
        slice_a.iter().all(|a| slice_b.contains(a))
    }

    is_subset(chord_a, chord_b) || is_subset(chord_b, chord_a)
}

/// Given the `input_streams`, does the provided clash actually occur?
///
/// Returns `Some(clash)` if they are clashing, and `None` if they are not.
#[must_use]
fn check_clash<A: Actionlike>(clash: &Clash<A>, input_streams: &InputStreams) -> Option<Clash<A>> {
    let mut actual_clash: Clash<A> = clash.clone();

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

    let not_empty = !clash.inputs_a.is_empty();
    not_empty.then_some(actual_clash)
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
                Ordering::Greater => Some(clash.action_b.clone()),
                Ordering::Less => Some(clash.action_a.clone()),
                Ordering::Equal => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as leafwing_input_manager;
    use bevy::app::App;
    use bevy::input::keyboard::KeyCode::*;
    use bevy::prelude::Reflect;
    use leafwing_input_manager_macros::Actionlike;

    #[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
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

        input_map.insert(One, Digit1);
        input_map.insert(Two, Digit2);
        input_map.insert_chord(OneAndTwo, [Digit1, Digit2]);
        input_map.insert_chord(TwoAndThree, [Digit2, Digit3]);
        input_map.insert_chord(OneAndTwoAndThree, [Digit1, Digit2, Digit3]);
        input_map.insert_chord(CtrlOne, [ControlLeft, Digit1]);
        input_map.insert_chord(AltOne, [AltLeft, Digit1]);
        input_map.insert_chord(CtrlAltOne, [ControlLeft, AltLeft, Digit1]);
        input_map.insert(
            MoveDPad,
            VirtualDPad {
                up: ArrowUp.into(),
                down: ArrowDown.into(),
                left: ArrowLeft.into(),
                right: ArrowRight.into(),
            },
        );
        input_map.insert_chord(CtrlUp, [ControlLeft, ArrowUp]);

        input_map
    }

    mod basic_functionality {
        use crate::axislike::VirtualDPad;
        use crate::input_mocking::MockInput;
        use bevy::input::InputPlugin;
        use Action::*;

        use super::*;

        #[test]
        fn clash_detection() {
            let a: UserInput = KeyA.into();
            let b: UserInput = KeyB.into();
            let c: UserInput = KeyC.into();
            let ab = UserInput::chord([KeyA, KeyB]);
            let bc = UserInput::chord([KeyB, KeyC]);
            let abc = UserInput::chord([KeyA, KeyB, KeyC]);
            let axyz_dpad: UserInput = VirtualDPad {
                up: KeyA.into(),
                down: KeyX.into(),
                left: KeyY.into(),
                right: KeyZ.into(),
            }
            .into();
            let abcd_dpad: UserInput = VirtualDPad {
                up: KeyA.into(),
                down: KeyB.into(),
                left: KeyC.into(),
                right: KeyD.into(),
            }
            .into();

            let ctrl_up: UserInput = UserInput::chord([ArrowUp, ControlLeft]);
            let directions_dpad: UserInput = VirtualDPad {
                up: ArrowUp.into(),
                down: ArrowDown.into(),
                left: ArrowLeft.into(),
                right: ArrowRight.into(),
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

            let observed_clash = input_map.possible_clash(&One, &OneAndTwo).unwrap();
            let correct_clash = Clash {
                action_a: One,
                action_b: OneAndTwo,
                inputs_a: vec![Digit1.into()],
                inputs_b: vec![UserInput::chord([Digit1, Digit2])],
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
                inputs_a: vec![UserInput::chord([Digit1, Digit2, Digit3])],
                inputs_b: vec![UserInput::chord([Digit1, Digit2])],
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
            app.send_input(Digit1);
            app.send_input(Digit2);
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

            let reversed_clash = input_map.possible_clash(&OneAndTwo, &One).unwrap();
            assert_eq!(
                resolve_clash(
                    &reversed_clash,
                    ClashStrategy::PrioritizeLongest,
                    &input_streams,
                ),
                Some(One)
            );

            let chord_clash = input_map
                .possible_clash(&OneAndTwo, &OneAndTwoAndThree)
                .unwrap();
            app.send_input(Digit3);
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
        fn handle_clashes() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.send_input(Digit1);
            app.send_input(Digit2);
            app.update();

            let mut action_data = HashMap::new();
            let mut action_datum = ActionData::default();
            action_datum.state.press();

            action_data.insert(One, action_datum.clone());
            action_data.insert(Two, action_datum.clone());
            action_data.insert(OneAndTwo, action_datum.clone());

            input_map.handle_clashes(
                &mut action_data,
                &InputStreams::from_world(&app.world, None),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = HashMap::new();
            expected.insert(OneAndTwo, action_datum.clone());

            assert_eq!(action_data, expected);
        }

        // Checks that a clash between a VirtualDPad and a chord choses the chord
        #[test]
        fn handle_clashes_dpad_chord() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.send_input(ControlLeft);
            app.send_input(ArrowUp);
            app.update();

            let mut action_data = HashMap::new();
            let mut action_datum = ActionData::default();
            action_datum.state.press();
            action_data.insert(CtrlUp, action_datum.clone());
            action_data.insert(MoveDPad, action_datum.clone());

            input_map.handle_clashes(
                &mut action_data,
                &InputStreams::from_world(&app.world, None),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = HashMap::new();
            expected.insert(CtrlUp, action_datum);

            assert_eq!(action_data, expected);
        }

        #[test]
        fn which_pressed() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.send_input(Digit1);
            app.send_input(Digit2);
            app.send_input(ControlLeft);
            app.update();

            let action_data = input_map.which_pressed(
                &InputStreams::from_world(&app.world, None),
                ClashStrategy::PrioritizeLongest,
            );

            for (action, action_data) in action_data.iter() {
                if *action == CtrlOne || *action == OneAndTwo {
                    assert!(action_data.state.pressed());
                } else {
                    assert!(action_data.state.released());
                }
            }
        }
    }
}
