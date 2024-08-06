use bevy::prelude::Reflect;
use bevy::{
    input::InputPlugin,
    prelude::{App, KeyCode},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use leafwing_input_manager::input_map::UpdatedActions;
use leafwing_input_manager::plugin::AccumulatorPlugin;
use leafwing_input_manager::prelude::Buttonlike;
use leafwing_input_manager::{
    input_store::InputStreams,
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
        (TestAction::A, KeyCode::KeyA),
        (TestAction::B, KeyCode::KeyB),
        (TestAction::C, KeyCode::KeyC),
        (TestAction::D, KeyCode::KeyD),
        (TestAction::E, KeyCode::KeyE),
        (TestAction::F, KeyCode::KeyF),
        (TestAction::G, KeyCode::KeyG),
        (TestAction::H, KeyCode::KeyH),
        (TestAction::I, KeyCode::KeyI),
        (TestAction::J, KeyCode::KeyJ),
    ]))
}

fn construct_input_map_from_chained_calls() -> InputMap<TestAction> {
    black_box(
        InputMap::default()
            .with(TestAction::A, KeyCode::KeyA)
            .with(TestAction::B, KeyCode::KeyB)
            .with(TestAction::C, KeyCode::KeyC)
            .with(TestAction::D, KeyCode::KeyD)
            .with(TestAction::E, KeyCode::KeyE)
            .with(TestAction::F, KeyCode::KeyF)
            .with(TestAction::G, KeyCode::KeyG)
            .with(TestAction::H, KeyCode::KeyH)
            .with(TestAction::I, KeyCode::KeyI)
            .with(TestAction::J, KeyCode::KeyJ),
    )
}

fn which_pressed(
    input_store: &CentralInputStore,
    clash_strategy: ClashStrategy,
) -> UpdatedActions<TestAction> {
    let input_map = construct_input_map_from_iter();
    input_map.process_actions(input_store, clash_strategy)
}

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("construct_input_map_from_iter", |b| {
        b.iter(construct_input_map_from_iter)
    });
    c.bench_function("construct_input_map_from_chained_calls", |b| {
        b.iter(construct_input_map_from_chained_calls)
    });
    let mut which_pressed_group = c.benchmark_group("which_pressed");

    // Constructing our test app / input stream outside the timed benchmark
    let mut app = App::new();
    app.add_plugins((InputPlugin, AccumulatorPlugin));
    KeyCode::KeyA.press(app.world_mut());
    KeyCode::KeyB.press(app.world_mut());
    app.update();

    let input_store = CentralInputStore::from_world(app.world_mut());

    for clash_strategy in ClashStrategy::variants() {
        which_pressed_group.bench_function(format!("{:?}", clash_strategy), |b| {
            b.iter(|| which_pressed(&input_store, *clash_strategy))
        });
    }
    which_pressed_group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
