#[allow(unused_mut)]
#[allow(unused)]
#[allow(dead_code)]

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}};
use bevy_prototype_lyon::prelude::*;

use std::{sync::atomic::{AtomicU8, Ordering}, fmt::{self}, time::Duration};

pub mod components;
pub use components::*;

// Package level variables
static NUMBER_OF_OWNERS: AtomicU8 = AtomicU8::new(0);
static NUMBER_OF_UNITS: AtomicU8 = AtomicU8::new(0);

const UNIT_ZORDER: f32 = 10.;
const SCALE: f32 = 1.;

type WindowSize = Vec2;

// -- KinematicCameraPlugin --------------------------------------------

const CAMERA_DRAG: f32 = 0.93;  // cam_v_t = cam_v_t-1 * CAMERA_DRAG

pub struct KinematicCameraPlugin;

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
    map: Res<Map>,
    mut query: Query<(&mut OrthographicProjection, &mut Transform, &mut OrthographicVelocity), With<Camera>>,
) {
    if let Ok((mut projection, mut cam_transform, mut cam_velocity)) = query.get_single_mut() {
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
        if kb.pressed(KeyCode::PageUp) {
			0.005
		} else if kb.pressed(KeyCode::PageDown) {
			-0.005
		} else {
			0.
		};
        // Transform camera
        if cam_velocity.dx.abs() > 0.01 {
            cam_transform.translation.x += (cam_velocity.dx * projection.scale);
        }
        if cam_velocity.dy.abs() > 0.01 {
            cam_transform.translation.y += (cam_velocity.dy * projection.scale);
        }
        // Zoom
        if cam_velocity.dz.abs() > 0.00001 {
            let mut log_zoom = projection.scale.ln();
            log_zoom += cam_velocity.dz;
            projection.scale = log_zoom.exp();
        }
        // TODO rotation?
    }
}

// -- UnitPlugin --------------------------------------------

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        // app.insert_resource(PlayerState::default())
        //     .add_system_set(
        //         SystemSet::new()
        //             .with_run_criteria(FixedTimestep::step(0.5))
        //             .with_system(player_spawn_system),
        //     )
        //     .add_system(player_keyboard_event_system)
        //     .add_system(player_fire_system);
        app
            .insert_resource(Msaa { samples: 4 })
            .add_plugin(ShapePlugin)
            .add_event::<SpawnUnitEvent>()
            .add_startup_system(startup_system)
            .add_system(kill_system)
            .add_system(unit_movement_system)
            .add_system(input_unit_select_system)
            .add_system(spawn_units_system);
    }
}

// Available units and their names
pub enum UnitType {
    DefaultUnit,
    Tank,
    Plane,
    Building,
}

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnitType::DefaultUnit => write!(f, "DefaultUnit"),
            UnitType::Tank => write!(f, "Tank"),
            UnitType::Plane => write!(f, "Plane"),
            UnitType::Building => write!(f, "Building"),
        }
    }
}

// An owner of a Unit in Konquer
#[derive(Clone)]
pub struct Owner {
    id: u8,
}

impl Owner {
    pub fn new() -> Owner {
        // Create a unique Owner ID each time new is called
        Owner {
            id: NUMBER_OF_OWNERS.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub struct SpawnUnitEvent {
    unit_type: UnitType,
    owner: Owner,
    position: Vec3,
}

impl SpawnUnitEvent {
    pub fn new(unit_type: UnitType, owner: Owner, position: Vec3) -> SpawnUnitEvent {
        SpawnUnitEvent { unit_type: unit_type, owner: owner, position: position}
    }
}

pub struct Ui {
    pub held_position: Vec2,
    pub selected_units: Vec<u8>,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            held_position: Vec2::new(f32::NAN, f32::NAN),
            selected_units: Vec::new(),
        }
    }
}

fn startup_system(
    mut commands: Commands,
    windows: Res<Windows>,
) {
	commands.insert_resource( Ui { ..Default::default() } );
    let window = windows.get_primary().unwrap();
    let window_size = WindowSize::new(window.width(), window.height());
	commands.insert_resource(window_size);
}

// Master decoder of units and their properties. TODO convert to table
fn spawn_units_system(
    mut ev_spawn: EventReader<SpawnUnitEvent>,
    mut commands: Commands,
) {
    for ev in ev_spawn.iter() {
        println!("Spawning unit owned by Owner {}", ev.owner.id);
        let mut ec = commands.spawn();
        ec.insert(Unit {
            name: ev.unit_type.to_string(),
            owner: ev.owner.clone(),
            id:  NUMBER_OF_UNITS.fetch_add(1, Ordering::Relaxed),
        });
        ec.insert( Position { x: ev.position.x, y: ev.position.y, w: ev.position.z } );
        ec.insert( Velocity { ..Default::default() } );
        println!("Drawing SVG {}", std::fs::read_to_string("konquer/assets/path01.svg").unwrap());
        match &ev.unit_type {
        UnitType::DefaultUnit => {
            ec.insert(Hp { max: 100, current: 100 } );
            ec.insert( UnitControls { ..Default::default() } );
            ec.insert_bundle(
                GeometryBuilder::build_as(
                    // &shapes::RegularPolygon {
                    //     sides: 5,
                    //     feature: shapes::RegularPolygonFeature::Radius(10.0),
                    //     ..shapes::RegularPolygon::default()
                    // },
                    &shapes::SvgPathShape{
                        svg_path_string: String::from(std::fs::read_to_string("konquer/assets/path01.svg").unwrap()),
                        svg_doc_size_in_px: Vec2::new(20., 20.).to_owned()
                    },
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::rgba(0.5, 0.5, 0.5, 0.5)),
                        outline_mode: StrokeMode::new(Color::CYAN, 2.0),
                    },
                    Transform {
                        translation: Vec3::new(
                            ev.position.x,
                            ev.position.y,
                            UNIT_ZORDER,
                        ),
                        rotation: Quat::from_rotation_z(ev.position.z),
                        scale: Vec3::new(SCALE, SCALE, SCALE),
                        ..Default::default()
                    }
                )
            );
        }
        UnitType::Tank => {
            println!("Tank not spawned.\n");
        }
        UnitType::Building => {
            println!("Building not spawned.\n");
        }
        other => {
            println!("Unit not spawned.\n");
        }
        };
    }
}

