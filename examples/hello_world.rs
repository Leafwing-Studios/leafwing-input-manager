use bevy::prelude::*;
use template_lib::HelloWorldPlugin;

fn main() {
    App::build().add_plugin(HelloWorldPlugin).run();
}
