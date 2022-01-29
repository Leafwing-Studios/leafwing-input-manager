//! When debugging this file, `cargo expand` is invaluable.
//! See: https://github.com/dtolnay/cargo-expand
//! use `cargo expand --test actionlike_derive`
use leafwing_input_manager::Actionlike;

#[derive(Actionlike, Hash, PartialEq, Eq, Clone, Copy)]
enum UnitAction {}

#[derive(Actionlike, Hash, PartialEq, Eq, Clone, Copy)]
enum OneAction {
    Jump,
}

#[derive(Actionlike, Hash, PartialEq, Eq, Clone, Copy)]
enum SimpleAction {
    Run,
    Jump,
}

#[derive(Actionlike, Hash, PartialEq, Eq, Clone, Copy)]
enum UnnamedFieldVariantsAction {
    Run,
    Jump(usize),
}

#[derive(Actionlike, Hash, PartialEq, Eq, Clone, Copy)]
enum NamedFieldVariantsAction {
    Run { x: usize, y: usize },
    Jump,
}