fn unit_movement_system(
    query: Query<(&Transform, &Position,&Velocity), With<Velocity>>,
) {
    for (transform, pos, vel) in query.iter() {
        // println!("Moving unit {} + {}, {} + {}", pos.x, vel.dx, pos.y, vel.dy);
    }
}

fn kill_system(
    query: Query<(&Unit, &Hp), With<Hp>>
) {
    for (unit, hp) in query.iter() {
        // eprintln!("Entity {}{} is owned by {} and has {} HP.", unit.name, unit.id, unit.owner.id, hp.current);
    }
}

fn input_unit_select_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    map: Res<Map>,
    mut ui: ResMut<Ui>,
    query: Query<(&mut UnitControls, &Position, &Unit), With<Unit>>,
    q_camera: Query<(&OrthographicProjection, &Camera, &GlobalTransform), With<Camera>>,
    q_rect: Query<Entity, With<SelectionRect>>,
) {
    // Delete any existing selection rectangles
    for rect in q_rect.iter() {
        commands.entity(rect).despawn();
    }
    
    let window = windows.get_primary().unwrap();
    if mb.pressed(MouseButton::Left) {
        let (projection, camera, camera_transform) = q_camera.single();
        if let Some(w_pos) = window.cursor_position() {
            // Convert cursor position to world position
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let ndc: Vec2 = (w_pos / window_size) * 2. - Vec2::ONE;
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
            let mut m_pos: Vec2 = ndc_to_world.project_point3(ndc.extend(-1.)).truncate();
            
            // Prevent selection from exceeding bounds of the world
            if m_pos[0] < -map.w as f32 / 2. {
                m_pos[0] = -map.w as f32 / 2.;
            }
            else if m_pos[0] > map.w as f32 / 2. {
                m_pos[0] = map.w as f32 / 2.;
            }
            if m_pos[1] < -map.h as f32 / 2. {
                m_pos[1] = -map.h as f32 / 2.;
            }
            else if m_pos[1] > map.h as f32 / 2. {
                m_pos[1] = map.h as f32 / 2.;
            }

            if ui.held_position[0].is_nan() {
                ui.held_position = m_pos;
                println!("Held position is {}, {}", ui.held_position[0], ui.held_position[1]);
            }
            println!("Cursor position is {}, {}", m_pos[0], m_pos[1]);
            
            // Draw selection rect
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(ui.held_position);
            path_builder.line_to(Vec2::new(ui.held_position[0], m_pos[1]));
            path_builder.line_to(Vec2::new(m_pos[0], m_pos[1]));
            path_builder.line_to(Vec2::new(m_pos[0], ui.held_position[1]));
            path_builder.line_to(ui.held_position);
            let line = path_builder.build();
            commands.spawn_bundle(GeometryBuilder::build_as(
                &line,
                DrawMode::Stroke(StrokeMode::new(
                    Color::YELLOW,
                    2.0 * projection.scale  // Always draw the same thickness of UI elements regardless of zoom
                )),
                Transform { ..Default::default()  },
            )).insert( SelectionRect );
        }
    }
    else if mb.just_released(MouseButton::Left) {
        ui.held_position = Vec2::new(f32::NAN, f32::NAN);
    }
    for (controls, pos, unit) in query.iter() {
        // println!("Unit {}{} is owned by {} and is at {}, {}. Selected? {}", unit.name, unit.id, unit.owner.id, pos.x, pos.y, controls.is_selected);
    }

}

