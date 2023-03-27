use criterion::{criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    action_state::{ActionData, Timing},
    buttonlike::ButtonState,
    prelude::ActionState,
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

fn update() {
    let action_data = TestAction::variants()
        .map(|_action| ActionData {
            state: ButtonState::JustPressed,
            value: 0.0,
            axis_pair: None,
            timing: Timing::default(),
            consumed: false,
        })
        .collect();

    let mut action_state = ActionState::<TestAction>::default();
    action_state.update(action_data);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pressed", |b| b.iter(|| pressed()));
    c.bench_function("just_pressed", |b| b.iter(|| just_pressed()));
    c.bench_function("released", |b| b.iter(|| released()));
    c.bench_function("just_released", |b| b.iter(|| just_released()));
    c.bench_function("update", |b| b.iter(|| update()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
