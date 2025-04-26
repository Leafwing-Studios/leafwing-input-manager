//! Keyboard inputs

use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::StaticSystemParam;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::{ButtonInput, ButtonState};
use bevy::prelude::{Entity, Events, KeyCode, Reflect, ResMut, World};
use leafwing_input_manager_macros::serde_typetag;
use serde::{Deserialize, Serialize};

use crate as leafwing_input_manager;
use crate::buttonlike::ButtonValue;
use crate::clashing_inputs::BasicInputs;
use crate::user_input::{ButtonlikeChord, UserInput};
use crate::InputControlKind;

use super::updating::{CentralInputStore, UpdatableInput};
use super::Buttonlike;

// Built-in support for Bevy's KeyCode
impl UserInput for KeyCode {
    /// [`KeyCode`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Returns a [`BasicInputs`] that only contains the [`KeyCode`] itself,
    /// as it represents a simple physical button.
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Simple(Box::new(*self))
    }
}

impl UpdatableInput for KeyCode {
    type SourceData = SRes<ButtonInput<KeyCode>>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        source_data: StaticSystemParam<Self::SourceData>,
    ) {
        for key in source_data.get_pressed() {
            central_input_store.update_buttonlike(*key, ButtonValue::from_pressed(true));
        }

        for key in source_data.get_just_released() {
            central_input_store.update_buttonlike(*key, ButtonValue::from_pressed(false));
        }
    }
}

#[serde_typetag]
impl Buttonlike for KeyCode {
    /// Checks if the specified key is currently pressed down.
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Entity) -> bool {
        input_store.pressed(self)
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Pressed`].
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn press(&self, world: &mut World) {
        let mut events = world.resource_mut::<Events<KeyboardInput>>();
        events.send(KeyboardInput {
            key_code: *self,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Pressed,
            repeat: false,
            window: Entity::PLACEHOLDER,
            text: None,
        });
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Released`].
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn release(&self, world: &mut World) {
        let mut events = world.resource_mut::<Events<KeyboardInput>>();
        events.send(KeyboardInput {
            key_code: *self,
            logical_key: Key::Unidentified(NativeKey::Unidentified),
            state: ButtonState::Released,
            repeat: false,
            window: Entity::PLACEHOLDER,
            text: None,
        });
    }

    /// If the value is greater than `0.0`, press the key; otherwise release it.
    fn set_value(&self, world: &mut World, value: f32) {
        if value > 0.0 {
            self.press(world);
        } else {
            self.release(world);
        }
    }
}

/// Keyboard modifiers like Alt, Control, Shift, and Super (OS symbol key).
///
/// Each variant represents a pair of [`KeyCode`]s, the left and right version of the modifier key,
/// allowing for handling modifiers regardless of which side is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[must_use]
pub enum ModifierKey {
    /// The Alt key, representing either [`KeyCode::AltLeft`] or [`KeyCode::AltRight`].
    Alt,

    /// The Control key, representing either [`KeyCode::ControlLeft`] or [`KeyCode::ControlRight`].
    Control,

    /// The Shift key, representing either [`KeyCode::ShiftLeft`] or [`KeyCode::ShiftRight`].
    Shift,

    /// The Super (OS symbol) key, representing either [`KeyCode::SuperLeft`] or [`KeyCode::SuperRight`].
    Super,
}

impl ModifierKey {
    /// Returns a pair of [`KeyCode`]s corresponding to both modifier keys.
    #[must_use]
    #[inline]
    pub const fn keycodes(&self) -> [KeyCode; 2] {
        [self.left(), self.right()]
    }

