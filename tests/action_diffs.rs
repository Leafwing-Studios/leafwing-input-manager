use bevy::{input::InputPlugin, prelude::*};
use leafwing_input_manager::action_state::ActionDiffEvent;
use leafwing_input_manager::{
    action_state::ActionDiff, axislike::DualAxisData, prelude::*, systems::generate_action_diffs,
};

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    PayTheBills,
}

#[derive(Default)]
struct Counter(pub u8);

fn spawn_da_bills(mut commands: Commands) {
    commands.spawn(ActionState::<Action>::default());
}

fn pay_da_bills(
    mutation: impl Fn(Mut<ActionState<Action>>),
) -> impl Fn(Query<&mut ActionState<Action>>, Local<Counter>) {
    move |mut action_state_query: Query<&mut ActionState<Action>>, mut counter: Local<Counter>| {
        if let Ok(mut action_state) = action_state_query.get_single_mut() {
            if !action_state.pressed(&Action::PayTheBills) {
                action_state.press(&Action::PayTheBills);
                mutation(action_state);
            } else if counter.0 > 1 {
                action_state.release(&Action::PayTheBills);
            }
            counter.0 += 1;
        }
    }
}

fn process_action_diffs<A: Actionlike>(
    mut action_state_query: Query<&mut ActionState<A>>,
    mut action_diff_events: EventReader<ActionDiffEvent<A>>,
) {
    for action_diff_event in action_diff_events.read() {
        if action_diff_event.owner.is_some() {
            let mut action_state = action_state_query.get_single_mut().unwrap();
            action_diff_event
                .action_diffs
                .iter()
                .for_each(|diff| action_state.apply_diff(diff));
        }
    }
}

fn create_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        InputManagerPlugin::<Action>::default(),
    ))
    .add_systems(Startup, spawn_da_bills)
    .add_event::<ActionDiffEvent<Action>>();
    app.update();
    app
}

fn get_events<E: Event>(app: &App) -> &Events<E> {
    app.world.resource()
}
fn get_events_mut<E: Event>(app: &mut App) -> Mut<Events<E>> {
    app.world.resource_mut()
}

fn send_action_diff(app: &mut App, action_diff: ActionDiffEvent<Action>) {
    let mut action_diff_events = get_events_mut::<ActionDiffEvent<Action>>(app);
    action_diff_events.send(action_diff);
}

fn assert_has_no_action_diffs(app: &mut App) {
    let action_diff_events = get_events::<ActionDiffEvent<Action>>(app);
    let action_diff_event_reader = &mut action_diff_events.get_reader();
    match action_diff_event_reader.read(action_diff_events).next() {
        Some(action_diff) => panic!(
            "Expected no `ActionDiff` variants. Received: {:?}",
            action_diff
        ),
        None => {}
    }
}

fn assert_action_diff_created(app: &mut App, predicate: impl Fn(&ActionDiffEvent<Action>)) {
    let mut action_diff_events = get_events_mut::<ActionDiffEvent<Action>>(app);
    let action_diff_event_reader = &mut action_diff_events.get_reader();
    assert!(action_diff_event_reader.len(action_diff_events.as_ref()) < 2);
    match action_diff_event_reader
        .read(action_diff_events.as_ref())
        .next()
    {
        Some(action_diff) => predicate(action_diff),
        None => panic!("Expected an `ActionDiff` variant. Received none."),
    };
    action_diff_events.clear();
}

fn assert_action_diff_received(app: &mut App, action_diff_event: ActionDiffEvent<Action>) {
    let mut action_state_query = app.world.query::<&ActionState<Action>>();
    let action_state = action_state_query.get_single(&app.world).unwrap();
    assert_eq!(action_diff_event.action_diffs.len(), 1);
    match action_diff_event.action_diffs.first().unwrap().clone() {
        ActionDiff::Pressed { action } => {
            assert!(action_state.pressed(&action));
            assert!(action_state.value(&action) == 1.);
        }
        ActionDiff::Released { action } => {
            assert!(action_state.released(&action));
            assert!(action_state.value(&action) == 0.);
            assert!(action_state.axis_pair(&action).is_none());
        }
        ActionDiff::ValueChanged { action, value } => {
            assert!(action_state.pressed(&action));
            assert!(action_state.value(&action) == value);
        }
        ActionDiff::AxisPairChanged { action, axis_pair } => {
            assert!(action_state.pressed(&action));
            match action_state.axis_pair(&action) {
                Some(axis_pair_data) => {
                    assert!(axis_pair_data.xy() == axis_pair);
                    assert!(action_state.value(&action) == axis_pair_data.xy().length());
                }
                None => panic!("Expected an `AxisPair` variant. Received none."),
            }
        }
    }
}

