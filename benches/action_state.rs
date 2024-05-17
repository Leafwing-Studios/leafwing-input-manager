use bevy::{prelude::Reflect, utils::HashMap};
use criterion::{criterion_group, criterion_main, Criterion};
use leafwing_input_manager::{
    action_state::ActionData, buttonlike::ButtonState, prelude::ActionState, timing::Timing,
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

impl TestAction {
    fn variants() -> impl Iterator<Item = TestAction> {
        use TestAction::*;
        [A, B, C, D, E, F, G, H, I, J].iter().copied()
    }
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

fn update(mut action_state: ActionState<TestAction>, action_data: HashMap<TestAction, ActionData>) {
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

    let action_data: HashMap<TestAction, ActionData> = TestAction::variants()
        .map(|action| {
            (
                action,
                ActionData {
                    state: ButtonState::JustPressed,
                    update_state: ButtonState::Released,
                    fixed_update_state: ButtonState::Released,
                    value: 0.0,
                    axis_pair: None,
                    timing: Timing::default(),
                    consumed: false,
                },
            )
        })
        .collect();

    c.bench_function("update", |b| {
        b.iter(|| update(action_state.clone(), action_data.clone()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
