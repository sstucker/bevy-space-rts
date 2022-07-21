#[allow(unused_mut)]
#[allow(unused)]
#[allow(dead_code)]

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}};
use bevy::core::FixedTimestep;
use bevy_prototype_lyon::prelude::*;

use std::{sync::atomic::{AtomicU8, Ordering}, fmt::{self, Debug}, time::Duration, u32::MAX};

pub mod components;
pub use components::*;

// Package level variables
static NUMBER_OF_OWNERS: AtomicU8 = AtomicU8::new(0);
static NUMBER_OF_UNITS: AtomicU8 = AtomicU8::new(0);

const MIN_ZOOM_SCALE: f32 = 0.1;
const MAX_ZOOM_SCALE: f32 = 6.;

const UI_ZORDER: f32 = 20.;
const UNIT_ZORDER: f32 = 10.;
const WORLD_ZORDER: f32 = 0.;

const MAP_W: i32 = 1200;
const MAP_H: i32 = 1200;

const SPRITE_SCALE: f32 = 0.05;

const USER_ID: u8 = 0;

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
            .add_startup_system_to_stage(StartupStage::PostStartup, map_system)
            .add_system_set(SystemSet::new()  // Unit updates
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(unit_hover_system)
                .with_system(unit_movement_system)
            )
            .add_system_set(SystemSet::new() // Input 
                // .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(input_mouse_system)
            )
            // Graphics
            .add_system(ui_highlight_selected_system)
            // Mechanics
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
    asset_server: Res<AssetServer>,
) {
    for ev in ev_spawn.iter() {
        println!("Spawning unit owned by Owner {}", ev.owner.id);
        let mut ec = commands.spawn();
        ec.insert(Unit {
            name: ev.unit_type.to_string(),
            owner: ev.owner.clone(),
            id:  NUMBER_OF_UNITS.fetch_add(1, Ordering::Relaxed),
        });
        ec.insert( Velocity { ..Default::default() } );
        match &ev.unit_type {
        UnitType::DefaultUnit => {
            let unit_size = Vec2::new(762., 1350.);
            ec.insert(Hp { max: 100, current: 100 } );
            ec.insert( Body::new(ev.position, unit_size) );
            ec.insert( UnitControls { ..Default::default() } );
            ec.insert_bundle( TransformBundle {
                local: Transform {
                    translation: Vec3::new( ev.position.x, ev.position.y, UNIT_ZORDER ),
                    scale: Vec3::ONE * SPRITE_SCALE,
                    rotation: Quat::from_rotation_z((std::f32::consts::PI / 2.) * ev.position.z ),
                    ..Default::default()
                },
                ..Default::default()
            });
            // Sprites
            ec.with_children(|parent| {
                
                // Debug sprites
                parent.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(1., 0., 0., 0.3),
                        custom_size: Some(unit_size),
                        ..Default::default()
                    },
                    transform: Transform { translation: Vec3::new(0., 0., -1.), ..Default::default() },
                    ..Default::default()
                }).insert(DebugRect);
                
                parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                    sides: 30,
                    feature: shapes::RegularPolygonFeature::Radius((unit_size[0] + unit_size[1]) / 4.),
                    ..shapes::RegularPolygon::default()
                },
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::rgba(0., 1., 0., 0.5)),
                    outline_mode: StrokeMode::new(Color::rgba(0., 1., 0., 0.5), 2.),
                },
                Transform { translation: Vec3::new(0., 0., -2.), ..Default::default() },
                )).insert(DebugSelectionRadius);

                // Sprites
                parent.spawn_bundle(SpriteBundle {
                    texture: asset_server.load("ship01.png"),
                    transform: Transform { translation: Vec3::new(0., 0., 0.), ..Default::default() },
                    ..Default::default()
                });

            });
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
    mut query: Query<(&Unit, &mut Transform, &mut Body ,&mut Velocity), With<Velocity>>,
    time: Res<Time>
) {
}

fn unit_hover_system(
    mut query: Query<(&Unit, &mut Transform, &Body, &Velocity), With<Velocity>>,
    time: Res<Time>
) {
    for (unit, mut transform, body, vel) in query.iter_mut() {
        // Randomly oscillate units around by some percentage of their size
        let t = time.seconds_since_startup() as f32;
        let second_order: f32 = f32::sin(t * 0.1 + 8. * unit.id as f32);
        transform.translation.x += SPRITE_SCALE * 0.0001 * body.size.y * f32::sin(t + 10. * unit.id as f32) * second_order;
        transform.translation.y += SPRITE_SCALE * 0.0001 * body.size.x * f32::sin(t + 5. * unit.id as f32) * second_order;
        // println!("Moving unit {} + {}, {} + {}", body.position.x, vel.dx, body.position.y, vel.dy);
    }
}

fn kill_system(
    query: Query<(&Unit, &Hp), With<Hp>>
) {
    for (unit, hp) in query.iter() {
        // eprintln!("Entity {}{} is owned by {} and has {} HP.", unit.name, unit.id, unit.owner.id, hp.current);
    }
}

