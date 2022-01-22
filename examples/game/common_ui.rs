use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum FontType {
    Body,
    Heading,
}

fn load_fonts(asset_server: Res<AssetServer>, mut fonts: ResMut<Fonts>) {
    let body_font_handle: Handle<Font> =
        asset_server.load("Montserrat/static/Montserrat-Black.ttf");
    let heading_font_handle: Handle<Font> =
        asset_server.load("Montserrat/static/Montserrat-Black.ttf");

    fonts.map.insert(FontType::Body, body_font_handle);
    fonts.map.insert(FontType::Heading, heading_font_handle);
}

#[derive(SystemLabel, Clone, PartialEq, Eq, Hash, Debug)]
pub enum UiSetupLabels {
    Loading,
    Spawning,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fonts>()
            .add_startup_system(load_fonts.label(UiSetupLabels::Loading))
            .add_startup_system(spawn_camera.label(UiSetupLabels::Spawning));
    }
}
#[derive(Default)]
pub struct Fonts {
    map: HashMap<FontType, Handle<Font>>,
}

impl Fonts {
    pub fn get_handle(&self, font_type: FontType) -> Handle<Font> {
        self.map.get(&font_type).unwrap().clone_weak()
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}
