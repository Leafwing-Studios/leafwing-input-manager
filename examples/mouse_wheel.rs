use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(InputManagerPlugin::<CameraMovement>::default())
        .add_startup_system(setup)
        .add_system(zoom_camera)
        .add_system(pan_camera.after(zoom_camera))
        .run()
}

#[derive(Actionlike, Clone, Debug, Copy, PartialEq, Eq)]
enum CameraMovement {
    Zoom,
    PanLeft,
    PanRight,
}

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(Camera2dBundle::default())
        .insert_bundle(InputManagerBundle::<CameraMovement> {
            input_map: InputMap::default()
                // This will capture the total continous value, for direct use
                .insert(SingleAxis::mouse_wheel_y(), CameraMovement::Zoom)
                // This will return a binary button-like output
                .insert(MouseWheelDirection::Left, CameraMovement::PanLeft)
                .insert(MouseWheelDirection::Right, CameraMovement::PanRight)
                // Alternatively, you could model this as a virtual Dpad,
                // which is extremely useful when you want to model 4-directional buttonlike inputs using the mouse wheel
                // .insert(VirtualDpad::mouse_wheel(), Pan)
                // Or even a continous `DualAxis`!
                // .insert(DualAxis::mouse_wheel(), Pan)
                .build(),
            ..default()
        });

    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform::from_scale(Vec3::new(100., 100., 1.)),
        ..default()
    });
}

fn zoom_camera(
    mut query: Query<(&mut OrthographicProjection, &ActionState<CameraMovement>), With<Camera2d>>,
) {
    const CAMERA_ZOOM_RATE: f32 = 0.05;

    let (mut camera_projection, action_state) = query.single_mut();
    // Here, we use the `action_value` method to extract the total net amount that the mouse wheel has travelled
    // Up and right axis movements are always positive by default
    let zoom_delta = action_state.value(CameraMovement::Zoom);

    // We want to zoom in when we use mouse wheel up
    // so we increase the scale proportionally
    // Note that the projections scale should always be positive (or our images will flip)
    camera_projection.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
}

fn pan_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<Camera2d>>) {
    const CAMERA_PAN_RATE: f32 = 10.;

    let (mut camera_transform, action_state) = query.single_mut();

    // When using the `MouseWheelDirection` type, mouse wheel inputs can be treated like simple buttons
    if action_state.pressed(CameraMovement::PanLeft) {
        camera_transform.translation.x -= CAMERA_PAN_RATE;
    }

    if action_state.pressed(CameraMovement::PanRight) {
        camera_transform.translation.x += CAMERA_PAN_RATE;
    }
}
