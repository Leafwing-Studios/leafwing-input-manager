use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, pan_camera)
        .run()
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    Pan,
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::new([
        // This will capture the total continuous value, for direct use.
        // Note that you can also use discrete gesture-like motion,
        // via the `MouseMotionDirection` enum.
        (CameraMovement::Pan, DualAxis::mouse_motion()),
    ]);
    commands
        .spawn(Camera2dBundle::default())
        .insert(InputManagerBundle::with_map(input_map));

    commands.spawn(SpriteBundle {
        transform: Transform::from_scale(Vec3::new(100., 100., 1.)),
        ..default()
    });
}

fn pan_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<Camera2d>>) {
    const CAMERA_PAN_RATE: f32 = 0.5;

    let (mut camera_transform, action_state) = query.single_mut();

    let camera_pan_vector = action_state.axis_pair(&CameraMovement::Pan).unwrap();

    // Because we're moving the camera, not the object, we want to pan in the opposite direction.
    // However, UI coordinates are inverted on the y-axis, so we need to flip y a second time.
    camera_transform.translation.x -= CAMERA_PAN_RATE * camera_pan_vector.x();
    camera_transform.translation.y += CAMERA_PAN_RATE * camera_pan_vector.y();
}
