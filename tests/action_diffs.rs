use bevy::{input::InputPlugin, prelude::*};
use leafwing_input_manager::{
    action_state::ActionDiff,
    axislike::DualAxisData,
    prelude::*,
    systems::{generate_action_diffs, process_action_diffs},
};

#[derive(Actionlike, Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
enum Action {
    PayTheBills,
}

#[derive(Clone, Copy, Component, Debug, Reflect, PartialEq, Eq, Hash)]
struct BankAccountId(u32);

#[derive(Default)]
struct Counter(pub u8);

fn spawn_da_bills(mut commands: Commands) {
    commands.spawn((BankAccountId(1337), ActionState::<Action>::default()));
}

fn pay_da_bills(
    mutation: impl Fn(Mut<ActionState<Action>>) -> (),
) -> impl Fn(Query<&mut ActionState<Action>>, Local<Counter>) -> () {
    move |mut action_state_query: Query<&mut ActionState<Action>>, mut counter: Local<Counter>| {
        if let Ok(mut action_state) = action_state_query.get_single_mut() {
            if !action_state.pressed(Action::PayTheBills) {
                action_state.press(Action::PayTheBills);
                mutation(action_state);
            } else if counter.0 > 1 {
                action_state.release(Action::PayTheBills);
            }
            counter.0 += 1;
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
    .add_event::<ActionDiff<Action, BankAccountId>>();
    app
}

fn get_events<E: Event>(app: &App) -> &Events<E> {
    app.world.resource()
}
fn get_events_mut<E: Event>(app: &mut App) -> Mut<Events<E>> {
    app.world.resource_mut()
}

fn send_action_diff(app: &mut App, action_diff: ActionDiff<Action, BankAccountId>) {
    let mut action_diff_events = get_events_mut::<ActionDiff<Action, BankAccountId>>(app);
    action_diff_events.send(action_diff);
}

fn assert_has_no_action_diffs(app: &mut App) {
    let action_diff_events = get_events::<ActionDiff<Action, BankAccountId>>(app);
    let action_diff_event_reader = &mut action_diff_events.get_reader();
    match action_diff_event_reader.read(action_diff_events).next() {
        Some(action_diff) => panic!(
            "Expected no `ActionDiff` variants. Received: {:?}",
            action_diff
        ),
        None => {}
    }
}

fn assert_action_diff_created(
    app: &mut App,
    predicate: impl Fn(&ActionDiff<Action, BankAccountId>),
) {
    let mut action_diff_events = get_events_mut::<ActionDiff<Action, BankAccountId>>(app);
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

fn assert_action_diff_received(app: &mut App, action_diff: ActionDiff<Action, BankAccountId>) {
    let mut action_state_query = app.world.query::<&ActionState<Action>>();
    let action_state = action_state_query.get_single(&app.world).unwrap();
    match action_diff {
        ActionDiff::Pressed { id: _, action } => {
            assert!(action_state.pressed(action));
            assert!(action_state.value(action) == 1.);
        }
        ActionDiff::Released { id: _, action } => {
            assert!(action_state.released(action));
            assert!(action_state.value(action) == 0.);
            assert!(action_state.axis_pair(action).is_none());
        }
        ActionDiff::ValueChanged {
            id: _,
            action,
            value,
        } => {
            assert!(action_state.pressed(action));
            assert!(action_state.value(action) == value);
        }
        ActionDiff::AxisPairChanged {
            id: _,
            action,
            axis_pair,
        } => {
            assert!(action_state.pressed(action));
            match action_state.axis_pair(action) {
                Some(axis_pair_data) => {
                    assert!(axis_pair_data.xy() == axis_pair);
                    assert!(action_state.value(action) == axis_pair_data.xy().length());
                }
                None => panic!("Expected an `AxisPair` variant. Received none."),
            }
        }
    }
}

#[test]
fn generate_binary_action_diffs() {
    let mut app = create_app();
    app.add_systems(
        Update,
        pay_da_bills(|mut action_state| {
            action_state.action_data_mut(Action::PayTheBills).value = 1.;
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action, BankAccountId>);

    app.update();
    assert_action_diff_created(&mut app, |action_diff| match action_diff {
        ActionDiff::Pressed { id, action } => {
            assert!(*id == BankAccountId(1337));
            assert!(*action == Action::PayTheBills);
        }
        ActionDiff::Released { action: _, id: _ } => {
            panic!("Expected a `Pressed` variant got a `Released` variant")
        }
        ActionDiff::ValueChanged {
            action: _,
            id: _,
            value: _,
        } => panic!("Expected a `Pressed` variant got a `ValueChanged` variant"),
        ActionDiff::AxisPairChanged {
            action: _,
            id: _,
            axis_pair: _,
        } => panic!("Expected a `Pressed` variant got a `AxisPairChanged` variant"),
    });

    app.update();

    assert_has_no_action_diffs(&mut app);

    app.update();

    assert_action_diff_created(&mut app, |action_diff| match action_diff {
        ActionDiff::Released { id, action } => {
            assert!(*id == BankAccountId(1337));
            assert!(*action == Action::PayTheBills);
        }
        ActionDiff::Pressed { action: _, id: _ } => {
            panic!("Expected a `Released` variant got a `Pressed` variant")
        }
        ActionDiff::ValueChanged {
            action: _,
            id: _,
            value: _,
        } => panic!("Expected a `Released` variant got a `ValueChanged` variant"),
        ActionDiff::AxisPairChanged {
            action: _,
            id: _,
            axis_pair: _,
        } => panic!("Expected a `Released` variant got a `AxisPairChanged` variant"),
    });
}

#[test]
fn generate_value_action_diffs() {
    let input_value = 0.5;
    let mut app = create_app();
    app.add_systems(
        Update,
        pay_da_bills(move |mut action_state| {
            action_state.action_data_mut(Action::PayTheBills).value = input_value;
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action, BankAccountId>)
    .add_event::<ActionDiff<Action, BankAccountId>>();

    app.update();

    assert_action_diff_created(&mut app, |action_diff| match action_diff {
        ActionDiff::ValueChanged { id, action, value } => {
            assert!(*id == BankAccountId(1337));
            assert!(*action == Action::PayTheBills);
            assert!(*value == input_value);
        }
        ActionDiff::Released { action: _, id: _ } => {
            panic!("Expected a `ValueChanged` variant got a `Released` variant")
        }
        ActionDiff::Pressed { action: _, id: _ } => {
            panic!("Expected a `ValueChanged` variant got a `Pressed` variant")
        }
        ActionDiff::AxisPairChanged {
            action: _,
            id: _,
            axis_pair: _,
        } => panic!("Expected a `ValueChanged` variant got a `AxisPairChanged` variant"),
    });

    app.update();

    assert_has_no_action_diffs(&mut app);

    app.update();

    assert_action_diff_created(&mut app, |action_diff| match action_diff {
        ActionDiff::Released { id, action } => {
            assert!(*id == BankAccountId(1337));
            assert!(*action == Action::PayTheBills);
        }
        ActionDiff::Pressed { action: _, id: _ } => {
            panic!("Expected a `Released` variant got a `Pressed` variant")
        }
        ActionDiff::ValueChanged {
            action: _,
            id: _,
            value: _,
        } => panic!("Expected a `Released` variant got a `ValueChanged` variant"),
        ActionDiff::AxisPairChanged {
            action: _,
            id: _,
            axis_pair: _,
        } => panic!("Expected a `Released` variant got a `AxisPairChanged` variant"),
    });
}

#[test]
fn generate_axis_action_diffs() {
    let input_axis_pair = Vec2 { x: 5., y: 8. };
    let mut app = create_app();
    app.add_systems(
        Update,
        pay_da_bills(move |mut action_state| {
            action_state.action_data_mut(Action::PayTheBills).axis_pair =
                Some(DualAxisData::from_xy(input_axis_pair));
        }),
    )
    .add_systems(PostUpdate, generate_action_diffs::<Action, BankAccountId>)
    .add_event::<ActionDiff<Action, BankAccountId>>();

    app.update();

    assert_action_diff_created(&mut app, |action_diff| match action_diff {
        ActionDiff::AxisPairChanged {
            id,
            action,
            axis_pair,
        } => {
            assert!(*id == BankAccountId(1337));
            assert!(*action == Action::PayTheBills);
            assert!(*axis_pair == input_axis_pair);
        }
        ActionDiff::Released { action: _, id: _ } => {
            panic!("Expected a `AxisPairChanged` variant got a `Released` variant")
        }
        ActionDiff::Pressed { action: _, id: _ } => {
            panic!("Expected a `AxisPairChanged` variant got a `Pressed` variant")
        }
        ActionDiff::ValueChanged {
            action: _,
            id: _,
            value: _,
        } => panic!("Expected a `AxisPairChanged` variant got a `ValueChanged` variant"),
    });

    app.update();

    assert_has_no_action_diffs(&mut app);

    app.update();

    assert_action_diff_created(&mut app, |action_diff| match action_diff {
        ActionDiff::Released { id, action } => {
            assert!(*id == BankAccountId(1337));
            assert!(*action == Action::PayTheBills);
        }
        ActionDiff::Pressed { action: _, id: _ } => {
            panic!("Expected a `Released` variant got a `Pressed` variant")
        }
        ActionDiff::ValueChanged {
            action: _,
            id: _,
            value: _,
        } => panic!("Expected a `Released` variant got a `ValueChanged` variant"),
        ActionDiff::AxisPairChanged {
            action: _,
            id: _,
            axis_pair: _,
        } => panic!("Expected a `Released` variant got a `AxisPairChanged` variant"),
    });
}

#[test]
fn process_binary_action_diffs() {
    let mut app = create_app();
    app.add_systems(PreUpdate, process_action_diffs::<Action, BankAccountId>);

    let action_diff = ActionDiff::Pressed {
        id: BankAccountId(1337),
        action: Action::PayTheBills,
    };
    send_action_diff(&mut app, action_diff.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff);

    let action_diff = ActionDiff::Released {
        id: BankAccountId(1337),
        action: Action::PayTheBills,
    };
    send_action_diff(&mut app, action_diff.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff);
}

#[test]
fn process_value_action_diff() {
    let mut app = create_app();
    app.add_systems(PreUpdate, process_action_diffs::<Action, BankAccountId>);

    let action_diff = ActionDiff::ValueChanged {
        id: BankAccountId(1337),
        action: Action::PayTheBills,
        value: 0.5,
    };
    send_action_diff(&mut app, action_diff.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff);

    let action_diff = ActionDiff::Released {
        id: BankAccountId(1337),
        action: Action::PayTheBills,
    };
    send_action_diff(&mut app, action_diff.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff);
}

#[test]
fn process_axis_action_diff() {
    let mut app = create_app();
    app.add_systems(PreUpdate, process_action_diffs::<Action, BankAccountId>);

    let action_diff = ActionDiff::AxisPairChanged {
        id: BankAccountId(1337),
        action: Action::PayTheBills,
        axis_pair: Vec2 { x: 1., y: 0. },
    };
    send_action_diff(&mut app, action_diff.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff);

    let action_diff = ActionDiff::Released {
        id: BankAccountId(1337),
        action: Action::PayTheBills,
    };
    send_action_diff(&mut app, action_diff.clone());

    app.update();

    assert_action_diff_received(&mut app, action_diff);
}
