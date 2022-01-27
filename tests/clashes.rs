use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::utils::HashSet;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputStreams;
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

fn test_input_map(strategy: ClashStrategy) -> InputMap<Action> {
    use Action::*;
    use KeyCode::*;

    let mut input_map = InputMap::default();
    input_map.clash_strategy = strategy;

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

fn spawn_input_maps(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: test_input_map(ClashStrategy::PressAll),
            ..Default::default()
        });

    commands
        .spawn()
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: test_input_map(ClashStrategy::PrioritizeLongest),
            ..Default::default()
        });

    commands
        .spawn()
        .insert_bundle(InputManagerBundle::<Action> {
            input_map: test_input_map(ClashStrategy::UseActionOrder),
            ..Default::default()
        });
}

trait ClashTestExt {
    /// Asserts that the set of `pressed_actions` matches the actions observed
    /// by the entity with the corresponding varaint of the [`ClashStrategy`] enum
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
        let pressed_actions: HashSet<Action> = HashSet::from_iter(pressed_actions.into_iter());
        // SystemState is love, SystemState is life
        let mut input_system_state: SystemState<(Query<&InputMap<Action>>, Res<Input<KeyCode>>)> =
            SystemState::new(&mut self.world);

        let (input_map_query, keyboard) = input_system_state.get(&self.world);

        let input_streams = InputStreams::from_keyboard(&*keyboard);

        let mut matching_input_map = InputMap::<Action>::default();
        let mut found = false;

        for input_map in input_map_query.iter() {
            if input_map.clash_strategy == clash_strategy {
                matching_input_map = input_map.clone();
                found = true;
                break;
            }
        }

        // Verify that we found the right input map
        assert!(found);

        let keyboard_input = input_streams.keyboard.unwrap();

        for action in Action::iter() {
            if pressed_actions.contains(&action) {
                assert!(
                    matching_input_map.pressed(action, &input_streams),
                    "{action:?} was incorrectly not pressed for {clash_strategy:?} when `Input<KeyCode>` was \n {keyboard_input:?}."
                );
            } else {
                assert!(
                    !matching_input_map.pressed(action, &input_streams),
                    "{action:?} was incorrectly pressed for {clash_strategy:?} when `Input<KeyCode>` was \n {keyboard_input:?}"
                );
            }
        }

        // Verify that the holistic view is also correct
        assert_eq!(
            matching_input_map.which_pressed(&input_streams),
            pressed_actions
        );
    }
}

#[test]
fn input_clash_handling() {
    use bevy::input::InputPlugin;
    use leafwing_input_manager::MockInput;
    use Action::*;
    use KeyCode::*;

    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(spawn_input_maps);

    // Two inputs
    app.send_input(Key1);
    app.send_input(Key2);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [One, Two, OneAndTwo]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [OneAndTwo]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One, Two]);

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

    // Modifier
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(Key2);
    app.send_input(Key3);
    app.send_input(LControl);
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

    // Multiple modifiers
    app.reset_inputs();
    app.send_input(Key1);
    app.send_input(LControl);
    app.send_input(LAlt);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [One, CtrlOne, AltOne, CtrlAltOne]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [CtrlAltOne]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [One]);

    // Action order
    app.reset_inputs();
    app.send_input(Key3);
    app.send_input(Key2);
    app.update();

    app.assert_input_map_actions_eq(ClashStrategy::PressAll, [Two, TwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::PrioritizeLongest, [TwoAndThree]);
    app.assert_input_map_actions_eq(ClashStrategy::UseActionOrder, [Two]);
}
