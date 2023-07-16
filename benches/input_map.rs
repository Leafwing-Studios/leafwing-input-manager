use bevy::input::Input;
use bevy::prelude::Reflect;
use bevy::{
    input::InputPlugin,
    prelude::{App, KeyCode},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    action_state::ActionData,
    input_streams::InputStreams,
    prelude::{ClashStrategy, InputMap},
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
        (KeyCode::A, TestAction::A),
        (KeyCode::B, TestAction::B),
        (KeyCode::C, TestAction::C),
        (KeyCode::D, TestAction::D),
        (KeyCode::E, TestAction::E),
        (KeyCode::F, TestAction::F),
        (KeyCode::G, TestAction::G),
        (KeyCode::H, TestAction::H),
        (KeyCode::I, TestAction::I),
        (KeyCode::J, TestAction::J),
    ]))
}

fn construct_input_map_from_chained_calls() -> InputMap<TestAction> {
    black_box(
        InputMap::default()
            .insert(KeyCode::A, TestAction::A)
            .insert(KeyCode::B, TestAction::B)
            .insert(KeyCode::C, TestAction::C)
            .insert(KeyCode::D, TestAction::D)
            .insert(KeyCode::E, TestAction::E)
            .insert(KeyCode::F, TestAction::F)
            .insert(KeyCode::G, TestAction::G)
            .insert(KeyCode::H, TestAction::H)
            .insert(KeyCode::I, TestAction::I)
            .insert(KeyCode::J, TestAction::J)
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
    app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::A);
    app.world.resource_mut::<Input<KeyCode>>().press(KeyCode::B);
    app.update();

    let input_streams = InputStreams::from_world(&app.world);

    for clash_strategy in ClashStrategy::variants() {
        which_pressed_group.bench_function(format!("{:?}", clash_strategy), |b| {
            b.iter(|| which_pressed(&input_streams, *clash_strategy))
        });
    }
    which_pressed_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
