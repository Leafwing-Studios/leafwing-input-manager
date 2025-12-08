//! [`ActionDiff`] message streams are minimalistic representations of the action state,
//! intended for serialization and networking.
//! While they are less convenient to work with than the complete [`ActionState`],
//! they are much smaller, and can be created from and reconstructed into [`ActionState`]
//!
//! Note that [`ActionState`] can also be serialized and sent directly.
//! This approach will be less bandwidth efficient, but involve less complexity and CPU work.

use bevy::ecs::message::MessageCursor;
use bevy::input::InputPlugin;
use bevy::prelude::*;
use leafwing_input_manager::action_diff::ActionDiffMessage;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::systems::generate_action_diffs;

use std::fmt::Debug;

#[derive(Actionlike, Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
enum FpsAction {
    MoveLeft,
    MoveRight,
    Jump,
    Shoot,
}

/// Processes an [`Messages`] stream of [`ActionDiff`] to update an [`ActionState`]
///
/// In a real scenario, you would have to map the entities between the server and client world.
/// In this case, we will just use the fact that there is only a single entity.
fn process_action_diffs<A: Actionlike>(
    mut action_state_query: Query<&mut ActionState<A>>,
    mut action_diff_messages: MessageReader<ActionDiffMessage<A>>,
) {
    for action_diff_message in action_diff_messages.read() {
        if action_diff_message.owner.is_some() {
            let mut action_state = action_state_query.single_mut().unwrap();
            action_diff_message
                .action_diffs
                .iter()
                .for_each(|diff| action_state.apply_diff(diff));
        }
    }
}

fn main() {
    // In a real use case, these apps would be running on separate devices.
    let mut client_app = App::new();

    client_app
        .add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<FpsAction>::default())
        // Creates an message stream of `ActionDiffs` to send to the server
        .add_systems(PostUpdate, generate_action_diffs::<FpsAction>)
        .add_message::<ActionDiffMessage<FpsAction>>()
        .add_systems(Startup, spawn_player);

    let mut server_app = App::new();
    server_app
        .add_plugins(MinimalPlugins)
        .add_plugins(InputManagerPlugin::<FpsAction>::server())
        .add_message::<ActionDiffMessage<FpsAction>>()
        // Reads in the message stream of `ActionDiffs` to update the `ActionState`
        .add_systems(PreUpdate, process_action_diffs::<FpsAction>)
        // Typically, the rest of this information would synchronize as well
        .add_systems(Startup, spawn_player);

    // Starting up the game
    client_app.update();

    // Sending inputs to the client
    KeyCode::Space.press(client_app.world_mut());
    MouseButton::Left.press(client_app.world_mut());

    // These are converted into actions when the client_app's `Schedule` runs
    client_app.update();

    let mut player_state_query = client_app.world_mut().query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(client_app.world()).next().unwrap();
    assert!(player_state.pressed(&FpsAction::Jump));
    assert!(player_state.pressed(&FpsAction::Shoot));

    // These messages are transferred to the server
    let message_reader =
        send_messages::<ActionDiffMessage<FpsAction>>(&client_app, &mut server_app, None);

    // The server processes the message stream
    server_app.update();

    // And the actions are pressed on the server!
    let mut player_state_query = server_app.world_mut().query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(server_app.world()).next().unwrap();
    assert!(player_state.pressed(&FpsAction::Jump));
    assert!(player_state.pressed(&FpsAction::Shoot));

    // If we wait a tick, the buttons will be released
    client_app.update();
    let mut player_state_query = client_app.world_mut().query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(client_app.world()).next().unwrap();
    assert!(player_state.released(&FpsAction::Jump));
    assert!(player_state.released(&FpsAction::Shoot));

    // Sending over the new `ActionDiff` message stream,
    // we can see that the actions are now released on the server too
    let _message_reader = send_messages::<ActionDiffMessage<FpsAction>>(
        &client_app,
        &mut server_app,
        Some(message_reader),
    );

    server_app.update();

    let mut player_state_query = server_app.world_mut().query::<&ActionState<FpsAction>>();
    let player_state = player_state_query.iter(server_app.world()).next().unwrap();
    assert!(player_state.released(&FpsAction::Jump));
    assert!(player_state.released(&FpsAction::Shoot));
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    use FpsAction::*;
    use KeyCode::*;

    let input_map = InputMap::new([(MoveLeft, KeyW), (MoveRight, KeyD), (Jump, Space)])
        .with(Shoot, MouseButton::Left);
    commands.spawn(input_map).insert(Player);
}

/// A simple mock network interface that copies a set of messages from the client to the server
///
/// The messages are sent directly;
/// in real applications they would be serialized to a networking protocol instead.
///
/// The [`ManualMessageReader`] returned must be reused to avoid double-sending messages
#[must_use]
fn send_messages<A: Send + Sync + 'static + Debug + Clone + Message>(
    client_app: &App,
    server_app: &mut App,
    reader: Option<MessageCursor<A>>,
) -> MessageCursor<A> {
    let client_messages: &Messages<A> = client_app.world().resource();
    let mut server_messages: Mut<Messages<A>> = server_app.world_mut().resource_mut();

    // Get an message reader, one way or another
    let mut reader = reader.unwrap_or_else(|| client_messages.get_cursor());

    // Push the clients' messages to the server
    for client_message in reader.read(client_messages) {
        dbg!(client_message.clone());
        server_messages.write(client_message.clone());
    }

    // Return the message reader for reuse
    reader
}
