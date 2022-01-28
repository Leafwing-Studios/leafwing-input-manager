use leafwing_input_manager::Actionlike;

#[derive(Actionlike)]
enum UnitAction {}

#[derive(Actionlike)]
enum OneAction {
    Jump,
}

#[derive(Actionlike)]
enum SimpleAction {
    Run,
    Jump,
}

#[derive(Actionlike)]
enum UnnamedFieldVariantsAction {
    Run,
    Jump(usize),
}

#[derive(Actionlike)]
enum NamedFieldVariantsAction {
    Run { x: usize, y: usize },
    Jump,
}
