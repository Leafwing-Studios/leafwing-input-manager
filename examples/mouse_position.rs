use bevy::{ecs::entity, input::InputSystem, prelude::*, window::PrimaryWindow};
use leafwing_input_manager::{
    axislike::DualAxisData, plugin::InputManagerSystem, prelude::*, systems::run_if_enabled,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<BoxMovement>::default())
        .add_startup_system(setup)
        .add_system(
            update_cursor_state_from_window::<BoxMovement>
                .run_if(run_if_enabled::<BoxMovement>)
                .in_base_set(CoreSet::PreUpdate)
                .in_set(InputManagerSystem::ManualControl)
                .before(InputManagerSystem::ReleaseOnDisable)
                .after(InputManagerSystem::Tick)
                // Must run after the system is updated from inputs, or it will be forcibly released due to the inputs
                // not being pressed
                .after(InputManagerSystem::Update)
                .after(InputSystem),
        )
        .add_system(pan_camera)
        .run();
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum BoxMovement {
    Pan,
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
        action: BoxMovement::Pan,
        targets: entity.into(),
    });
}

fn update_cursor_state_from_window<A: Actionlike>(
    window_query: Query<(&Window, &ActionStateDriver<A>)>,
    mut action_state_query: Query<&mut ActionState<A>>,
) {
    for (window, driver) in window_query.iter() {
        for entity in driver.targets.iter() {
            let mut action_state = action_state_query
                .get_mut(*entity)
                .expect("Entity does not exist, or does not have an `ActionState` component");

            if let Some(val) = window.cursor_position() {
                action_state
                    .action_data_mut(driver.action.clone())
                    .axis_pair = Some(DualAxisData::from_xy(val));
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
        .axis_pair(BoxMovement::Pan)
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor.xy()))
        .map(|ray| ray.origin.truncate())
    {
        // Because we're moving the camera, not the object, we want to pan in the opposite direction.
        // However, UI coordinates are inverted on the y-axis, so we need to flip y a second time.
        box_transform.translation.x = box_pan_vector.x;
        box_transform.translation.y = box_pan_vector.y;
    }
}
