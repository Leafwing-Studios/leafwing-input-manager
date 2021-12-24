use bevy::ecs::{
    system::{CommandQueue, Commands},
    world::World,
};
use criterion::{criterion_group, criterion_main, Criterion};

criterion_group!(benches, spawn_entities);
criterion_main!(benches);

fn spawn_entities(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("spawn_entities");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    for entity_count in (1..5).map(|i| i * 2 * 1000) {
        group.bench_function(format!("{}_entities", entity_count), |bencher| {
            let mut world = World::default();
            let mut command_queue = CommandQueue::default();

            bencher.iter(|| {
                let mut commands = Commands::new(&mut command_queue, &world);
                for _ in 0..entity_count {
                    commands.spawn();
                }
                command_queue.apply(&mut world);
            });
        });
    }

    group.finish();
}
