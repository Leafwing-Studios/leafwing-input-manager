use bevy::{input::InputSystem, prelude::*, window::PrimaryWindow};
use leafwing_input_manager::{
    axislike::DualAxisData, plugin::InputManagerSystem, prelude::*, systems::run_if_enabled,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<BoxMovement>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            update_cursor_state_from_window
                .run_if(run_if_enabled::<BoxMovement>)
                .in_set(InputManagerSystem::ManualControl)
                .before(InputManagerSystem::ReleaseOnDisable)
                .after(InputManagerSystem::Tick)
                .after(InputManagerSystem::Update)
                .after(InputSystem),
        )
        .add_systems(Update, pan_camera)
        .run();
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum BoxMovement {
    MousePosition,
}

fn setup(mut commands: Commands, window: Query<Entity, With<PrimaryWindow>>) {
    commands.spawn(Camera2dBundle::default());

    let entity = commands
        .spawn(SpriteBundle {
            transform: Transform::from_scale(Vec3::new(100., 100., 1.)),
            ..default()
        })
        .insert(InputManagerBundle::<BoxMovement>::default())
        // Note: another entity will be driving this input
        .id();

    commands.entity(window.single()).insert(ActionStateDriver {
        action: BoxMovement::MousePosition,
        targets: entity.into(),
    });
}

fn update_cursor_state_from_window(
    window_query: Query<(&Window, &ActionStateDriver<BoxMovement>)>,
    mut action_state_query: Query<&mut ActionState<BoxMovement>>,
) {
    // Update each actionstate with the mouse position from the window
    // by using the referenced entities in ActionStateDriver and the stored action as
    // a key into the action data
    for (window, driver) in window_query.iter() {
        for entity in driver.targets.iter() {
            let mut action_state = action_state_query
                .get_mut(*entity)
                .expect("Entity does not exist, or does not have an `ActionState` component");

            if let Some(val) = window.cursor_position() {
                action_state.action_data_mut(driver.action).axis_pair =
                    Some(DualAxisData::from_xy(val));
            }
        }
    }
}

fn pan_camera(
    mut query: Query<(&mut Transform, &ActionState<BoxMovement>)>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (mut box_transform, action_state) = query.single_mut();
    let (camera, camera_transform) = camera.single();

    // Note: Nothing is stopping us from doing this in the action update system instead!
    if let Some(box_pan_vector) = action_state
        .axis_pair(BoxMovement::MousePosition)
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor.xy()))
        .map(|ray| ray.origin.truncate())
    {
        box_transform.translation.x = box_pan_vector.x;
        box_transform.translation.y = box_pan_vector.y;
    }
}
