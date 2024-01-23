use bevy::prelude::Reflect;
use criterion::{criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    action_state::{ActionData, Timing},
    buttonlike::ButtonState,
    prelude::ActionState,
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

fn pressed(action_state: &ActionState<TestAction>) -> bool {
    action_state.pressed(&TestAction::A)
}

fn just_pressed(action_state: &ActionState<TestAction>) -> bool {
    action_state.just_pressed(&TestAction::A)
}

fn released(action_state: &ActionState<TestAction>) -> bool {
    action_state.released(&TestAction::A)
}

fn just_released(action_state: &ActionState<TestAction>) -> bool {
    action_state.just_released(&TestAction::A)
}

fn update(mut action_state: ActionState<TestAction>, action_data: Vec<ActionData>) {
    action_state.update(action_data);
}

fn criterion_benchmark(c: &mut Criterion) {
    let action_state = ActionState::<TestAction>::default();

    c.bench_function("action_state_default", |b| {
        b.iter(ActionState::<TestAction>::default)
    });
    c.bench_function("pressed", |b| b.iter(|| pressed(&action_state)));
    c.bench_function("just_pressed", |b| b.iter(|| just_pressed(&action_state)));
    c.bench_function("released", |b| b.iter(|| released(&action_state)));
    c.bench_function("just_released", |b| b.iter(|| just_released(&action_state)));

    let action_data: Vec<ActionData> = TestAction::variants()
        .map(|_action| ActionData {
            state: ButtonState::JustPressed,
            value: 0.0,
            axis_pair: None,
            timing: Timing::default(),
            consumed: false,
        })
        .collect();

    c.bench_function("update", |b| {
        b.iter(|| update(action_state.clone(), action_data.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
