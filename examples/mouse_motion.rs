use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<CameraMovement>::default())
        .add_startup_system(setup)
        .add_system(pan_camera)
        .run()
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum CameraMovement {
    Pan,
}

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert_bundle(InputManagerBundle::<CameraMovement> {
            input_map: InputMap::default()
                // This will capture the total continous value, for direct use
                // Note that you can also use discrete gesture-like motion, via the `MouseMotionDirection` enum
                .insert(DualAxis::mouse_motion(), CameraMovement::Pan)
                .build(),
            ..default()
        });

    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform::from_scale(Vec3::new(100., 100., 1.)),
        ..default()
    });
}

fn pan_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<Camera2d>>) {
    const CAMERA_PAN_RATE: f32 = 0.5;

    let (mut camera_transform, action_state) = query.single_mut();

    let camera_pan_vector = action_state.axis_pair(CameraMovement::Pan).unwrap();

    // Because we're moving the camera, not the object, we want to pan in the opposite direction
    camera_transform.translation.x -= CAMERA_PAN_RATE * camera_pan_vector.x();
    camera_transform.translation.y -= CAMERA_PAN_RATE * camera_pan_vector.y();
}
