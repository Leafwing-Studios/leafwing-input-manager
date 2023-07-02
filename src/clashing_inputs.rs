//! Handles clashing inputs into a [`InputMap`](crate::input_map::InputMap) in a configurable fashion.

use crate::action_state::ActionData;
use crate::input_like::InputLikeObject;
use crate::input_map::InputMap;
use crate::Actionlike;

use crate::input_streams::InputStreams;
use bevy::prelude::Resource;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Debug;
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
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum ClashStrategy {
    /// All matching inputs will always be pressed
    PressAll,
    /// Only press the action that corresponds to the longest chord
    ///
    /// This is the default strategy.
    #[default]
    PrioritizeLongest,
    /// Use the order in which actions are defined in the enum to resolve clashing inputs
    ///
    /// Uses the iteration order returned by [`Actionlike::variants()`],
    /// which is generated in order of the enum items by the `#[derive(Actionlike)]` macro.
    UseActionOrder,
}

impl ClashStrategy {
    /// Returns the list of all possible clash strategies.
    pub fn variants() -> &'static [ClashStrategy] {
        use ClashStrategy::*;

        &[PressAll, PrioritizeLongest, UseActionOrder]
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
    fn possible_clash(&self, action_a: A, action_b: A) -> Option<Clash<A>> {
        let mut clash = Clash::new(action_a.clone(), action_b.clone());

        for input_a in self.get(action_a).iter() {
            for input_b in self.get(action_b.clone()).iter() {
                if input_a.clashes(input_b.as_ref()) {
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
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Clash<A: Actionlike> {
    /// The `Actionlike::index` value corresponding to `action_a`
    index_a: usize,
    /// The `Actionlike::index` value corresponding to `action_b`
    index_b: usize,
    inputs_a: Vec<Box<dyn InputLikeObject>>,
    inputs_b: Vec<Box<dyn InputLikeObject>>,
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
        .filter(|&input| input_streams.input_pressed(input.as_ref()))
    {
        // For all inputs that were actually pressed that match action B
        for input_b in clash
            .inputs_b
            .iter()
            .filter(|&input| input_streams.input_pressed(input.as_ref()))
        {
            // If a clash was detected,
            if input_a.clashes(input_b.as_ref()) {
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
    let reasons_a_is_pressed: Vec<&Box<dyn InputLikeObject>> = clash
        .inputs_a
        .iter()
        .filter(|&input| input_streams.input_pressed(input.as_ref()))
        .collect();

    let reasons_b_is_pressed: Vec<&Box<dyn InputLikeObject>> = clash
        .inputs_b
        .iter()
        .filter(|&input| input_streams.input_pressed(input.as_ref()))
        .collect();

    // Clashes are spurious if the actions are pressed for any non-clashing reason
    for reason_a in reasons_a_is_pressed.iter() {
        for reason_b in reasons_b_is_pressed.iter() {
            // If there is at least one non-clashing reason why these buttons should both be pressed,
            // we can avoid resolving the clash completely
            if !reason_a.clashes(reason_b.as_ref()) {
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
    use crate::input_like::virtual_dpad::VirtualDPad;
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
        use crate::buttonlike::ButtonState;
        use crate::input_like::chords::Chord;
        use crate::input_like::virtual_dpad::VirtualDPad;
        use bevy::input::{Input, InputPlugin};
        use bevy::prelude::KeyCode;
        use Action::*;

        use super::*;

        #[test]
        fn clash_detection() {
            let ab = Chord::new([A, B]);
            let bc = Chord::new([B, C]);
            let abc = Chord::new([A, B, C]);
            let axyz_dpad = VirtualDPad {
                up: A.into(),
                down: X.into(),
                left: Y.into(),
                right: Z.into(),
            };
            let abcd_dpad = VirtualDPad {
                up: A.into(),
                down: B.into(),
                left: C.into(),
                right: D.into(),
            };

            let ctrl_up = Chord::new([Up, LControl]);
            let directions_dpad = VirtualDPad {
                up: Up.into(),
                down: Down.into(),
                left: Left.into(),
                right: Right.into(),
            };

            assert!(!A.clashes(&B));
            assert!(A.clashes(ab.as_ref()));
            assert!(!C.clashes(ab.as_ref()));
            assert!(!ab.clashes(bc.as_ref()));
            assert!(ab.clashes(abc.as_ref()));
            // VirtualDPads are considered single inputs so they don't clash with other single inputs
            assert!(!axyz_dpad.clashes(&A));
            assert!(axyz_dpad.clashes(ab.as_ref()));
            assert!(!axyz_dpad.clashes(bc.as_ref()));
            assert!(!axyz_dpad.clashes(&abcd_dpad));
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
                inputs_b: vec![Chord::new([Key1, Key2])],
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
                inputs_a: vec![Chord::new([Key1, Key2, Key3]).into()],
                inputs_b: vec![Chord::new([Key1, Key2]).into()],
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
            input_map.insert(Chord::new([LControl, LAlt, Key1]), Action::Two);
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
            app.world.resource_mut::<Input<KeyCode>>().press(Key1);
            app.world.resource_mut::<Input<KeyCode>>().press(Key2);
            app.update();

            let input_streams = InputStreams::from_world(&app.world);

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
            app.world.resource_mut::<Input<KeyCode>>().press(Key3);
            app.update();

            let input_streams = InputStreams::from_world(&app.world);

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
            app.world.resource_mut::<Input<KeyCode>>().press(Key1);
            app.world.resource_mut::<Input<KeyCode>>().press(LControl);
            app.update();

            let input_streams = InputStreams::from_world(&app.world);

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

            app.world.resource_mut::<Input<KeyCode>>().press(Key1);
            app.world.resource_mut::<Input<KeyCode>>().press(Key2);
            app.update();

            let mut action_data = vec![ActionData::default(); Action::n_variants()];
            action_data[One.index()].state = ButtonState::JustPressed;
            action_data[Two.index()].state = ButtonState::JustPressed;
            action_data[OneAndTwo.index()].state = ButtonState::JustPressed;

            input_map.handle_clashes(
                &mut action_data,
                &InputStreams::from_world(&app.world),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = vec![ActionData::default(); Action::n_variants()];
            expected[OneAndTwo.index()].state = ButtonState::JustPressed;

            assert_eq!(action_data, expected);
        }

        // Checks that a clash between a VirtualDPad and a chord choses the chord
        #[test]
        fn handle_clashes_dpad_chord() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);
            let input_map = test_input_map();

            app.world.resource_mut::<Input<KeyCode>>().press(LControl);
            app.world.resource_mut::<Input<KeyCode>>().press(Up);
            app.update();

            let mut action_data = vec![ActionData::default(); Action::n_variants()];
            action_data[MoveDPad.index()].state = ButtonState::JustPressed;
            action_data[CtrlUp.index()].state = ButtonState::JustPressed;

            input_map.handle_clashes(
                &mut action_data,
                &InputStreams::from_world(&app.world),
                ClashStrategy::PrioritizeLongest,
            );

            let mut expected = vec![ActionData::default(); Action::n_variants()];
            expected[CtrlUp.index()].state = ButtonState::JustPressed;

            assert_eq!(action_data, expected);
        }

        #[test]
        fn which_pressed() {
            let mut app = App::new();
            app.add_plugin(InputPlugin);
            let input_map = test_input_map();

            app.world.resource_mut::<Input<KeyCode>>().press(Key1);
            app.world.resource_mut::<Input<KeyCode>>().press(Key2);
            app.world.resource_mut::<Input<KeyCode>>().press(LControl);
            app.update();

            let action_data = input_map.which_pressed(
                &InputStreams::from_world(&app.world),
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

pub trait Clashes {
    #[must_use]
    fn clashes(&self, other: &dyn InputLikeObject) -> bool;
}

impl<T: ?Sized + InputLikeObject> Clashes for T {
    /// Does `self` clash with `other`?
    ///
    /// Inputs "clash" if and only if one [`InputLikeObject`] is a strict subset of the other.
    /// By example:
    ///
    /// - `S` and `W`: does not clash
    /// - `LControl + S` and `S`: clashes
    /// - `S` and `S`: does not clash
    /// - `LControl + S` and ` LAlt + S`: clashes
    /// - `LControl + S`, `LAlt + S` and `LControl + LAlt + S`: clashes
    /// - `VirtualDPad::arrow_keys()` and `LControl + Up`:  clashes
    ///
    /// ```
    /// # use bevy::input::prelude::*;
    /// # use leafwing_input_manager::clashing_inputs::Clashes;
    /// # use leafwing_input_manager::prelude::{Chord, VirtualDPad};
    /// # use bevy::input::keyboard::KeyCode::*;
    /// assert!(!S.clashes(&W));
    /// assert!(Chord::new([LControl, S]).clashes(&S));
    /// assert!(!S.clashes(&S));
    /// assert!(Chord::new([Chord::new([LControl, S]), Chord::new([LAlt, S])]).clashes(Chord::new([LAlt, S]).as_ref()));
    /// assert!(Chord::new([LControl, S]).clashes(Chord::new([LControl, LAlt, S]).as_ref()));
    /// assert!(Chord::new([LAlt, S]).clashes(Chord::new([LControl, LAlt, S]).as_ref()));
    /// assert!(VirtualDPad::arrow_keys().clashes(Chord::new([LControl, Up]).as_ref()));
    /// ```
    fn clashes(&self, other: &dyn InputLikeObject) -> bool {
        // Single inputs don't clash with other single inputs
        if self.len() <= 1 && other.len() <= 1 {
            return false;
        }

        // If the inputs are equal, they aren't a _strict_ subset so they don't clash.
        if self
            .as_reflect()
            .reflect_partial_eq(other.as_reflect())
            .unwrap_or_default()
        {
            return false;
        }

        // Check subsets in both directions since [A, B, C] is not a subset of [A, B], but [A, B]
        // is a subset of [A, B, C] and should still clash.
        self.is_strict_subset(other.clone_dyn()) || other.is_strict_subset(self.clone_dyn())
    }
}
