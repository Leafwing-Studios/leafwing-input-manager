//! Handles clashing inputs into a [`InputMap`] in a configurable fashion.

use std::cmp::Ordering;

use bevy::prelude::Resource;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};

use crate::action_state::ActionData;
use crate::input_map::InputMap;
use crate::input_streams::InputStreams;
use crate::user_input::UserInput;
use crate::Actionlike;

/// How should clashing inputs by handled by an [`InputMap`]?
///
/// Inputs "clash" if and only if one [`UserInput`] is a strict subset of the other.
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

/// Checks if the given two [`UserInput`]s clash with each other.
#[must_use]
#[inline]
fn check_clashed(lhs: &dyn UserInput, rhs: &dyn UserInput) -> bool {
    #[inline]
    fn clash(list_a: &[Box<dyn UserInput>], list_b: &[Box<dyn UserInput>]) -> bool {
        // Checks if the length is greater than one,
        // as simple, atomic input are able to trigger multiple actions.
        list_a.len() > 1 && list_b.iter().all(|a| list_a.contains(a))
    }

    let lhs_inners = lhs.destructure();
    let rhs_inners = rhs.destructure();

    clash(&lhs_inners, &rhs_inners) || clash(&rhs_inners, &lhs_inners)
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
            let pressed = |action: &A| -> bool {
                matches!(action_data.get(action), Some(data) if data.state.pressed())
            };

            // Clashes can only occur if both actions were triggered
            // This is not strictly necessary, but saves work
            if pressed(&clash.action_a) && pressed(&clash.action_b) {
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
                if check_clashed(input_a.as_ref(), input_b.as_ref()) {
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
    inputs_a: Vec<Box<dyn UserInput>>,
    inputs_b: Vec<Box<dyn UserInput>>,
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
            // If a clash was detected,
            if check_clashed(input_a.as_ref(), input_b.as_ref()) {
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
    let reasons_a_is_pressed: Vec<Box<dyn UserInput>> = clash
        .inputs_a
        .iter()
        .filter(|&input| input.pressed(input_streams))
        .cloned()
        .collect();

    let reasons_b_is_pressed: Vec<Box<dyn UserInput>> = clash
        .inputs_b
        .iter()
        .filter(|&input| input.pressed(input_streams))
        .cloned()
        .collect();

    // Clashes are spurious if the actions are pressed for any non-clashing reason
    for reason_a in reasons_a_is_pressed.iter() {
        for reason_b in reasons_b_is_pressed.iter() {
            // If there is at least one non-clashing reason why these buttons should both be pressed,
            // we can avoid resolving the clash completely
            if !check_clashed(reason_a.as_ref(), reason_b.as_ref()) {
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
                .map(|input| input.destructure().len())
                .reduce(|a, b| a.max(b))
                .unwrap_or_default();

            let longest_b: usize = reasons_b_is_pressed
                .iter()
                .map(|input| input.destructure().len())
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
    use leafwing_input_manager_macros::Actionlike;

    use super::*;
    use crate as leafwing_input_manager;
    use crate::prelude::KeyboardVirtualDPad;
    use crate::user_input::InputChord;

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
        input_map.insert(OneAndTwo, InputChord::from_multiple([Digit1, Digit2]));
        input_map.insert(TwoAndThree, InputChord::from_multiple([Digit2, Digit3]));
        input_map.insert(
            OneAndTwoAndThree,
            InputChord::from_multiple([Digit1, Digit2, Digit3]),
        );
        input_map.insert(CtrlOne, InputChord::from_multiple([ControlLeft, Digit1]));
        input_map.insert(AltOne, InputChord::from_multiple([AltLeft, Digit1]));
        input_map.insert(
            CtrlAltOne,
            InputChord::from_multiple([ControlLeft, AltLeft, Digit1]),
        );
        input_map.insert(MoveDPad, KeyboardVirtualDPad::ARROW_KEYS);
        input_map.insert(CtrlUp, InputChord::from_multiple([ControlLeft, ArrowUp]));

        input_map
    }

    fn test_input_clash<A: UserInput, B: UserInput>(input_a: A, input_b: B) -> bool {
        let input_a: Box<dyn UserInput> = Box::new(input_a);
        let input_b: Box<dyn UserInput> = Box::new(input_b);
        check_clashed(input_a.as_ref(), input_b.as_ref())
    }

    mod basic_functionality {
        use crate::input_mocking::MockInput;
        use bevy::input::InputPlugin;
        use Action::*;

        use super::*;

        #[test]
        fn clash_detection() {
            let a = KeyA;
            let b = KeyB;
            let c = KeyC;
            let ab = InputChord::from_multiple([KeyA, KeyB]);
            let bc = InputChord::from_multiple([KeyB, KeyC]);
            let abc = InputChord::from_multiple([KeyA, KeyB, KeyC]);
            let axyz_dpad = KeyboardVirtualDPad::new(KeyA, KeyX, KeyY, KeyZ);
            let abcd_dpad = KeyboardVirtualDPad::WASD;

            let ctrl_up = InputChord::from_multiple([ArrowUp, ControlLeft]);
            let directions_dpad = KeyboardVirtualDPad::ARROW_KEYS;

            assert!(!test_input_clash(a, b));
            assert!(test_input_clash(a, ab.clone()));
            assert!(!test_input_clash(c, ab.clone()));
            assert!(!test_input_clash(ab.clone(), bc.clone()));
            assert!(test_input_clash(ab.clone(), abc.clone()));
            assert!(test_input_clash(axyz_dpad.clone(), a));
            assert!(test_input_clash(axyz_dpad.clone(), ab.clone()));
            assert!(!test_input_clash(axyz_dpad.clone(), bc.clone()));
            assert!(test_input_clash(axyz_dpad.clone(), abcd_dpad.clone()));
            assert!(test_input_clash(ctrl_up.clone(), directions_dpad.clone()));
        }

        #[test]
        fn button_chord_clash_construction() {
            let input_map = test_input_map();

            let observed_clash = input_map.possible_clash(&One, &OneAndTwo).unwrap();

            let correct_clash = Clash {
                action_a: One,
                action_b: OneAndTwo,
                inputs_a: vec![Box::new(Digit1)],
                inputs_b: vec![Box::new(InputChord::from_multiple([Digit1, Digit2]))],
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
                inputs_a: vec![Box::new(InputChord::from_multiple([
                    Digit1, Digit2, Digit3,
                ]))],
                inputs_b: vec![Box::new(InputChord::from_multiple([Digit1, Digit2]))],
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
            app.press_input(Digit3);
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

            app.press_input(Digit1);
            app.press_input(Digit2);
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

        // Checks that a clash between a VirtualDPad and a chord chooses the chord
        #[test]
        fn handle_clashes_dpad_chord() {
            let mut app = App::new();
            app.add_plugins(InputPlugin);
            let input_map = test_input_map();

            app.press_input(ControlLeft);
            app.press_input(ArrowUp);
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

            app.press_input(Digit1);
            app.press_input(Digit2);
            app.press_input(ControlLeft);
            app.update();

            let action_data = input_map.process_actions(
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
