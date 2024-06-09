//! Demonstrates how to "consume" actions, so they can only be responded to by a single system

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use menu_mocking::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<MenuAction>::default())
        .init_resource::<ActionState<MenuAction>>()
        .insert_resource(InputMap::<MenuAction>::new([
            (MenuAction::CloseWindow, KeyCode::Escape),
            (MenuAction::OpenMainMenu, KeyCode::KeyM),
            (MenuAction::OpenSubMenu, KeyCode::KeyS),
        ]))
        .init_resource::<MainMenu>()
        .init_resource::<SubMenu>()
        .add_systems(Update, report_menus)
        .add_systems(Update, open_main_menu)
        .add_systems(Update, open_sub_menu)
        // We want to ensure that if both the main menu and submenu are open,
        // only the submenu is closed if the user hits (or holds) Escape
        .add_systems(Update, close_menu::<SubMenu>.before(close_menu::<MainMenu>))
        // We can do this by ordering our systems and using `ActionState::consume`
        .add_systems(Update, close_menu::<MainMenu>)
        .run();
}

#[derive(Actionlike, Debug, Clone, Reflect, PartialEq, Eq, Hash)]
enum MenuAction {
    CloseWindow,
    OpenMainMenu,
    OpenSubMenu,
}

// A simple "visualization" of app state
fn report_menus(main_menu: Res<MainMenu>, submenu: Res<SubMenu>) {
    if main_menu.is_changed() {
        if main_menu.is_open() {
            println!("The main menu is now open.")
        } else {
            println!("The main menu is now closed.")
        }
    }

    if submenu.is_changed() {
        if submenu.is_open() {
            println!("The submenu is now open.")
        } else {
            println!("The submenu is now closed.")
        }
    }
}

fn open_main_menu(action_state: Res<ActionState<MenuAction>>, mut menu_state: ResMut<MainMenu>) {
    if action_state.just_pressed(&MenuAction::OpenMainMenu) && !menu_state.is_open() {
        menu_state.open();
    }
}

fn open_sub_menu(action_state: Res<ActionState<MenuAction>>, mut menu_state: ResMut<SubMenu>) {
    if action_state.just_pressed(&MenuAction::OpenSubMenu) && !menu_state.is_open() {
        menu_state.open();
    }
}

// We want to ensure that, e.g., the submenu is closed in preference to the main menu if both are open.
// If you can, use a real focus system for this logic.
// However, workarounds of this sort are necessary in bevy_egui
// as it is an immediate mode UI library
fn close_menu<M: Resource + Menu>(
    mut action_state: ResMut<ActionState<MenuAction>>,
    mut menu_status: ResMut<M>,
) {
    if action_state.pressed(&MenuAction::CloseWindow) && menu_status.is_open() {
        println!("Closing the top window, as requested.");
        menu_status.close();
        // Because the action is consumed, further systems won't see this action as pressed,
        // and it cannot be pressed again until after the next time it would be released.
        action_state.consume(&MenuAction::CloseWindow);
    }
}

// A quick mock of some UI behavior for demonstration purposes
mod menu_mocking {
    use bevy::prelude::Resource;

    pub trait Menu {
        fn is_open(&self) -> bool;

        fn open(&mut self);

        fn close(&mut self);
    }

    #[derive(Resource, Default)]
    pub struct MainMenu {
        is_open: bool,
    }

    impl Menu for MainMenu {
        fn is_open(&self) -> bool {
            self.is_open
        }

        fn open(&mut self) {
            self.is_open = true;
        }

        fn close(&mut self) {
            self.is_open = false;
        }
    }

    #[derive(Resource, Default)]
    pub struct SubMenu {
        is_open: bool,
    }

    impl Menu for SubMenu {
        fn is_open(&self) -> bool {
            self.is_open
        }

        fn open(&mut self) {
            self.is_open = true;
        }

        fn close(&mut self) {
            self.is_open = false;
        }
    }
}