fn input_mouse_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    kb: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut ui: ResMut<Ui>,
    mut q_units: Query<(&mut UnitControls, &Transform, &Body, &Unit), With<Unit>>,
    q_camera: Query<(&OrthographicProjection, &Camera, &GlobalTransform), With<Camera>>,
    q_rect: Query<Entity, With<SelectionRect>>,
    q_map: Query<&Map, With<Map>>,
) {
    // Delete any existing selection rect UI 
    for rect in q_rect.iter() {
        commands.entity(rect).despawn();
    }
    
    // On click
    if mb.pressed(MouseButton::Left)
    || mb.just_released(MouseButton::Left)
    || mb.pressed(MouseButton::Right)
    || mb.just_released(MouseButton::Right)
     {  
        let window = windows.get_primary().unwrap();
        let map = q_map.single();
        if let Some(w_pos) = window.cursor_position() {  // If cursor is in window
            // Convert cursor position to world position
            let (projection, camera, camera_transform) = q_camera.single();
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
            
            // If drawing selection rectangle or clicking a unit
            if mb.pressed(MouseButton::Left) {
                
                if ui.held_position[0].is_nan() {
                    ui.held_position = m_pos;
                    println!("Held position is {}, {}", ui.held_position[0], ui.held_position[1]);
                }

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

            // If selection box needs to be evaluated
            else if mb.just_released(MouseButton::Left) {
                let shift: bool = kb.pressed(KeyCode::RShift) || kb.pressed(KeyCode::LShift);
                let mut clicked_unit = false;
                // If we clicked a unit, select it instead of evaluating the rectangle
                for (mut controls, _, body, unit) in q_units.iter_mut() {
                    if ((m_pos - body.position.truncate()).length() < body.selection_radius) && unit.owner.id == USER_ID {  // TODO multiplayer
                        println!("Clicked Unit {} {} away", unit.id, (m_pos - body.position.truncate()).length());
                        if shift {
                            controls.is_selected = ! controls.is_selected;
                        } else {
                            controls.is_selected = true;
                        }
                        clicked_unit = true;
                    }
                    else {
                        if !shift {
                            controls.is_selected = false;
                        }
                    }
                }
                if clicked_unit { // Do not evaluate rectangle if a unit was clicked
                    ui.held_position = Vec2::new(f32::NAN, f32::NAN);
                    return; 
                }  

                let bb: Vec4 = Vec4::new(
                    ui.held_position[0].min(m_pos[0]),
                    ui.held_position[1].max(m_pos[1]),
                    ui.held_position[0].max(m_pos[0]),
                    ui.held_position[1].min(m_pos[1]),
                );
                println!("Box evaluated! ({}, {}), ({}, {})", bb.x, bb.y, bb.z, bb.w);
                for (mut controls, _, body, unit) in q_units.iter_mut() {
                    if (bb.x <= body.position.x && body.position.x <= bb.z) && (bb.y >= body.position.y && body.position.y >= bb.w) {
                        println!("Unit {} at {}, {} in bounding box.", unit.id, body.position.x, body.position.x);
                        if unit.owner.id == USER_ID { // TODO multiplayer
                            println!("Unit {} is now selected!", unit.owner.id);
                            if shift {
                                controls.is_selected = ! controls.is_selected;
                            } else {
                                controls.is_selected = true;
                            }
                        }  
                    }
                    else {
                        if !shift {
                            controls.is_selected = false;
                        }
                    }
                }
                ui.held_position = Vec2::new(f32::NAN, f32::NAN);
            }
        }
    }
}

fn map_system(
    mut commands: Commands,
	asset_server: Res<AssetServer>,
	q_camera: Query<&OrthographicProjection, With<Camera>>,
) {
    let map: Map = Map { w: MAP_W, h: MAP_H };
	let mut ec = commands.spawn();
    ec.insert(map);

    // Draw map grid
	let projection = q_camera.single();
    
    ec.with_children(|parent| {
        let mut draw_gridline = |p1: Vec2, p2: Vec2| {
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(p1);
            path_builder.line_to(p2);
            let line = path_builder.build();
            parent.spawn_bundle(GeometryBuilder::build_as(
                &line,
                DrawMode::Stroke(StrokeMode::new(
                    Color::rgba(1., 1., 1., 0.2),
                    1.0 * projection.scale  // Always draw the same thickness of UI elements regardless of zoom
                )),
                Transform { translation: Vec3::new(0., 0., WORLD_ZORDER), ..Default::default() },
            )).insert( GridLine );
        };
        for y in (-map.h / 2..map.h / 2).step_by(100) {
            draw_gridline(Vec2::new(-MAP_W as f32 / 2., y as f32), Vec2::new(MAP_W as f32 / 2., y as f32));
        }
        for x in (-map.w / 2..map.w / 2).step_by(100) {
            draw_gridline(Vec2::new(x as f32, -MAP_H as f32 / 2.), Vec2::new(x as f32, MAP_H as f32 / 2.));
        }
    });
}


fn ui_highlight_selected_system(
    mut commands: Commands,
    q_circ: Query<Entity, With<SelectedCircle>>,
    q_units: Query<(Entity, &UnitControls, &Body, &Unit), With<Unit>>,
    q_camera: Query<&OrthographicProjection, With<Camera>>,
) {
    for circ in q_circ.iter() {
        commands.entity(circ).despawn();
    }
    let projection = q_camera.single();
    for (entity, controls, body, unit) in q_units.iter() {
        if controls.is_selected {
            let mut ec = commands.entity(entity);
            ec.with_children(|parent| {
                parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                    sides: 30,
                    feature: shapes::RegularPolygonFeature::Radius((body.size[0] + body.size[1]) / 3.),
                    ..shapes::RegularPolygon::default()
                },
                DrawMode::Outlined {
                    fill_mode: FillMode::color(Color::rgba(0., 0., 0., 0.)),
                    outline_mode: StrokeMode::new(Color::GREEN, 20. * projection.scale),
                },
                Transform { translation: Vec3::new(0., 0., UI_ZORDER), ..Default::default() },
            )).insert(SelectedCircle);
            });
        }
    }
}

