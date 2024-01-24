//! When debugging this file, `cargo expand` is invaluable.
//! See: https://github.com/dtolnay/cargo-expand
//! use `cargo expand --test actionlike_derive`
use bevy::prelude::Reflect;
use leafwing_input_manager::Actionlike;

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
enum UnitAction {}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
enum OneAction {
    Jump,
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
enum SimpleAction {
    Zero,
    One,
    Two,
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
enum UnnamedFieldVariantsAction {
    Run,
    Jump(usize),
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
enum NamedFieldVariantsAction {
    Run { x: usize, y: usize },
    Jump,
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
struct StructAction {
    x: usize,
    y: usize,
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy, Reflect)]
struct TupleAction(usize, usize);
