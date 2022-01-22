use crate::common_ui::UiSetupLabels;
use bevy::prelude::*;

pub struct InputMenuPlugin;

impl Plugin for InputMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(
            spawn_input_menu
                .label(UiSetupLabels::Spawning)
                .after(UiSetupLabels::Loading),
        );
    }
}

fn spawn_input_menu(mut commands: Commands) {
    let root_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::GREEN.into(),
            ..Default::default()
        })
        .id();

    let menu_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(40.0), Val::Percent(80.0)),
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::RED.into(),
            ..Default::default()
        })
        .id();

    commands.entity(root_node).push_children(&[menu_node]);
}
