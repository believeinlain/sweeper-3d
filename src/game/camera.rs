use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::{GameState, InputEvent, input::ScreenPosition};

use super::GamePiece;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // Add Camera systems
        app.add_systems(OnEnter(GameState::GameStart), spawn.after(super::cleanup));
        app.add_systems(Update, camera_controls.run_if(GameState::in_game()));
        app.add_event::<RayEvent>();
        app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)));
        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, cursor_ray_gizmo.run_if(GameState::playable()));
    }
}

#[derive(Component)]
pub struct MainCamera {
    zoom_speed: f32,
    zoom_limit_near: f32,
    zoom_limit_far: f32,
}
impl Default for MainCamera {
    fn default() -> Self {
        Self {
            zoom_speed: 1.0,
            zoom_limit_near: 1.0,
            zoom_limit_far: 20.0,
        }
    }
}

#[derive(Message)]
pub enum RayEvent {
    ClearBlock(Ray3d),
    MarkBlock(Ray3d),
}

pub(super) fn spawn(mut commands: Commands) {
    let zoom = 10.0;
    let translation = Vec3::ONE.normalize() * zoom;

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera::default(),
        GamePiece,
    ));
}

pub(super) fn camera_controls(
    mut input_events: MessageReader<InputEvent>,
    mut camera_transform: Query<(&Camera, &MainCamera, &mut Transform)>,
    mut ray_events: MessageWriter<RayEvent>,
) {
    let (camera, main_camera, mut transform) = camera_transform.single_mut().unwrap();
    for input_event in input_events.read() {
        match input_event {
            InputEvent::RotateCamera { delta } => {
                let delta_x = delta.x * TAU;
                let delta_y = delta.y * PI;
                // Rotate around local X axis and global Y axis
                let camera_tilt = transform.up().dot(Vec3::Y);
                log::debug!("Camera tilt: {camera_tilt}");
                let x_rot = Quat::from_axis_angle(
                    Vec3::Y,
                    if camera_tilt > 0.0 { -delta_x } else { delta_x },
                );
                let y_rot = Quat::from_axis_angle(*transform.local_x(), -delta_y);
                transform.rotate_around(Vec3::ZERO, x_rot);
                transform.rotate_around(Vec3::ZERO, y_rot);
                // Fix the odd tilt we get sometimes
                let camera_tilt = transform.up().dot(Vec3::Y);
                if camera_tilt > 0.01 {
                    *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
                }
                if camera_tilt < -0.01 {
                    *transform = transform.looking_at(Vec3::ZERO, Vec3::NEG_Y);
                }
            }
            InputEvent::ZoomCamera { delta } => {
                let zoom = *delta * transform.translation * main_camera.zoom_speed * -0.1;
                let new_translation = transform.translation + zoom;
                let zoom_dist = new_translation.distance(Vec3::ZERO);
                if zoom_dist > main_camera.zoom_limit_near && zoom_dist < main_camera.zoom_limit_far
                {
                    *transform = transform.with_translation(new_translation);
                }
            }
            InputEvent::ClearBlock(cursor_pos) => {
                if let Some(ray) = get_cursor_ray(camera, &transform, *cursor_pos) {
                    log::debug!("Send RayEvent::ClearBlock");
                    ray_events.write(RayEvent::ClearBlock(ray));
                }
            }
            InputEvent::MarkBlock(cursor_pos) => {
                if let Some(ray) = get_cursor_ray(camera, &transform, *cursor_pos) {
                    log::debug!("Send RayEvent::MarkBlock");
                    ray_events.write(RayEvent::MarkBlock(ray));
                }
            }
            _ => {}
        }
    }
}

fn get_cursor_ray(
    camera: &Camera,
    camera_trans: &Transform,
    cursor_pos: ScreenPosition,
) -> Option<Ray3d> {
    let view = camera_trans.to_matrix();

    let viewport = camera.logical_viewport_rect()?;
    let screen_size = camera.logical_target_size()?;
    let invert_cursor_pos = Vec2::new(cursor_pos.x, screen_size.y - cursor_pos.y);
    let adj_cursor_pos =
        invert_cursor_pos - Vec2::new(viewport.min.x, screen_size.y - viewport.max.y);

    let projection = camera.clip_from_view();
    let far_ndc = projection.project_point3(Vec3::NEG_Z).z;
    let near_ndc = projection.project_point3(Vec3::Z).z;
    let cursor_ndc = (adj_cursor_pos / viewport.size()) * 2.0 - Vec2::ONE;
    let ndc_to_world: Mat4 = view * projection.inverse();
    let near = ndc_to_world.project_point3(cursor_ndc.extend(near_ndc));
    let far = ndc_to_world.project_point3(cursor_ndc.extend(far_ndc));
    let ray_direction = Dir3::new(far - near).unwrap();
    Some(Ray3d::new(near, ray_direction))
}

#[cfg(feature = "debug-draw")]
fn cursor_ray_gizmo(
    mut gizmos: Gizmos,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    primary_window: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let Some(cursor_pos) = primary_window.single().cursor_position() else {
        return;
    };
    let (camera, camera_trans) = main_camera.single();
    let Some(ray) = get_cursor_ray(camera, camera_trans, cursor_pos) else {
        return;
    };
    gizmos.arrow(ray.origin, ray.get_point(20.0), Color::YELLOW);
}
