use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit}};

use crate::{Map, inputs::MouseOverEvent};

// TODO parameters
const CAMERA_DRAG: f32 = 0.8;  // cam_v_t = cam_v_t-1 * CAMERA_DRAG
const MIN_ZOOM_SCALE: f32 = 0.04;
const MAX_ZOOM_SCALE: f32 = 30.;
const LATERAL_MOVEMENT_SENS: f32 = 2.;
const ZOOM_SENS_LN: f32 = 1. / 16.;
const ZOOM_SENS_PX: f32 = 1. / 16.;

pub struct KinematicCameraPlugin;

#[derive(Component)]
pub struct OrthographicVelocity {
	pub dx: f32,
	pub dy: f32,
    pub dz: f32,
    pub dw: f32,  // Angular velocity
}

impl Default for OrthographicVelocity {
    fn default() -> Self {
        Self {
            dx: 0.,
            dy: 0.,
            dz: 0.,
            dw: 0.,
        }
    }
}


impl Plugin for KinematicCameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(camera_startup_system)
            .add_system(camera_move_system);
    }
}

fn camera_startup_system(
    mut commands: Commands,
) {
    commands.spawn_bundle(Camera2dBundle::default())
		.insert(OrthographicVelocity { ..Default::default() });
}

fn camera_move_system(
    kb: Res<Input<KeyCode>>,
    mut scrolls: EventReader<MouseWheel>,
    mut query: Query<(&Camera, &mut OrthographicProjection, &mut Transform, &mut OrthographicVelocity), With<Camera>>,
    q_map: Query<&Map, With<Map>>,
    windows: Res<Windows>,
    mb: Res<Input<MouseButton>>,
) {
    if let Ok((camera, mut projection, mut cam_transform, mut cam_velocity)) = query.get_single_mut() {
        let map = q_map.single();
        // Change velocity
        cam_velocity.dx +=
        if kb.pressed(KeyCode::Left) && (cam_transform.translation.x > 0.) {
			-LATERAL_MOVEMENT_SENS
		} else if kb.pressed(KeyCode::Right) && (cam_transform.translation.x < map.w as f32) {
			LATERAL_MOVEMENT_SENS
		} else {
			0.
		};
        cam_velocity.dy +=
        if kb.pressed(KeyCode::Up) && (cam_transform.translation.y < map.h as f32) {
			LATERAL_MOVEMENT_SENS
		} else if kb.pressed(KeyCode::Down) && (cam_transform.translation.y > 0.) {
			-LATERAL_MOVEMENT_SENS
		} else {
			0.
		};
        for ev in scrolls.iter() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    cam_velocity.dz -= ev.y * ZOOM_SENS_LN;
                }
                MouseScrollUnit::Pixel => {
                    cam_velocity.dz += ev.y * ZOOM_SENS_PX;
                }
            }
        }
        cam_velocity.dz +=
        if kb.pressed(KeyCode::PageUp) {
			0.008
		} else if kb.pressed(KeyCode::PageDown) {
			-0.008
		} else {
			0.
		};
        // Transform camera
        if cam_velocity.dx.abs() > 0.01 {
            cam_transform.translation.x += cam_velocity.dx * projection.scale;
        }
        if cam_velocity.dy.abs() > 0.01 {
            cam_transform.translation.y += cam_velocity.dy * projection.scale;
        }
        // Zoom
        if cam_velocity.dz.abs() > 0.00001 {
            let window = windows.get_primary().unwrap();
            let zoom = projection.scale * cam_velocity.dz.exp();
            let dz = zoom - projection.scale;
            if zoom > MAX_ZOOM_SCALE {
                projection.scale = MAX_ZOOM_SCALE
            } else if zoom < MIN_ZOOM_SCALE {
                projection.scale = MIN_ZOOM_SCALE
            } else {
                projection.scale = zoom;
                if let Some(w_pos) = window.cursor_position() {
                    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
                    let ndc: Vec2 = (w_pos / window_size) * 2. - Vec2::ONE;
                    if cam_velocity.dz < 0. {
                        let dwin = (window_size / 2.) * ndc * dz;
                        cam_transform.translation.x -= dwin.x;
                        cam_transform.translation.y -= dwin.y;
                    }
                }
            };
        }
        // Camera drag
        cam_velocity.dx *= CAMERA_DRAG;
        cam_velocity.dy *= CAMERA_DRAG;
        cam_velocity.dz *= CAMERA_DRAG;
        // TODO rotation?
    }
}
