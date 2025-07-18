use bevy::{input::InputPlugin, prelude::*};
use leafwing_input_manager::action_diff::{ActionDiff, ActionDiffEvent};
use leafwing_input_manager::{
    prelude::*,
    systems::{generate_action_diffs, generate_action_diffs_filtered},
};

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    Button,
    #[actionlike(Axis)]
    Axis,
    #[actionlike(DualAxis)]
    DualAxis,
}

#[derive(Component)]
pub struct NoActionDiffs;

fn spawn_test_entity(mut commands: Commands) {
    commands.spawn(ActionState::<Action>::default());
}

fn process_action_diffs<A: Actionlike>(
    mut action_state_query: Query<&mut ActionState<A>>,
    mut action_diff_events: EventReader<ActionDiffEvent<A>>,
) {
    for action_diff_event in action_diff_events.read() {
        if action_diff_event.owner.is_some() {
            let mut action_state = action_state_query.single_mut().unwrap();
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
    .add_systems(Startup, spawn_test_entity)
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

#[track_caller]
fn assert_has_no_action_diffs(app: &mut App) {
    let action_diff_events = get_events::<ActionDiffEvent<Action>>(app);
    let action_diff_event_reader = &mut action_diff_events.get_cursor();
    if let Some(action_diff) = action_diff_event_reader.read(action_diff_events).next() {
        panic!("Expected no `ActionDiff` variants. Received: {action_diff:?}")
    }
}

#[track_caller]
fn assert_action_diff_created(app: &mut App, predicate: impl Fn(&ActionDiffEvent<Action>)) {
    let mut action_diff_events = get_events_mut::<ActionDiffEvent<Action>>(app);
    let action_diff_event_reader = &mut action_diff_events.get_cursor();
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

#[track_caller]
fn assert_action_diff_received(app: &mut App, action_diff_event: ActionDiffEvent<Action>) {
    let mut action_state_query = app.world_mut().query::<&ActionState<Action>>();
    let action_state = action_state_query.single(app.world()).unwrap();
    assert_eq!(action_diff_event.action_diffs.len(), 1);
    match action_diff_event.action_diffs.first().unwrap().clone() {
        ActionDiff::Pressed { action, .. } => {
            assert!(action_state.pressed(&action));
        }
        ActionDiff::Released { action } => {
            assert!(action_state.released(&action));
        }
        ActionDiff::AxisChanged { action, value } => {
            assert_eq!(action_state.value(&action), value);
        }
        ActionDiff::DualAxisChanged { action, axis_pair } => {
            assert_eq!(action_state.axis_pair(&action), axis_pair);
        }
        ActionDiff::TripleAxisChanged {
            action,
            axis_triple,
        } => {
            assert_eq!(action_state.axis_triple(&action), axis_triple);
        }
    }
}

#[test]
fn generate_button_action_diffs() {
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world())
        .expect("Need a Entity with ActionState");
    app.add_systems(PostUpdate, generate_action_diffs::<Action>);

    // Press
    let mut action_state = app
        .world_mut()
        .query::<&mut ActionState<Action>>()
        .get_mut(app.world_mut(), entity)
        .unwrap();
    action_state.press(&Action::Button);
    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Pressed { action, value } => {
                assert_eq!(action, Action::Button);
                assert_eq!(value, 1.0);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `Pressed` variant got a `Released` variant")
            }
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got an `AxisChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `DualAxisChanged` variant")
            }
            ActionDiff::TripleAxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `TripleAxisChanged` variant")
            }
        }
    });

    // Hold
    app.update();
    assert_has_no_action_diffs(&mut app);

    // Change value
    let mut action_state = app
        .world_mut()
        .query::<&mut ActionState<Action>>()
        .get_mut(app.world_mut(), entity)
        .unwrap();
    action_state.set_button_value(&Action::Button, 0.5);
    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Pressed { action, value } => {
                assert_eq!(action, Action::Button);
                assert_eq!(value, 0.5);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `Pressed` variant got a `Released` variant")
            }
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got an `AxisChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `DualAxisChanged` variant")
            }
            ActionDiff::TripleAxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `TripleAxisChanged` variant")
            }
        }
    });

    // Release
    let mut action_state = app
        .world_mut()
        .query::<&mut ActionState<Action>>()
        .get_mut(app.world_mut(), entity)
        .unwrap();
    action_state.release(&Action::Button);
    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Released { action } => {
                assert_eq!(action, Action::Button);
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `Released` variant got a `Pressed` variant")
            }
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Released` variant got an `AxisChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
                panic!("Expected a `Released` variant got a `DualAxisChanged` variant")
            }
            ActionDiff::TripleAxisChanged { .. } => {
                panic!("Expected a `Released` variant got a `TripleAxisChanged` variant")
            }
        }
    });
}

