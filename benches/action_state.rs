use bevy::prelude::KeyCode;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    prelude::{ActionState, InputMap},
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

fn simple_input_map() -> InputMap<TestAction> {
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

fn pressed() -> bool {
    let action_state = ActionState::<TestAction>::default();
    action_state.pressed(TestAction::A)
}

fn just_pressed() -> bool {
    let action_state = ActionState::<TestAction>::default();
    action_state.just_pressed(TestAction::A)
}

fn released() -> bool {
    let action_state = ActionState::<TestAction>::default();
    action_state.released(TestAction::A)
}

fn just_released() -> bool {
    let action_state = ActionState::<TestAction>::default();
    action_state.just_released(TestAction::A)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pressed", |b| b.iter(|| pressed()));
    c.bench_function("just_pressed", |b| b.iter(|| just_pressed()));
    c.bench_function("released", |b| b.iter(|| released()));
    c.bench_function("just_released", |b| b.iter(|| just_released()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
