use std::ops::Div;

use bevy::{prelude::*, input::mouse::{MouseWheel, MouseScrollUnit}};

use crate::{Map, inputs::MouseOverEvent, Background, BACKGROUND_ZORDER};

// TODO parameters
const CAMERA_DRAG: f32 = 0.8;  // cam_v_t = cam_v_t-1 * CAMERA_DRAG
const MIN_ZOOM_SCALE: f32 = 0.04;
const MAX_ZOOM_SCALE: f32 = 60.;
const LATERAL_MOVEMENT_SENS: f32 = 2.;
const ZOOM_SENS_LN: f32 = 1. / 16.;
const ZOOM_SENS_PX: f32 = 1. / 16.;

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

pub fn camera_startup_system(
    mut commands: Commands,
) {
    commands.spawn_bundle(Camera2dBundle::default())
		.insert(OrthographicVelocity { ..Default::default() });
}

pub fn camera_move_system(
    kb: Res<Input<KeyCode>>,
    mut scrolls: EventReader<MouseWheel>,
    mut q_camera: Query<(&Camera, &mut OrthographicProjection, &mut Transform, &mut OrthographicVelocity), With<Camera>>,
    mut q_background: Query<(&mut Transform, &Background), (With<Background>, Without<Camera>)>,
    q_map: Query<&Map, With<Map>>,
    windows: Res<Windows>,
    mb: Res<Input<MouseButton>>,
) {
    if let Ok((camera, mut projection, mut cam_transform, mut cam_velocity)) = q_camera.get_single_mut() {
        let map = q_map.single();
        let map_size = Vec2::new(map.w as f32, map.h as f32);
        // Change velocity
        cam_velocity.dx +=
        if kb.pressed(KeyCode::Left) && (cam_transform.translation.x > 0.) {
			-LATERAL_MOVEMENT_SENS
		} else if kb.pressed(KeyCode::Right) && (cam_transform.translation.x < map_size.x) {
			LATERAL_MOVEMENT_SENS
		} else {
			0.
		};
        cam_velocity.dy +=
        if kb.pressed(KeyCode::Up) && (cam_transform.translation.y < map_size.y) {
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
        // Move background with camera
        let cam_proportional_pos = cam_transform.translation.truncate().div(map_size) - 0.5;
        for (mut transform, background) in q_background.iter_mut() {
            transform.translation = (cam_transform.translation.truncate() + cam_proportional_pos * 400. * projection.scale * background.layer as f32)
            .extend(BACKGROUND_ZORDER + background.layer as f32);
            transform.scale = Vec3::new(1., 1., 0.) * projection.scale;
        }
    }
}
