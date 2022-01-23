// TODO: This example is still incomplete!

use bevy::prelude::*;

mod common_ui;
mod gameplay;
mod input_binding_menu;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(common_ui::UiPlugin)
        .add_plugin(input_binding_menu::InputMenuPlugin)
        .run()
}
