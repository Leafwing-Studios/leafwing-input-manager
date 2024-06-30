use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<CameraMovement>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, zoom_camera)
        .add_systems(Update, pan_camera.after(zoom_camera))
        .run();
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Reflect)]
enum CameraMovement {
    Zoom,
    Pan,
    PanLeft,
    PanRight,
}

impl Actionlike for CameraMovement {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            CameraMovement::Zoom => InputControlKind::Axis,
            CameraMovement::Pan => InputControlKind::DualAxis,
            CameraMovement::PanLeft | CameraMovement::PanRight => InputControlKind::Button,
        }
    }
}

fn setup(mut commands: Commands) {
    let input_map = InputMap::default()
        // This will capture the total continuous value, for direct use.
        .with_axis(CameraMovement::Zoom, MouseScrollAxis::Y)
        // This will return a binary button-like output.
        .with(CameraMovement::PanLeft, MouseScrollDirection::LEFT)
        .with(CameraMovement::PanRight, MouseScrollDirection::RIGHT)
        // Alternatively, you could model them as a continuous dual-axis input
        .with_dual_axis(CameraMovement::Pan, MouseScroll::default())
        // Or even a digital dual-axis input!
        .with_dual_axis(CameraMovement::Pan, MouseScroll::default().digital());
    commands
        .spawn(Camera2dBundle::default())
        .insert(InputManagerBundle::with_map(input_map));

    commands.spawn(SpriteBundle {
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
    let zoom_delta = action_state.value(&CameraMovement::Zoom);

    // We want to zoom in when we use mouse wheel up,
    // so we increase the scale proportionally
    // Note that the projection's scale should always be positive (or our images will flip)
    camera_projection.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
}

fn pan_camera(mut query: Query<(&mut Transform, &ActionState<CameraMovement>), With<Camera2d>>) {
    const CAMERA_PAN_RATE: f32 = 10.;

    let (mut camera_transform, action_state) = query.single_mut();

    // When using the `MouseScrollDirection` type, mouse wheel inputs can be treated like simple buttons
    if action_state.pressed(&CameraMovement::PanLeft) {
        camera_transform.translation.x -= CAMERA_PAN_RATE;
    }

    if action_state.pressed(&CameraMovement::PanRight) {
        camera_transform.translation.x += CAMERA_PAN_RATE;
    }
}
