//! In some genres (commonly Diablo-like action RPGs), it is common to have two layers of actions.
//! The first layer corresponds to a "slot",
//! which can then be set to a second-layer "ability" of the player's choice
//!
//! This example demonstrates how to model that pattern by copying [`ActionData`]
//! between two distinct [`ActionState`] components.

use bevy::prelude::*;
use bevy::utils::HashMap;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // These are the generic "slots" that make up the player's action bar
        .add_plugin(InputManagerPlugin::<Slot>::default())
        // These are the actual abilities used by our characters
        .add_plugin(InputManagerPlugin::<Ability>::default())
        .add_startup_system(spawn_player)
        // This system coordinates the state of our two actions
        .add_system_to_stage(
            CoreStage::PreUpdate,
            copy_action_state.after(InputManagerSystem::ManualControl),
        )
        // Try it out, using QWER / left click / right click!
        .add_system(report_abilities_used)
        .run();
}

#[derive(Actionlike, PartialEq, Eq, Clone, Debug, Hash, Copy)]
enum Slot {
    Primary,
    Secondary,
    Ability1,
    Ability2,
    Ability3,
    Ability4,
}

// The list of possible abilities is typically longer than the list of slots
#[derive(Actionlike, PartialEq, Eq, Clone, Debug, Copy)]
enum Ability {
    Slash,
    Shoot,
    LightningBolt,
    Fireball,
    Dash,
    Heal,
    FrozenOrb,
    PolymorphSheep,
}

/// This struct stores which ability corresponds to which slot for a particular player
#[derive(Component, Debug, Default, Deref, DerefMut)]
struct AbilitySlotMap {
    map: HashMap<Slot, Ability>,
}

#[derive(Component)]
struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    slot_input_map: InputMap<Slot>,
    slot_action_state: ActionState<Slot>,
    // We do not need an InputMap<Ability> component,
    // as abilities are never triggered directly from inputs.
    ability_action_state: ActionState<Ability>,
    ability_slot_map: AbilitySlotMap,
}

fn spawn_player(mut commands: Commands) {
    use KeyCode::*;

    // We can control which abilities are stored in each mapping
    let mut ability_slot_map = AbilitySlotMap::default();
    ability_slot_map.insert(Slot::Primary, Ability::Slash);
    ability_slot_map.insert(Slot::Secondary, Ability::Shoot);
    ability_slot_map.insert(Slot::Ability1, Ability::FrozenOrb);
    // Some slots may be empty!
    ability_slot_map.insert(Slot::Ability3, Ability::Dash);
    ability_slot_map.insert(Slot::Ability4, Ability::PolymorphSheep);

    commands.spawn_bundle(PlayerBundle {
        player: Player,
        slot_input_map: InputMap::new([
            (Q, Slot::Ability1),
            (W, Slot::Ability2),
            (E, Slot::Ability3),
            (R, Slot::Ability4),
        ])
        .insert(MouseButton::Left, Slot::Primary)
        .insert(MouseButton::Right, Slot::Secondary)
        .build(),
        slot_action_state: ActionState::default(),
        ability_action_state: ActionState::default(),
        ability_slot_map,
    });
}

fn copy_action_state(
    mut query: Query<(
        &ActionState<Slot>,
        &mut ActionState<Ability>,
        &AbilitySlotMap,
    )>,
) {
    for (slot_state, mut ability_state, ability_slot_map) in query.iter_mut() {
        for slot in Slot::variants() {
            if let Some(&matching_ability) = ability_slot_map.get(&slot) {
                // This copies the `ActionData` between the ActionStates,
                // including information about how long the buttons have been pressed or released
                ability_state
                    .set_action_data(matching_ability, slot_state.action_data(slot).clone());
            }
        }
    }
}

fn report_abilities_used(query: Query<&ActionState<Ability>>) {
    for ability_state in query.iter() {
        for ability in ability_state.get_just_pressed() {
            dbg!(ability);
        }
    }
}
