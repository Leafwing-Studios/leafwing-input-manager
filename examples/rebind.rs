use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .add_systems(Startup, spawn_player)
        // Added a system that runs each updates
        .add_systems(Update, (jump, rebind_jump_on_new_key))
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    Run,
    Jump,
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    let input_map = InputMap::new([(Action::Jump, KeyCode::Space)]);
    commands.spawn(input_map).insert(Player);
}
fn jump(action_state: Single<&ActionState<Action>, With<Player>>) {
    if action_state.just_pressed(&Action::Jump) {
        println!("I'm jumping!");
    }
}

/// Each time a new key is pressed it will rebind the jumping action to that key.
fn rebind_jump_on_new_key(
    action_state: Single<&ActionState<Action>, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query_inputmap_action: Query<&mut InputMap<Action>>
){
    // This is a rather silly check so it doesn't spam the console
    if action_state.pressed(&Action::Jump) {
        return
    }
    let mut inputmap_action = query_inputmap_action.single_mut().unwrap();
    // Checks all recent keys being pressed on the keyboard
    // If multiple keys have been pressed during this updated,
    // then it will rebind the last one in the list.
    for keycode_pressed in keyboard.get_pressed(){
        // removes previous input bindings
        inputmap_action.clear_action(&Action::Jump);
        // insert the new binding
        inputmap_action.insert(Action::Jump, *keycode_pressed);
        println!("Rebinded the jump to the key: {keycode_pressed:?}");
    }
}