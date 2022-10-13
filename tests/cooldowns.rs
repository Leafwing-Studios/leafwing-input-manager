// BLOCKED: these tests should set the time manually.
// Requires https://github.com/bevyengine/bevy/issues/6146 to do so.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::utils::Duration;
use leafwing_input_manager::prelude::*;
use std::thread::sleep;

#[derive(Actionlike, Clone, Copy)]
enum Action {
    NoCooldown,
    Zero,
    Short,
    Long,
}

impl Action {
    fn cooldown(&self) -> Option<Cooldown> {
        match self {
            Action::NoCooldown => None,
            Action::Zero => Some(Cooldown::new(Duration::ZERO)),
            Action::Short => Some(Cooldown::from_secs(0.1)),
            Action::Long => Some(Cooldown::from_secs(1.)),
        }
    }

    fn cooldowns() -> Cooldowns<Action> {
        let mut cd = Cooldowns::default();
        for action in Action::variants() {
            if let Some(cooldown) = action.cooldown() {
                cd.set(cooldown, action);
            }
        }

        cd
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn_bundle(InputManagerBundle {
        cooldowns: Action::cooldowns(),
        ..default()
    });
}

#[test]
fn cooldowns_on_entity() {
    use Action::*;

    let mut app = App::new();
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_startup_system(spawn);

    // Spawn entities
    app.update();

    // Cooldown start ready
    let mut query_state = app.world.query::<&mut Cooldowns<Action>>();
    let mut cooldowns: Mut<Cooldowns<Action>> = query_state.single_mut(&mut app.world);
    for action in Action::variants() {
        assert!(cooldowns.ready(action));
        // Trigger all the cooldowns once
        cooldowns.trigger(action);
    }

    app.update();

    // No waiting
    let mut query_state = app.world.query::<&Cooldowns<Action>>();
    let cooldowns: &Cooldowns<Action> = query_state.single(&mut app.world);
    assert!(cooldowns.ready(NoCooldown));
    assert!(cooldowns.ready(Zero));
    assert!(!cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));

    sleep(Duration::from_secs_f32(0.2));
    app.update();

    // Short wait
    let mut query_state = app.world.query::<&Cooldowns<Action>>();
    let cooldowns: &Cooldowns<Action> = query_state.single(&mut app.world);
    assert!(cooldowns.ready(NoCooldown));
    assert!(cooldowns.ready(Zero));
    assert!(cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));
}

#[test]
fn cooldowns_in_resource() {
    use Action::*;

    let mut app = App::new();
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .insert_resource(Action::cooldowns());

    // Cooldown start ready
    let mut cooldowns: Mut<Cooldowns<Action>> = app.world.resource_mut();
    for action in Action::variants() {
        assert!(cooldowns.ready(action));
        cooldowns.trigger(action);
    }

    app.update();

    // No waiting
    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(cooldowns.ready(NoCooldown));
    assert!(cooldowns.ready(Zero));
    assert!(!cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));

    sleep(Duration::from_secs_f32(0.2));
    app.update();

    // Short wait
    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(cooldowns.ready(NoCooldown));
    assert!(cooldowns.ready(Zero));
    assert!(cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));
}
