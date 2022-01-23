use bevy::prelude::*;
use derive_more::Display;
use leafwing_input_manager::prelude::*;
use strum_macros::EnumIter;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, EnumIter, Display)]
pub enum GameAction {
    Jump,
    MoveLeft,
    MoveRight,
    Shoot,
}
