use bevy::{input::InputPlugin, prelude::*};
use leafwing_input_manager::action_diff::{ActionDiff, ActionDiffEvent};
use leafwing_input_manager::{prelude::*, systems::generate_action_diffs};

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
    app.world().resource()
}
fn get_events_mut<E: Event>(app: &mut App) -> Mut<Events<E>> {
    app.world_mut().resource_mut()
}

fn send_action_diff(app: &mut App, action_diff: ActionDiffEvent<Action>) {
    let mut action_diff_events = get_events_mut::<ActionDiffEvent<Action>>(app);
    action_diff_events.send(action_diff);
}

fn assert_has_no_action_diffs(app: &mut App) {
    let action_diff_events = get_events::<ActionDiffEvent<Action>>(app);
    let action_diff_event_reader = &mut action_diff_events.get_reader();
    if let Some(action_diff) = action_diff_event_reader.read(action_diff_events).next() {
        panic!(
            "Expected no `ActionDiff` variants. Received: {:?}",
            action_diff
        )
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
    let mut action_state_query = app.world_mut().query::<&ActionState<Action>>();
    let action_state = action_state_query.get_single(app.world()).unwrap();
    assert_eq!(action_diff_event.action_diffs.len(), 1);
    match action_diff_event.action_diffs.first().unwrap().clone() {
        ActionDiff::Pressed { action } => {
            assert!(action_state.pressed(&action));
            assert_eq!(action_state.value(&action), 1.);
        }
        ActionDiff::Released { action } => {
            assert!(action_state.released(&action));
            assert_eq!(action_state.value(&action), 0.);
            assert_eq!(action_state.axis_pair(&action), Vec2::ZERO);
        }
        ActionDiff::AxisChanged { action, value } => {
            assert!(action_state.pressed(&action));
            assert_eq!(action_state.value(&action), value);
        }
        ActionDiff::DualAxisChanged { action, axis_pair } => {
            let axis_pair_data = action_state.axis_pair(&action);
            assert!(action_state.pressed(&action));
            assert_eq!(axis_pair_data.xy(), axis_pair);
            assert_eq!(action_state.value(&action), axis_pair_data.xy().length());
        }
    }
}

#[test]
#[ignore = "ActionDiff support has been temporarily removed."]
fn generate_binary_action_diffs() {
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world());
    app.add_systems(
        Update,
        pay_da_bills(|mut action_state| {
            action_state.press(&Action::PayTheBills);
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
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `ValueChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
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
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Released` variant got a `ValueChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
                panic!("Expected a `Released` variant got a `AxisPairChanged` variant")
            }
        }
    });
}

#[test]
#[ignore = "ActionDiff support has been temporarily removed."]
fn generate_axis_action_diffs() {
    let input_axis_pair = Vec2 { x: 5., y: 8. };
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world());
    app.add_systems(
        Update,
        pay_da_bills(move |mut action_state| {
            action_state
                .dual_axis_data_mut(&Action::PayTheBills)
                .unwrap()
                .pair = input_axis_pair;
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action>)
    .add_event::<ActionDiffEvent<Action>>();

    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::DualAxisChanged { action, axis_pair } => {
                assert_eq!(action, Action::PayTheBills);
                assert_eq!(axis_pair, input_axis_pair);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `AxisPairChanged` variant got a `Released` variant")
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `AxisPairChanged` variant got a `Pressed` variant")
            }
            ActionDiff::AxisChanged { .. } => {
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
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Released` variant got a `ValueChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
                panic!("Expected a `Released` variant got a `AxisPairChanged` variant")
            }
        }
    });
}

#[test]
#[ignore = "ActionDiff support has been temporarily removed."]
fn process_binary_action_diffs() {
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world());
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
#[ignore = "ActionDiff support has been temporarily removed."]
fn process_value_action_diff() {
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world());
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::AxisChanged {
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
#[ignore = "ActionDiff support has been temporarily removed."]
fn process_axis_action_diff() {
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world());
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::DualAxisChanged {
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
