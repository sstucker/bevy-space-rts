use bevy::prelude::*;

use crate::Map;

// TODO parameters
const CAMERA_DRAG: f32 = 0.93;  // cam_v_t = cam_v_t-1 * CAMERA_DRAG
const MIN_ZOOM_SCALE: f32 = 0.1;
const MAX_ZOOM_SCALE: f32 = 6.;

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
    commands.spawn_bundle(OrthographicCameraBundle::new_2d())
		.insert(OrthographicVelocity { ..Default::default() });
}

fn camera_move_system(
    kb: Res<Input<KeyCode>>,
    mut q_map: Query<&Map, With<Map>>,
    mut query: Query<(&mut OrthographicProjection, &mut Transform, &mut OrthographicVelocity), With<Camera>>,
) {
    if let Ok((mut projection, mut cam_transform, mut cam_velocity)) = query.get_single_mut() {
        let map = q_map.single();
        
        // Camera drag
        cam_velocity.dx *= CAMERA_DRAG;
        cam_velocity.dy *= CAMERA_DRAG;
        cam_velocity.dz *= CAMERA_DRAG;
        // Change velocity
        cam_velocity.dx +=
        if kb.pressed(KeyCode::Left) && (cam_transform.translation.x > -map.w as f32 / 2.) {
			-1.
		} else if kb.pressed(KeyCode::Right) && (cam_transform.translation.x < map.w as f32 / 2.) {
			1.
		} else {
			0.
		};
        cam_velocity.dy +=
        if kb.pressed(KeyCode::Up) && (cam_transform.translation.y < map.h as f32 / 2.) {
			1.
		} else if kb.pressed(KeyCode::Down) && (cam_transform.translation.y > -map.h as f32 / 2.) {
			-1.
		} else {
			0.
		};
        cam_velocity.dz +=
        if kb.pressed(KeyCode::PageUp) || kb.pressed(KeyCode::Plus) {
			0.005
		} else if kb.pressed(KeyCode::PageDown) || kb.pressed(KeyCode::Minus) {
			-0.005
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
            let mut log_zoom = projection.scale.ln();
            log_zoom += cam_velocity.dz;
            projection.scale = if log_zoom.exp() > MAX_ZOOM_SCALE {
                MAX_ZOOM_SCALE
            } else if log_zoom.exp() < MIN_ZOOM_SCALE {
                MIN_ZOOM_SCALE
            } else {
                log_zoom.exp()
            };
            println!("Zoom level is {}", projection.scale);
        }
        // TODO rotation?
    }
}
