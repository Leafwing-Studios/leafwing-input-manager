use bevy::{ecs::system::ResMut, input::InputPlugin, prelude::*};
use leafwing_input_manager::{
    clashing_inputs::BasicInputs,
    prelude::{updating::InputRegistration, *},
    user_input::updating::{CentralInputStore, UpdatableInput},
    InputControlKind,
};
use serde::{Deserialize, Serialize};

#[derive(Resource, Default, Debug)]
pub struct MidiControllerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct MidiAxis(u8);

impl UserInput for MidiAxis {
    fn kind(&self) -> InputControlKind {
        InputControlKind::Axis
    }

    fn decompose(&self) -> BasicInputs {
        BasicInputs::None
    }
}

impl Axislike for MidiAxis {
    fn get_value(&self, input_store: &CentralInputStore, _entity: Entity) -> Option<f32> {
        // Read the axis value from the central input store
        input_store.value(self)
        // 0.5
    }
}

impl UpdatableInput for MidiAxis {
    type SourceData = Res<'static, MidiControllerState>;

    fn compute(
        mut central_input_store: ResMut<CentralInputStore>,
        _source_data: bevy::ecs::system::StaticSystemParam<Self::SourceData>,
    ) {
        // Update only the second axis input for demonstration
        central_input_store.update_axislike(MidiAxis(1), 0.5);
    }
}

#[derive(Actionlike, Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
enum Action {
    #[actionlike(Axis)]
    Axis0,
    #[actionlike(Axis)]
    Axis1,
    #[actionlike(Axis)]
    Axis2,
}

#[test]
fn axis_data_should_be_none_before_update_axislike() {
    let mut app = App::new();

    app.init_resource::<MidiControllerState>();

    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(InputManagerPlugin::<Action>::default());

    // Register a new input kind (custom axis input)
    app.register_input_kind::<MidiAxis>(InputControlKind::Axis);

    // Insert a default ActionState resource
    app.init_resource::<ActionState<Action>>();

    // Add an InputMap binding for the custom axis input
    app.insert_resource(
        InputMap::default()
            .with_axis(Action::Axis0, MidiAxis(0))
            .with_axis(Action::Axis1, MidiAxis(1)),
        // Note: Axis2 is not in the input map
    );

    app.update();

    // Before any update, axis_data should be None
    let action_state = app.world().resource::<ActionState<Action>>();

    // not in input map and not updated, should definitely be None
    assert_eq!(action_state.axis_data(&Action::Axis2), None);

    // in input map and updated, should be Some
    assert!(action_state.axis_data(&Action::Axis1).is_some());
    assert_eq!(action_state.axis_data(&Action::Axis1).unwrap().value, 0.5);

    // in input map but not updated, should be None
    assert_eq!(action_state.axis_data(&Action::Axis0), None);
}
