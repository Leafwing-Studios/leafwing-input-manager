use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum_macros::EnumIter;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter)]
pub enum GameAction {
    Jump,
    MoveLeft,
    MoveRight,
    Shoot,
}
