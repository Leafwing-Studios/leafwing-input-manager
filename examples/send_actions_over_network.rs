//! [`ActionDiff`] event streams are minimalistic representations
//! of the action state, intended for serialization and networking
//! While they are less convenient to work with than the complete [`ActionState`],
//! they are much smaller, and can be created from and reconstructed into [`ActionState`]
//!
//! Note that [`ActionState`] can also be serialized and sent directly.
//! This approach will be less bandwidth efficient, but involve less complexity and CPU work.

use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionDiff;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::systems::{generate_action_diffs, process_action_diffs};

use std::fmt::Debug;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum FpsAction {
    MoveLeft,
    MoveRight,
    Jump,
    Shoot,
}

/// This identifier uniquely identifies entities across the network
#[derive(Component, Clone, PartialEq, Eq, Debug)]
struct StableId(u64);

fn main() {
    // In a real use case, these apps would be running on seperate devices
    let mut client_app = App::new();

    client_app
        .add_plugins(MinimalPlugins)
        .add_plugin(InputPlugin)
        .add_plugin(InputManagerPlugin::<FpsAction>::default())
        // Creates an event stream of `ActionDiffs` to send to the server
        .add_system_to_stage(
            CoreStage::PostUpdate,
            generate_action_diffs::<FpsAction, StableId>,
        )
        .add_event::<ActionDiff<FpsAction, StableId>>()
        .add_startup_system(spawn_player);

    let mut server_app = App::new();
    server_app
        .add_plugins(MinimalPlugins)
        .add_plugin(InputManagerPlugin::<FpsAction>::server())
        .add_event::<ActionDiff<FpsAction, StableId>>()
        // Reads in the event stream of `ActionDiffs` to update the `ActionState`
        .add_system_to_stage(
            CoreStage::PreUpdate,
            process_action_diffs::<FpsAction, StableId>,
        )
        // Typically, the rest of this information would synchronized as well
        .add_startup_system(spawn_player);

    // Starting up the game
    client_app.update();

    // Sending inputs to the client
    client_app.send_input(KeyCode::Space);
    client_app.send_input(MouseButton::Left);

    // These are converted into actions when the client_app's `Schedule` runs
    client_app.update();

    let mut player_state_query = client_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&client_app.world).next().unwrap();
    assert!(player_state.pressed(FpsAction::Jump));
    assert!(player_state.pressed(FpsAction::Shoot));

    // These events are transferred to the server
    let event_reader =
        send_events::<ActionDiff<FpsAction, StableId>>(&client_app, &mut server_app, None);

    // The server processes the event stream
    server_app.update();

    // And the actions are pressed on the server!
    let mut player_state_query = server_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&server_app.world).next().unwrap();
    assert!(player_state.pressed(FpsAction::Jump));
    assert!(player_state.pressed(FpsAction::Shoot));

    // If we wait a tick, the buttons will be released
    client_app.reset_inputs();
    client_app.update();
    let mut player_state_query = client_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&client_app.world).next().unwrap();
    assert!(player_state.released(FpsAction::Jump));
    assert!(player_state.released(FpsAction::Shoot));

    // Sending over the new `ActionDiff` event stream,
    // we can see that the actions are now released on the server too
    let _event_reader = send_events::<ActionDiff<FpsAction, StableId>>(
        &client_app,
        &mut server_app,
        Some(event_reader),
    );

    server_app.update();

    let mut player_state_query = server_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&server_app.world).next().unwrap();
    assert!(player_state.released(FpsAction::Jump));
    assert!(player_state.released(FpsAction::Shoot));
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    use FpsAction::*;
    use KeyCode::*;

    commands
        .spawn_bundle(InputManagerBundle {
            input_map: InputMap::new([(W, MoveLeft), (D, MoveRight), (Space, Jump)])
                .insert(MouseButton::Left, Shoot)
                .build(),
            ..default()
        })
        // This identifier must match on both the client and server
        // and be unique between players
        .insert(StableId(76))
        .insert(Player);
}

/// A simple mock network interface that copies a set of events from the client to the server
///
/// The events are sent directly;
/// in real applications they would be serialized to a networking protocol instead.
///
/// The [`ManualEventReader`] returned must be reused in order to avoid double-sending events
#[must_use]
fn send_events<A: Send + Sync + 'static + Debug + Clone>(
    client_app: &App,
    server_app: &mut App,
    reader: Option<ManualEventReader<A>>,
) -> ManualEventReader<A> {
    let client_events: &Events<A> = client_app.world.resource();
    let mut server_events: Mut<Events<A>> = server_app.world.resource_mut();

    // Get an event reader, one way or another
    let mut reader = reader.unwrap_or_else(|| client_events.get_reader());

    // Push the clients' events to the server
    for client_event in reader.iter(client_events) {
        dbg!(client_event.clone());
        server_events.send(client_event.clone());
    }

    // Return the event reader for reuse
    reader
}
