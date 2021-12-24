use bevy::prelude::*;

pub mod input;
pub mod movement;
pub mod utils;

pub struct HelloWorldPlugin;

impl Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(hello_world);
    }
}

fn hello_world() {
    println!("Hello, World!");
}
