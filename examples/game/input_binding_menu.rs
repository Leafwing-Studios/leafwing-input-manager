use bevy::prelude::*;
use leafwing_input_manager::input_map::{InputMap, UserInput};
use strum::IntoEnumIterator;

use crate::common_ui::{FontType, Fonts, UiSetupLabels};
use crate::gameplay::GameAction;

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

fn spawn_input_menu(mut commands: Commands, fonts: Res<Fonts>) {
    // Reusable text styles
    let title_style = TextStyle {
        font: fonts.get_handle(FontType::Heading),
        font_size: 30.0,
        color: Color::BLACK,
    };
    let title_alignment = TextAlignment::default();

    // Global node for all other UI to live in
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

    // Stores our title, column headings and input selectors
    let menu_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(40.0), Val::Percent(80.0)),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::RED.into(),
            ..Default::default()
        })
        .id();

    commands.entity(root_node).add_child(menu_node);

    // Stores the title
    let title_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            color: Color::BLUE.into(),
            ..Default::default()
        })
        .id();

    let title_text = commands
        .spawn_bundle(TextBundle {
            text: Text::with_section("Configure Inputs", title_style, title_alignment),
            ..Default::default()
        })
        .id();

    commands.entity(title_node).add_child(title_text);

    // Contains the body of the menu
    // This stores a number of distinct input binding rows,
    // stacked vertically on top of each other (from top to bottom)
    let menu_body_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(60.0)),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                ..Default::default()
            },
            color: Color::PURPLE.into(),
            ..Default::default()
        })
        .id();

    commands
        .entity(menu_node)
        .push_children(&[title_node, menu_body_node]);

    // Begin with the default input map settings
    let input_map = InputMap::<GameAction>::default();

    for action in GameAction::iter() {
        // Fetch the initial bindings from the input map
        // In this game, our inputs are not split by input modes, so we pass in `input_mode = None`
        let input_bindings = input_map.get(action, None);
        // In this example, we only permit two input map bindings per action
        let input_1 = input_bindings.get_at(0);
        let input_2 = input_bindings.get_at(1);

        // This call spawns an entity within the function as a side effect
        // returning the Entity reference of the row itself
        let input_binding_row_entity =
            spawn_input_binding_row(&mut commands, action, input_1, input_2);

        commands
            .entity(menu_node)
            .add_child(input_binding_row_entity);
    }
}

fn spawn_input_binding_row(
    commands: &mut Commands,
    action: GameAction,
    input_1: Option<&UserInput>,
    input_2: Option<&UserInput>,
) -> Entity {
    let row_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                flex_direction: FlexDirection::Row,
                ..Default::default()
            },
            color: Color::ORANGE.into(),
            ..Default::default()
        })
        .id();

    let action_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(30.0), Val::Percent(100.0)),
                ..Default::default()
            },
            color: Color::RED.into(),
            ..Default::default()
        })
        .id();

    let input1_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(30.0), Val::Percent(100.0)),
                ..Default::default()
            },
            color: Color::GREEN.into(),
            ..Default::default()
        })
        .id();

    let input2_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(30.0), Val::Percent(100.0)),
                ..Default::default()
            },
            color: Color::BLUE.into(),
            ..Default::default()
        })
        .id();

    commands
        .entity(row_node)
        .push_children(&[action_node, input1_node, input2_node]);

    row_node
}
