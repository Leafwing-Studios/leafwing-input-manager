//! When debugging this file, `cargo expand` is invaluable.
//! See: https://github.com/dtolnay/cargo-expand
//! use `cargo expand --test actionlike_derive`
use leafwing_input_manager::Actionlike;

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum UnitAction {}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum OneAction {
    Jump,
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum SimpleAction {
    Zero,
    One,
    Two,
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum UnnamedFieldVariantsAction {
    Run,
    Jump(usize),
}

#[derive(Actionlike, Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum NamedFieldVariantsAction {
    Run { x: usize, y: usize },
    Jump,
}

#[test]
fn in_order_iteration() {
    let constructed_vec = vec![SimpleAction::Zero, SimpleAction::One, SimpleAction::Two];
    let reversed_vec = vec![SimpleAction::Two, SimpleAction::One, SimpleAction::Zero];

    let iterated_vec: Vec<SimpleAction> = SimpleAction::variants().collect();

    assert_eq!(constructed_vec, iterated_vec);
    assert!(iterated_vec != reversed_vec);
}

#[test]
fn get_at() {
    assert_eq!(SimpleAction::get_at(0), Some(SimpleAction::Zero));
    assert_eq!(SimpleAction::get_at(1), Some(SimpleAction::One));
    assert_eq!(SimpleAction::get_at(2), Some(SimpleAction::Two));
    assert_eq!(SimpleAction::get_at(3), None);
}

#[test]
fn index() {
    assert_eq!(SimpleAction::Zero.index(), 0);
    assert_eq!(SimpleAction::One.index(), 1);
    assert_eq!(SimpleAction::Two.index(), 2);
}
