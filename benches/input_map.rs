use bevy::prelude::KeyCode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    action_state::ActionData,
    input_streams::InputStreams,
    prelude::{ClashStrategy, InputMap},
    Actionlike,
};

#[derive(Actionlike, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("construct_input_map_from_iter", |b| {
        b.iter(|| construct_input_map_from_iter())
    });
    c.bench_function("construct_input_map_from_chained_calls", |b| {
        b.iter(|| construct_input_map_from_chained_calls())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
