//! [`ActionDiff`] event streams are minimalistic representations
//! of the action state, intended for serialization and networking
//! While they are less convenient to work with than the complete [`ActionState`],
//! they are much smaller, and can be created from and reconstructed into [`ActionState`]
//!
//! Note that [`ActionState`] can also be serialized and sent directly.
//! This approach will be less bandwidth efficient, but involve less complexity and CPU work.

use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionDiff;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::MockInput;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum FpsAction {
    MoveLeft,
    MoveRight,
    Jump,
    Shoot,
}

fn main() {
    // In a real use case, these apps would be running on seperate devices
    let mut client_app = App::new();

    client_app
        .add_plugins(MinimalPlugins)
        .add_plugin(InputManagerPlugin::<FpsAction>::default())
        // Creates an event stream of `ActionDiffs` to send to the server
        .add_system(generate_action_events)
        .add_event::<ActionDiff<FpsAction>>()
        .add_startup_system(spawn_player);

    let mut server_app = App::new();
    server_app
        .add_plugins(MinimalPlugins)
        .add_plugin(InputManagerPlugin::<FpsAction>::server())
        .add_event::<ActionDiff<FpsAction>>()
        // Reads in the event stream of `ActionDiffs` to update the `ActionState`
        .add_system_to_stage(CoreStage::PreUpdate, process_action_events)
        // Typically, the rest of this information would synchronized as well
        .add_startup_system(spawn_player);

    // Starting up the game
    client_app.update();

    // Sending inputs to the client
    client_app.send_input(KeyCode::Space);
    client_app.send_input(MouseButton::Left);

    // These are converted into actions when the client_app's `Schedule` runs
    let mut player_state_query = client_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&client_app.world).next().unwrap();
    assert!(player_state.pressed(FpsAction::Jump));
    assert!(player_state.pressed(FpsAction::Shoot));

    // These events are serialized, transferred to the server and then deserialized
    send_events::<ActionDiff<FpsAction>>(&client_app, &mut server_app);

    // The server processes the event stream
    server_app.update();

    // And the actions are pressed on the server!
    let mut player_state_query = server_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&client_app.world).next().unwrap();
    assert!(player_state.pressed(FpsAction::Jump));
    assert!(player_state.pressed(FpsAction::Shoot));

    // If we wait a tick, the buttons will be released
    client_app.update();
    let mut player_state_query = client_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&client_app.world).next().unwrap();
    assert!(player_state.released(FpsAction::Jump));
    assert!(player_state.released(FpsAction::Shoot));

    // Sending over the new `ActionDiff` event stream,
    // we can see that the actions are now released on the server too
    send_events::<ActionDiff<FpsAction>>(&client_app, &mut server_app);

    let mut player_state_query = server_app.world.query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(&client_app.world).next().unwrap();
    assert!(player_state.released(FpsAction::Jump));
    assert!(player_state.released(FpsAction::Shoot));
}

fn spawn_player(mut commands: Commands) {
    todo!()
}

fn generate_action_events(
    action_state_query: Query<&ActionState<FpsAction>>,
    action_diffs: EventWriter<ActionDiff<FpsAction>>,
) {
    todo!()
}

fn process_action_events(
    action_state_query: Query<&mut ActionState<FpsAction>>,
    action_diffs: EventReader<ActionDiff<FpsAction>>,
) {
    todo!()
}

/// A simple mock network interface that copies a set of events from the client to the server
///
/// The events are serialized and then deserialized to the hard drive;
/// in real applications they would be serialized to a networking protocol instead
fn send_events<A: Send + Sync + 'static + Clone>(client_app: &App, server_app: &mut App) {
    todo!()
}