#[test]
fn generate_binary_action_diffs() {
    let mut app = create_app();
    let entity = app
        .world
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(&app.world);
    app.add_systems(
        Update,
        pay_da_bills(|mut action_state| {
            action_state.action_data_mut(&Action::PayTheBills).value = 1.;
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action>);

    app.update();
    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Pressed { action } => {
                assert_eq!(action, Action::PayTheBills);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `Pressed` variant got a `Released` variant")
            }
            ActionDiff::ValueChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `ValueChanged` variant")
            }
            ActionDiff::AxisPairChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `AxisPairChanged` variant")
            }
        }
    });

    app.update();

    assert_has_no_action_diffs(&mut app);

    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Released { action } => {
                assert_eq!(action, Action::PayTheBills);
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `Released` variant got a `Pressed` variant")
            }
            ActionDiff::ValueChanged { .. } => {
                panic!("Expected a `Released` variant got a `ValueChanged` variant")
            }
            ActionDiff::AxisPairChanged { .. } => {
                panic!("Expected a `Released` variant got a `AxisPairChanged` variant")
            }
        }
    });
}

#[test]
fn generate_value_action_diffs() {
    let input_value = 0.5;
    let mut app = create_app();
    let entity = app
        .world
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(&app.world);
    app.add_systems(
        Update,
        pay_da_bills(move |mut action_state| {
            action_state.action_data_mut(&Action::PayTheBills).value = input_value;
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action>)
    .add_event::<ActionDiffEvent<Action>>();

    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::ValueChanged { action, value } => {
                assert_eq!(action, Action::PayTheBills);
                assert_eq!(value, input_value);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `ValueChanged` variant got a `Released` variant")
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `ValueChanged` variant got a `Pressed` variant")
            }
            ActionDiff::AxisPairChanged { .. } => {
                panic!("Expected a `ValueChanged` variant got a `AxisPairChanged` variant")
            }
        }
    });

    app.update();

    assert_has_no_action_diffs(&mut app);

    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Released { action } => {
                assert_eq!(action, Action::PayTheBills);
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `Released` variant got a `Pressed` variant")
            }
            ActionDiff::ValueChanged { .. } => {
                panic!("Expected a `Released` variant got a `ValueChanged` variant")
            }
            ActionDiff::AxisPairChanged { .. } => {
                panic!("Expected a `Released` variant got a `AxisPairChanged` variant")
            }
        }
    });
}

#[test]
fn generate_axis_action_diffs() {
    let input_axis_pair = Vec2 { x: 5., y: 8. };
    let mut app = create_app();
    let entity = app
        .world
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(&app.world);
    app.add_systems(
        Update,
        pay_da_bills(move |mut action_state| {
            action_state.action_data_mut(&Action::PayTheBills).axis_pair =
                Some(DualAxisData::from_xy(input_axis_pair));
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action>)
    .add_event::<ActionDiffEvent<Action>>();

    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::AxisPairChanged { action, axis_pair } => {
                assert_eq!(action, Action::PayTheBills);
                assert_eq!(axis_pair, input_axis_pair);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `AxisPairChanged` variant got a `Released` variant")
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `AxisPairChanged` variant got a `Pressed` variant")
            }
            ActionDiff::ValueChanged { .. } => {
                panic!("Expected a `AxisPairChanged` variant got a `ValueChanged` variant")
            }
        }
    });

    app.update();

    assert_has_no_action_diffs(&mut app);

    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Released { action } => {
                assert_eq!(action, Action::PayTheBills);
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `Released` variant got a `Pressed` variant")
            }
            ActionDiff::ValueChanged { .. } => {
                panic!("Expected a `Released` variant got a `ValueChanged` variant")
            }
            ActionDiff::AxisPairChanged { .. } => {
                panic!("Expected a `Released` variant got a `AxisPairChanged` variant")
            }
        }
    });
}

#[test]
fn process_binary_action_diffs() {
    let mut app = create_app();
    let entity = app
        .world
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(&app.world);
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Pressed {
            action: Action::PayTheBills,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Released {
            action: Action::PayTheBills,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);
}

#[test]
fn process_value_action_diff() {
    let mut app = create_app();
    let entity = app
        .world
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(&app.world);
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::ValueChanged {
            action: Action::PayTheBills,
            value: 0.5,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Released {
            action: Action::PayTheBills,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);
}

#[test]
fn process_axis_action_diff() {
    let mut app = create_app();
    let entity = app
        .world
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(&app.world);
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::AxisPairChanged {
            action: Action::PayTheBills,
            axis_pair: Vec2 { x: 1., y: 0. },
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Released {
            action: Action::PayTheBills,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);
}
