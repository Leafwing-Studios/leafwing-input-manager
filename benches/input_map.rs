use bevy::prelude::Reflect;
use bevy::{
    input::InputPlugin,
    prelude::{App, KeyCode},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    action_state::ActionData,
    input_streams::InputStreams,
    prelude::{ClashStrategy, InputMap, MockInput},
    Actionlike,
};

#[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
enum TestAction {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
}

fn construct_input_map_from_iter() -> InputMap<TestAction> {
    black_box(InputMap::new([
        (TestAction::A, KeyCode::A),
        (TestAction::B, KeyCode::B),
        (TestAction::C, KeyCode::C),
        (TestAction::D, KeyCode::D),
        (TestAction::E, KeyCode::E),
        (TestAction::F, KeyCode::F),
        (TestAction::G, KeyCode::G),
        (TestAction::H, KeyCode::H),
        (TestAction::I, KeyCode::I),
        (TestAction::J, KeyCode::J),
    ]))
}

fn construct_input_map_from_chained_calls() -> InputMap<TestAction> {
    black_box(
        InputMap::default()
            .insert(TestAction::A, KeyCode::A)
            .insert(TestAction::B, KeyCode::B)
            .insert(TestAction::C, KeyCode::C)
            .insert(TestAction::D, KeyCode::D)
            .insert(TestAction::E, KeyCode::E)
            .insert(TestAction::F, KeyCode::F)
            .insert(TestAction::G, KeyCode::G)
            .insert(TestAction::H, KeyCode::H)
            .insert(TestAction::I, KeyCode::I)
            .insert(TestAction::J, KeyCode::J)
            .build(),
    )
}

fn which_pressed(input_streams: &InputStreams, clash_strategy: ClashStrategy) -> Vec<ActionData> {
    let input_map = construct_input_map_from_iter();
    input_map.which_pressed(input_streams, clash_strategy)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("construct_input_map_from_iter", |b| {
        b.iter(construct_input_map_from_iter)
    });
    c.bench_function("construct_input_map_from_chained_calls", |b| {
        b.iter(construct_input_map_from_chained_calls)
    });
    let mut which_pressed_group = c.benchmark_group("which_pressed");

    // Constructing our test app / input stream outside of the timed benchmark
    let mut app = App::new();
    app.add_plugins(InputPlugin);
    app.send_input(KeyCode::A);
    app.send_input(KeyCode::B);
    app.update();

    let input_streams = InputStreams::from_world(&app.world, None);

    for clash_strategy in ClashStrategy::variants() {
        which_pressed_group.bench_function(format!("{:?}", clash_strategy), |b| {
            b.iter(|| which_pressed(&input_streams, *clash_strategy))
        });
    }
    which_pressed_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