#[test]
fn generate_axis_action_diffs() {
    let test_axis_pair = Vec2 { x: 5., y: 8. };
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world())
        .expect("Need a Entity with ActionState");

    app.add_systems(PostUpdate, generate_action_diffs::<Action>);

    // Change axis value
    let mut action_state = app
        .world_mut()
        .query::<&mut ActionState<Action>>()
        .get_mut(app.world_mut(), entity)
        .unwrap();
    action_state.set_axis_pair(&Action::DualAxis, test_axis_pair);
    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::DualAxisChanged { action, axis_pair } => {
                assert_eq!(action, Action::DualAxis);
                assert_eq!(axis_pair, test_axis_pair);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `DualAxisChanged` variant got a `Released` variant")
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `DualAxisChanged` variant got a `Pressed` variant")
            }
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `DualAxisChanged` variant got an `AxisChanged` variant")
            }
            ActionDiff::TripleAxisChanged { .. } => {
                panic!("Expected a `DualAxisChanged` variant got a `TripleAxisChanged` variant")
            }
        }
    });

    // Do nothing for a frame
    app.update();
    assert_has_no_action_diffs(&mut app);

    // Reset axis value
    let mut action_state = app
        .world_mut()
        .query::<&mut ActionState<Action>>()
        .get_mut(app.world_mut(), entity)
        .unwrap();
    action_state.set_axis_pair(&Action::DualAxis, Vec2::ZERO);
    app.update();

    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::DualAxisChanged { action, axis_pair } => {
                assert_eq!(action, Action::DualAxis);
                assert_eq!(axis_pair, Vec2::ZERO);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `DualAxisChanged` variant got a `Released` variant")
            }
            ActionDiff::Pressed { .. } => {
                panic!("Expected a `DualAxisChanged` variant got a `Pressed` variant")
            }
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `DualAxisChanged` variant got an `AxisChanged` variant")
            }
            ActionDiff::TripleAxisChanged { .. } => {
                panic!("Expected a `DualAxisChanged` variant got a `TripleAxisChanged` variant")
            }
        }
    });
}

#[test]
fn generate_filtered_binary_action_diffs() {
    let mut app = create_app();
    app.add_systems(
        PostUpdate,
        generate_action_diffs_filtered::<Action, Without<NoActionDiffs>>,
    );

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world())
        .expect("Need a Entity with ActionState");
    // Also spawn an entity that will not send action diffs
    app.world_mut()
        .spawn((ActionState::<Action>::default(), NoActionDiffs));

    // Press both entities
    for mut action_state in app
        .world_mut()
        .query::<&mut ActionState<Action>>()
        .iter_mut(app.world_mut())
    {
        action_state.press(&Action::Button);
    }
    app.update();

    // Expect only `entity` to have an action diff event
    assert_action_diff_created(&mut app, |action_diff_event| {
        assert_eq!(action_diff_event.owner, Some(entity));
        assert_eq!(action_diff_event.action_diffs.len(), 1);
        match action_diff_event.action_diffs.first().unwrap().clone() {
            ActionDiff::Pressed { action, .. } => {
                assert_eq!(action, Action::Button);
            }
            ActionDiff::Released { .. } => {
                panic!("Expected a `Pressed` variant got a `Released` variant")
            }
            ActionDiff::AxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `ValueChanged` variant")
            }
            ActionDiff::DualAxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got an `AxisPairChanged` variant")
            }
            ActionDiff::TripleAxisChanged { .. } => {
                panic!("Expected a `Pressed` variant got a `TripleAxisChanged` variant")
            }
        }
    });
}

#[test]
fn process_binary_action_diffs() {
    let mut app = create_app();
    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world())
        .expect("Need a Entity with ActionState");
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Pressed {
            action: Action::Button,
            value: 1.0,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Released {
            action: Action::Button,
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
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world())
        .expect("Need a Entity with ActionState");
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::AxisChanged {
            action: Action::Axis,
            value: 0.5,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Released {
            action: Action::Button,
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
        .world_mut()
        .query_filtered::<Entity, With<ActionState<Action>>>()
        .single(app.world())
        .expect("Need a Entity with ActionState");
    app.add_systems(PreUpdate, process_action_diffs::<Action>);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::DualAxisChanged {
            action: Action::DualAxis,
            axis_pair: Vec2 { x: 1., y: 0. },
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);

    let action_diff_event = ActionDiffEvent {
        owner: Some(entity),
        action_diffs: vec![ActionDiff::Released {
            action: Action::Button,
        }],
    };
    send_action_diff(&mut app, action_diff_event.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff_event);
}
