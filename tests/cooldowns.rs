// BLOCKED: these tests should set the time manually.
// Requires https://github.com/bevyengine/bevy/issues/6146 to do so.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy::utils::Duration;
use leafwing_input_manager::prelude::*;
use std::thread::sleep;

#[derive(Actionlike, Debug, Clone, Copy)]
enum Action {
    NoCooldown,
    Short,
    Long,
}

impl Action {
    fn cooldown(&self) -> Option<Cooldown> {
        match self {
            Action::NoCooldown => None,
            Action::Short => Some(Cooldown::from_secs(0.1)),
            Action::Long => Some(Cooldown::from_secs(1.)),
        }
    }

    fn cooldowns() -> Cooldowns<Action> {
        let mut cd = Cooldowns::default();
        for action in Action::variants() {
            if let Some(cooldown) = action.cooldown() {
                cd.set(action, cooldown);
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
    assert!(!cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));

    sleep(Duration::from_secs_f32(0.2));
    app.update();

    // Short wait
    let mut query_state = app.world.query::<&Cooldowns<Action>>();
    let cooldowns: &Cooldowns<Action> = query_state.single(&mut app.world);
    assert!(cooldowns.ready(NoCooldown));
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
    assert!(!cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));

    sleep(Duration::from_secs_f32(0.2));
    app.update();

    // Short wait
    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(cooldowns.ready(NoCooldown));
    assert!(cooldowns.ready(Short));
    assert!(!cooldowns.ready(Long));
}

#[test]
fn global_cooldowns_tick() {
    let mut app = App::new();
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .insert_resource(Action::cooldowns());

    let mut cooldowns: Mut<Cooldowns<Action>> = app.world.resource_mut();
    let initial_gcd = Some(Cooldown::new(Duration::from_micros(15)));
    cooldowns.global_cooldown = initial_gcd.clone();
    // Trigger the GCD
    cooldowns.trigger(Action::Long);

    app.update();

    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(initial_gcd != cooldowns.global_cooldown);
}

#[test]
fn global_cooldown_blocks_cooldownless_actions() {
    let mut app = App::new();
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<Cooldowns<Action>> = app.world.resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::new(Duration::from_micros(15)));

    assert!(cooldowns.ready(Action::NoCooldown));

    cooldowns.trigger(Action::NoCooldown);
    assert!(!cooldowns.ready(Action::NoCooldown));

    sleep(Duration::from_micros(30));
    app.update();

    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(cooldowns.ready(Action::NoCooldown));
}

#[test]
fn global_cooldown_affects_other_actions() {
    let mut app = App::new();
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<Cooldowns<Action>> = app.world.resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::new(Duration::from_micros(15)));
    cooldowns.trigger(Action::Long);
    assert!(!cooldowns.ready(Action::Short));
    assert!(!cooldowns.ready(Action::Long));

    sleep(Duration::from_micros(30));
    app.update();

    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(cooldowns.ready(Action::Short));
    assert!(!cooldowns.ready(Action::Long));
}

#[test]
fn global_cooldown_overrides_short_cooldowns() {
    let mut app = App::new();
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<Cooldowns<Action>> = app.world.resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::from_secs(0.5));
    cooldowns.trigger(Action::Short);
    assert!(!cooldowns.ready(Action::Short));

    // Let per-action cooldown elapse
    sleep(Duration::from_millis(200));
    app.update();

    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(!cooldowns.ready(Action::Short));

    // Wait for full GCD to expire
    sleep(Duration::from_millis(400));
    app.update();

    let cooldowns: &Cooldowns<Action> = app.world.resource();
    assert!(cooldowns.ready(Action::Short));
}
