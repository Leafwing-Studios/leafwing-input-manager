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

#[derive(Actionlike, Hash, PartialEq, Eq, Clone, Copy, Debug)]
enum SimpleAction {
    One,
    Two,
    Three,
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

#[test]
fn in_order_iteration() {
    let constructed_vec = vec![SimpleAction::One, SimpleAction::Two, SimpleAction::Three];
    let reversed_vec = vec![SimpleAction::Three, SimpleAction::Two, SimpleAction::One];

    let iterated_vec: Vec<SimpleAction> = SimpleAction::iter().collect();

    assert_eq!(constructed_vec, iterated_vec);
    assert!(iterated_vec != reversed_vec);
}
