use bevy::prelude::*;
use bevy::window::WindowMode;
use template_lib::*;

fn main() {
    App::build()
        // Configure the game window
        .insert_resource(WindowDescriptor {
            width: 1920.0,
            height: 1080.0,
            title: "Template".to_string(),
            mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        // Standard Bevy functionality
        .add_plugins(DefaultPlugins)
        // Add plugins here
        .add_plugin(HelloWorldPlugin)
        .run();
}