    /// Returns the [`KeyCode`] corresponding to the left modifier key.
    #[must_use]
    #[inline]
    pub const fn left(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltLeft,
            ModifierKey::Control => KeyCode::ControlLeft,
            ModifierKey::Shift => KeyCode::ShiftLeft,
            ModifierKey::Super => KeyCode::SuperLeft,
        }
    }

    /// Returns the [`KeyCode`] corresponding to the right modifier key.
    #[must_use]
    #[inline]
    pub const fn right(&self) -> KeyCode {
        match self {
            ModifierKey::Alt => KeyCode::AltRight,
            ModifierKey::Control => KeyCode::ControlRight,
            ModifierKey::Shift => KeyCode::ShiftRight,
            ModifierKey::Super => KeyCode::SuperRight,
        }
    }

    /// Create an [`ButtonlikeChord`] that includes this [`ModifierKey`] and the given `input`.
    #[inline]
    pub fn with(&self, other: impl Buttonlike) -> ButtonlikeChord {
        ButtonlikeChord::from_single(*self).with(other)
    }
}

impl UserInput for ModifierKey {
    /// [`ModifierKey`] acts as a button.
    #[inline]
    fn kind(&self) -> InputControlKind {
        InputControlKind::Button
    }

    /// Returns the two [`KeyCode`]s used by this [`ModifierKey`].
    #[inline]
    fn decompose(&self) -> BasicInputs {
        BasicInputs::Composite(vec![Box::new(self.left()), Box::new(self.right())])
    }
}

#[serde_typetag]
impl Buttonlike for ModifierKey {
    /// Checks if the specified modifier key is currently pressed down.
    #[inline]
    fn pressed(&self, input_store: &CentralInputStore, _gamepad: Entity) -> bool {
        input_store.pressed(&self.left()) || input_store.pressed(&self.right())
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Pressed`].
    ///
    /// The left and right keys will be pressed simultaneously.
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn press(&self, world: &mut World) {
        self.left().press(world);
        self.right().press(world);
    }

    /// Sends a fake [`KeyboardInput`] event to the world with [`ButtonState::Released`].
    ///
    /// The left and right keys will be released simultaneously.
    ///
    /// # Note
    ///
    /// The `logical_key` and `window` fields will be filled with placeholder values.
    fn release(&self, world: &mut World) {
        self.left().release(world);
        self.right().release(world);
    }

    /// If the value is greater than `0.0`, press the keys; otherwise release it.
    fn set_value(&self, world: &mut World, value: f32) {
        if value > 0.0 {
            self.press(world);
        } else {
            self.release(world);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::CentralInputStorePlugin;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(InputPlugin)
            .add_plugins(CentralInputStorePlugin);
        app
    }

    #[test]
    fn test_keyboard_input() {
        let up = KeyCode::ArrowUp;
        assert_eq!(up.kind(), InputControlKind::Button);

        let left = KeyCode::ArrowLeft;
        assert_eq!(left.kind(), InputControlKind::Button);

        let alt = ModifierKey::Alt;
        assert_eq!(alt.kind(), InputControlKind::Button);

        // No inputs
        let mut app = test_app();
        app.update();
        let gamepad = app.world_mut().spawn(()).id();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));

        // Press arrow up
        let mut app = test_app();
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));

        // Press arrow down
        let mut app = test_app();
        KeyCode::ArrowDown.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));

        // Press arrow left
        let mut app = test_app();
        KeyCode::ArrowLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));

        // Press arrow down and arrow up
        let mut app = test_app();
        KeyCode::ArrowDown.press(app.world_mut());
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));

        // Press arrow left and arrow up
        let mut app = test_app();
        KeyCode::ArrowLeft.press(app.world_mut());
        KeyCode::ArrowUp.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(up.pressed(inputs, gamepad));
        assert!(left.pressed(inputs, gamepad));
        assert!(!alt.pressed(inputs, gamepad));

        // Press left Alt
        let mut app = test_app();
        KeyCode::AltLeft.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(alt.pressed(inputs, gamepad));

        // Press right Alt
        let mut app = test_app();
        KeyCode::AltRight.press(app.world_mut());
        app.update();
        let inputs = app.world().resource::<CentralInputStore>();

        assert!(!up.pressed(inputs, gamepad));
        assert!(!left.pressed(inputs, gamepad));
        assert!(alt.pressed(inputs, gamepad));
    }
}
