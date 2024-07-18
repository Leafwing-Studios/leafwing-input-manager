use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<BoxMovement>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, pan_camera)
        .run();
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum BoxMovement {
    MousePosition,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(SpriteBundle {
            transform: Transform::from_scale(Vec3::new(100., 100., 1.)),
            ..default()
        })
        .insert(InputManagerBundle::<BoxMovement>::default());
}

fn pan_camera(
    mut query: Query<(&mut Transform, &ActionState<BoxMovement>)>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let (mut box_transform, action_state) = query.single_mut();
    let (camera, camera_transform) = camera.single();

    // Note: Nothing is stopping us from doing this in the action update system instead!
    let cursor_movement = action_state.axis_pair(&BoxMovement::MousePosition);
    let ray = camera
        .viewport_to_world(camera_transform, cursor_movement.xy())
        .unwrap();
    let box_pan_vector = ray.origin.truncate();

    box_transform.translation.x = box_pan_vector.x;
    box_transform.translation.y = box_pan_vector.y;
}
