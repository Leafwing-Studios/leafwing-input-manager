use bevy::ecs::system::SystemState;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::utils::HashSet;
use leafwing_input_manager::input_streams::InputStreams;
use leafwing_input_manager::prelude::*;

fn test_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_input_map);
    app
}

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
}

fn spawn_input_map(mut commands: Commands) {
    use Action::*;
    use KeyCode::*;

    let mut input_map = InputMap::default();

    input_map.insert(Key1, One);
    input_map.insert(Key2, Two);
    input_map.insert_chord([Key1, Key2], OneAndTwo);
    input_map.insert_chord([Key2, Key3], TwoAndThree);
    input_map.insert_chord([Key1, Key2, Key3], OneAndTwoAndThree);
    input_map.insert_chord([ControlLeft, Key1], CtrlOne);
    input_map.insert_chord([AltLeft, Key1], AltOne);
    input_map.insert_chord([ControlLeft, AltLeft, Key1], CtrlAltOne);

    commands.spawn(input_map);
}

trait ClashTestExt {
    /// Asserts that the set of `pressed_actions` matches the actions observed
    /// by the entity with the corresponding variant of the [`ClashStrategy`] enum
    /// in its [`InputMap`] component
    fn assert_input_map_actions_eq(
        &mut self,
        clash_strategy: ClashStrategy,
        pressed_actions: impl IntoIterator<Item = Action>,
    );
}

impl ClashTestExt for App {
    fn assert_input_map_actions_eq(
        &mut self,
        clash_strategy: ClashStrategy,
        pressed_actions: impl IntoIterator<Item = Action>,
    ) {
        let pressed_actions: HashSet<Action> = HashSet::from_iter(pressed_actions);
        // SystemState is love, SystemState is life
        let mut input_system_state: SystemState<Query<&InputMap<Action>>> =
            SystemState::new(&mut self.world);

        let input_map_query = input_system_state.get(&self.world);

        let input_map = input_map_query.single();
        let keyboard_input = self.world.resource::<Input<KeyCode>>();

        for action in Action::variants() {
            if pressed_actions.contains(&action) {
                assert!(
                    input_map.pressed(action, &InputStreams::from_world(&self.world, None), clash_strategy),
                    "{action:?} was incorrectly not pressed for {clash_strategy:?} when `Input<KeyCode>` was \n {keyboard_input:?}."
                );
            } else {
                assert!(
                    !input_map.pressed(action, &InputStreams::from_world(&self.world, None), clash_strategy),
                    "{action:?} was incorrectly pressed for {clash_strategy:?} when `Input<KeyCode>` was \n {keyboard_input:?}"
                );
            }
        }
    }
}

#[test]
fn two_inputs_clash_handling() {
    use Action::*;
    use KeyCode::*;

    let mut app = test_app();

    // Two inputs
    app.send_input(Key1);
    app.send_input(Key2);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [One, Two, OneAndTwo]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [OneAndTwo]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);
}

#[test]
fn three_inputs_clash_handling() {
    use Action::*;
    use KeyCode::*;

    let mut app = test_app();

    // Three inputs
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(Key2);
    app.send_input(Key3);
    app.update();

    app.assert_input_map_actions_eq(
        ClashStrategy::PressAll,
        [One, Two, OneAndTwo, TwoAndThree, OneAndTwoAndThree],
    );
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [OneAndTwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);
}

#[test]
fn modifier_clash_handling() {
    use Action::*;
    use KeyCode::*;

    let mut app = test_app();

    // Modifier
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(Key2);
    app.send_input(Key3);
    app.send_input(ControlLeft);
    app.update();

    app.assert_input_map_actions_eq(
        ClashStrategy::PressAll,
        [One, Two, OneAndTwo, TwoAndThree, OneAndTwoAndThree, CtrlOne],
    );
    app.assert_input_map_actions_eq(
        ClashStrategy::PrioritizeLongest,
        [CtrlOne, OneAndTwoAndThree],
    );
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);
}

#[test]
fn multiple_modifiers_clash_handling() {
    use Action::*;
    use KeyCode::*;

    let mut app = test_app();

    // Multiple modifiers
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(ControlLeft);
    app.send_input(AltLeft);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [One, CtrlOne, AltOne, CtrlAltOne]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [CtrlAltOne]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One]);
}

#[test]
fn action_order_clash_handling() {
    use Action::*;
    use KeyCode::*;

    let mut app = test_app();

    // Action order
    app.reset_inputs();
    app.send_input(Key3);
    app.send_input(Key2);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [Two, TwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [TwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [Two]);
}
