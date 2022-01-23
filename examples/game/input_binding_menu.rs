use bevy::prelude::*;
use leafwing_input_manager::{input_map::InputMap, user_input::UserInput};
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

    let column_heading_style = TextStyle {
        font: fonts.get_handle(FontType::Body),
        font_size: 20.0,
        color: Color::BLACK,
    };

    let body_style = TextStyle {
        font: fonts.get_handle(FontType::Body),
        font_size: 16.0,
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

    // Stores our title, column headings, input selectors and quit menu button
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

    // This call spawns an entity within the function as a side effect
    // returning the Entity reference of the row itself
    let column_headings_row = spawn_input_binding_row(
        &mut commands,
        "Actions".to_string(),
        "Primary Input".to_string(),
        "Secondary Input".to_string(),
        column_heading_style,
    );

    commands
        .entity(menu_node)
        .push_children(&[title_node, column_headings_row]);

    // We need a variable number of distinct input binding rows,
    // stacked vertically on top of each other (from top to bottom)

    for action in GameAction::iter() {
        // In this example, we only permit two input map bindings per action
        let input_binding_row_entity = spawn_input_binding_row(
            &mut commands,
            action.to_string(),
            // The strings displayed for our input bindings start empty,
            // but are immediately set when the input map is initialized
            "".to_string(),
            "".to_string(),
            body_style.clone(),
        );

        commands
            .entity(menu_node)
            .add_child(input_binding_row_entity);
    }
}

fn spawn_input_binding_row(
    commands: &mut Commands,
    action: String,
    input_1: String,
    input_2: String,
    text_style: TextStyle,
) -> Entity {
    let row_node = commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                padding: Rect {
                    top: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                },
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
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

    let action_text = commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                action,
                text_style.clone(),
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Left,
                },
            ),
            ..Default::default()
        })
        .id();

    commands.entity(action_node).add_child(action_text);

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

    let input1_text = commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                input_1,
                text_style.clone(),
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            ..Default::default()
        })
        .id();

    commands.entity(input1_node).add_child(input1_text);

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

    let input2_text = commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                input_2,
                text_style.clone(),
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            ..Default::default()
        })
        .id();

    commands.entity(input2_node).add_child(input2_text);

    commands
        .entity(row_node)
        .push_children(&[action_node, input1_node, input2_node]);

    row_node
}
