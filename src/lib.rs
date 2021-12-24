use bevy::prelude::*;

pub mod utils;

pub struct HelloWorldPlugin;

impl Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(hello_world.system());
    }
}

fn hello_world() {
    println!("Hello, World!");
}
