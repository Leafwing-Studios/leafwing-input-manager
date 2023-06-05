use crate::axislike::DualAxisData;
use crate::input_like::{ButtonLike, DualAxisLike, InputLike, InputLikeObject, SingleAxisLike};
use crate::prelude::QwertyScanCode;
use bevy::prelude::{Reflect, World};
use erased_serde::Serialize;

#[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an un-quoted item
/// A virtual DPad that you can get an [`DualAxis`] from.
///
/// Typically, you don't want to store a [`DualAxis`] in this type,
/// even though it can be stored as an [`InputKind`].
///
/// Instead, use it directly as [`InputKind::DualAxis`]!
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualDPad {
    /// The input that represents the up direction in this virtual DPad
    #[serde(deserialize_with = "deserialize_boxed_button")]
    pub up: Box<dyn ButtonLike>,
    /// The input that represents the down direction in this virtual DPad
    #[serde(deserialize_with = "deserialize_boxed_button")]
    pub down: Box<dyn ButtonLike>,
    /// The input that represents the left direction in this virtual DPad
    #[serde(deserialize_with = "deserialize_boxed_button")]
    pub left: Box<dyn ButtonLike>,
    /// The input that represents the right direction in this virtual DPad
    #[serde(deserialize_with = "deserialize_boxed_button")]
    pub right: Box<dyn ButtonLike>,
}

impl VirtualDPad {
    /// Generates a [`VirtualDPad`] corresponding to the arrow keyboard keycodes
    pub fn arrow_keys() -> Self {
        Self {
            up: QwertyScanCode::Up.into(),
            down: QwertyScanCode::Down.into(),
            left: QwertyScanCode::Left.into(),
            right: QwertyScanCode::Right.into(),
        }
    }

    /// Generates a [`VirtualDPad`] corresponding to the `WASD` keys on the standard US QWERTY layout.
    ///
    /// Note that on other keyboard layouts, different keys need to be pressed.
    /// The _location_ of the keys is the same on all keyboard layouts.
    /// This ensures that the classic triangular shape is retained on all layouts,
    /// which enables comfortable movement controls.
    pub fn wasd() -> Self {
        Self {
            up: QwertyScanCode::W.into(),
            down: QwertyScanCode::S.into(),
            left: QwertyScanCode::A.into(),
            right: QwertyScanCode::D.into(),
        }
    }

    // TODO: add the rest of the helper functions once their corresponding InputLike is implemented
    //
    // #[allow(clippy::doc_markdown)] // False alarm because it thinks DPad is an un-quoted item
    // /// Generates a [`VirtualDPad`] corresponding to the DPad on a gamepad
    // pub fn dpad() -> VirtualDPad {
    //     VirtualDPad {
    //         up: InputKind::GamepadButton(GamepadButtonType::DPadUp),
    //         down: InputKind::GamepadButton(GamepadButtonType::DPadDown),
    //         left: InputKind::GamepadButton(GamepadButtonType::DPadLeft),
    //         right: InputKind::GamepadButton(GamepadButtonType::DPadRight),
    //     }
    // }
    //
    // /// Generates a [`VirtualDPad`] corresponding to the face buttons on a gamepad
    // ///
    // /// North corresponds to up, west corresponds to left, east corresponds to right, south corresponds to down
    // pub fn gamepad_face_buttons() -> VirtualDPad {
    //     VirtualDPad {
    //         up: InputKind::GamepadButton(GamepadButtonType::North),
    //         down: InputKind::GamepadButton(GamepadButtonType::South),
    //         left: InputKind::GamepadButton(GamepadButtonType::West),
    //         right: InputKind::GamepadButton(GamepadButtonType::East),
    //     }
    // }
    //
    // /// Generates a [`VirtualDPad`] corresponding to discretized mousewheel movements
    // pub fn mouse_wheel() -> VirtualDPad {
    //     VirtualDPad {
    //         up: InputKind::MouseWheel(MouseWheelDirection::Up),
    //         down: InputKind::MouseWheel(MouseWheelDirection::Down),
    //         left: InputKind::MouseWheel(MouseWheelDirection::Left),
    //         right: InputKind::MouseWheel(MouseWheelDirection::Right),
    //     }
    // }
    //
    // /// Generates a [`VirtualDPad`] corresponding to discretized mouse motions
    // pub fn mouse_motion() -> VirtualDPad {
    //     VirtualDPad {
    //         up: InputKind::MouseMotion(MouseMotionDirection::Up),
    //         down: InputKind::MouseMotion(MouseMotionDirection::Down),
    //         left: InputKind::MouseMotion(MouseMotionDirection::Left),
    //         right: InputKind::MouseMotion(MouseMotionDirection::Right),
    //     }
    // }
}

fn deserialize_boxed_button<'de, D>(deserializer: D) -> Result<Box<dyn ButtonLike>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    todo!("Implement deserialize for `Box<dyn ButtonLike>`");
}

impl PartialEq for VirtualDPad {
    fn eq(&self, other: &Self) -> bool {
        self.up.eq(&other.up)
            && self.down.eq(&other.down)
            && self.left.eq(&other.left)
            && self.right.eq(&other.right)
    }
}

impl Eq for VirtualDPad {}

impl<'a> InputLike<'a> for VirtualDPad {}

impl InputLikeObject for VirtualDPad {
    fn as_button(&self) -> Option<&dyn ButtonLike> {
        Some(self)
    }

    fn as_axis(&self) -> Option<&dyn SingleAxisLike> {
        None
    }

    fn as_dual_axis(&self) -> Option<&dyn DualAxisLike> {
        Some(self)
    }

    fn clone_dyn(&self) -> Box<dyn InputLikeObject> {
        Box::new(self.clone())
    }

    fn as_serialize(&self) -> &dyn Serialize {
        todo!()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        todo!()
    }

    fn raw_inputs(&self) -> Vec<Box<dyn InputLikeObject>> {
        vec![
            InputLikeObject::clone_dyn(self.up.as_ref()),
            InputLikeObject::clone_dyn(self.down.as_ref()),
            InputLikeObject::clone_dyn(self.left.as_ref()),
            InputLikeObject::clone_dyn(self.right.as_ref()),
        ]
    }
    fn len(&self) -> usize {
        4
    }
}

impl ButtonLike for VirtualDPad {
    fn input_pressed(&self, world: &bevy::prelude::World) -> bool {
        self.raw_inputs().iter().any(|x| {
            x.as_button()
                .map(|x| x.input_pressed(world))
                .unwrap_or_default()
        })
    }

    fn clone_dyn(&self) -> Box<dyn ButtonLike> {
        Box::new(self.clone())
    }
}

impl DualAxisLike for VirtualDPad {
    fn input_axis_pair(&self, world: &World) -> DualAxisData {
        let up = self
            .up
            .as_button()
            .map(|x| x.input_pressed(world))
            .unwrap_or_default();
        let down = self
            .down
            .as_button()
            .map(|x| x.input_pressed(world))
            .unwrap_or_default();
        let left = self
            .left
            .as_button()
            .map(|x| x.input_pressed(world))
            .unwrap_or_default();
        let right = self
            .right
            .as_button()
            .map(|x| x.input_pressed(world))
            .unwrap_or_default();

        let x = match (left, right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let y = match (up, down) {
            (true, false) => 1.0,
            (false, true) => -1.0,
            _ => 0.0,
        };

        DualAxisData::new(x, y)
    }
}
