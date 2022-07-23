
use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}};
use bevy::core::FixedTimestep;
use bevy_prototype_lyon::prelude::*;

use std::{sync::atomic::{AtomicU8, Ordering}, fmt::{self}};

pub mod components;
pub use components::*;

pub mod spawner;
use spawner::*;

pub mod inputs;
use inputs::InputPlugin;

pub mod camera;
pub use camera::*;

pub mod ui;
pub use ui::*;

// Package level variables
static NUMBER_OF_OWNERS: AtomicU8 = AtomicU8::new(0);

// TODO parameterize and IO
const UI_ZORDER: f32 = 20.;
const UNIT_ZORDER: f32 = 10.;
const WORLD_ZORDER: f32 = 0.;

const MAP_W: i32 = 1200;
const MAP_H: i32 = 1200;

const SPRITE_SCALE: f32 = 0.05;

const USER_ID: u8 = 0;

type WindowSize = Vec2;

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
            .add_plugin(InputPlugin)
            .add_event::<SpawnUnitEvent>()
            .add_startup_system(startup_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, map_system)
            .add_system_set(SystemSet::new()  // Unit updates
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(unit_hover_system)
                .with_system(unit_movement_system)
                .with_system(turret_track_and_fire_system)
                .with_system(turret_target_dispatcher)
                .with_system(unit_pathing_system)
            )
            .add_system_set(SystemSet::new() // Input 
                // .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(inputs::input_mouse_system)
            )
            // Graphics
            .add_system(ui_highlight_selected_system)
            .add_system(ui_show_path_system)
            // Mechanics
            .add_system(spawn_units_system);
    }
}

// An player of a Unit in Konquer
#[derive(Clone)]
pub struct Player {
    id: u8,
}

impl Player {
    pub fn new() -> Player {
        // Create a unique Player ID each time new is called
        Player {
            id: NUMBER_OF_OWNERS.fetch_add(1, Ordering::Relaxed),
        }
    }
}

pub struct SpawnUnitEvent {
    unit_type: UnitType,
    player: Player,
    position: Vec3,
}

impl SpawnUnitEvent {
    pub fn new(unit_type: UnitType, player: Player, position: Vec3) -> SpawnUnitEvent {
        SpawnUnitEvent { unit_type: unit_type, player: player, position: position}
    }
}

fn startup_system(
    mut commands: Commands,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = WindowSize::new(window.width(), window.height());
	commands.insert_resource(window_size);
}

// TODO add these as parameters for various units
const HEADING_THRESH_BURN: f32 = 0.8;  // Radians
const DRAG_LATERAL: f32 = 0.97;
const DRAG_RADIAL: f32 = 0.95;
const APPROACH_THRESHOLD_REAR: f32 = 100.;
const APPROACH_THRESHOLD_OMNI: f32 = 5.;
const THRESH_ARRIVAL: f32 = 5.;

fn unit_pathing_system(
    mut query: Query<(&mut UnitPath, &Body, &mut Velocity), With<UnitPath>>,
) {
    for (mut path, body, mut velocity ) in query.iter_mut() {
        if !path.path.is_empty() {  // For units with a destination
            let dist_to_dest = (path.path[0] - body.position.truncate()).length();
            let target = (path.path[0] - body.position.truncate()).normalize();
            let heading = Vec2::new(f32::cos(body.position.z), f32::sin(body.position.z));
            let cross = target.x * heading.y - target.y * heading.x;
            let err = 1. - heading.dot(target);
            velocity.dw += 
            if cross > 0.0 {
                -0.001 * err.min(1.).max(0.1)
            } else if cross < 0.0 {
                0.001 * err.min(1.).max(0.1)
            } else {
                0.
            };
        
            if cross.abs() < HEADING_THRESH_BURN {  // If we are close enough to the right heading to use rear thrusters
                // TODO get values from thrusters
                // Rear thrusters
                velocity.dx += (heading.x * 0.006) * (dist_to_dest / APPROACH_THRESHOLD_REAR).min(1.);
                velocity.dy += (heading.y * 0.006) * (dist_to_dest / APPROACH_THRESHOLD_REAR).min(1.);
                // omni thrusters
                velocity.dx += target.x * 0.01 * (dist_to_dest / APPROACH_THRESHOLD_OMNI).min(1.);
                velocity.dy += target.y * 0.01 * (dist_to_dest / APPROACH_THRESHOLD_OMNI).min(1.);

            }

            if dist_to_dest < THRESH_ARRIVAL {
                // TODO interface
                path.path.pop_front();
            }

        }
    }

}

// Passes targets from the parents of turrets to the turrets
fn turret_target_dispatcher(
    mut commands: Commands,
    q_targeters: Query<(Entity, &Children, &Targets), (With<Targets>, Without<Subunit>)>,
    mut q_targets: Query<&mut Targets, With<Subunit>>
) {
    for (entity, children, parent_targets) in q_targeters.iter() {
        for child in children.iter() {
            if let Ok(mut turret_targets) = q_targets.get_mut(*child) {
                turret_targets.deque.clear();
                for targets in parent_targets.deque.iter() {
                    turret_targets.deque.push_back(*targets);
                }
            }
        }
    }
}

fn turret_track_and_fire_system(
    mut commands: Commands,
    mut query: Query<( &Targets, &Body, &mut Velocity), With<Turret>>,
) {
    for (targets, body, mut velocity) in query.iter_mut() {
        velocity.dw += 0.001;
        // println!("Turret at {}, {} has velocity {}", velocity.dw, body.position.x, body.position.y);
        for target in targets.deque.iter() {
            println!("   And has target {}", target.id());
        }
    }
}

fn unit_movement_system(
    mut query: Query<(&mut Transform, &mut Body, &mut Velocity), (With<Velocity>, With<Body>)>,
    // time: Res<Time>
) {
    for (mut transform, mut body, mut velocity) in query.iter_mut() {

        // Update
        body.position.x += velocity.dx;
        body.position.y += velocity.dy;
        body.position.z += velocity.dw;

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
        transform.rotation = Quat::from_rotation_z(body.position.z);

        // Apply drag
        velocity.dx *= DRAG_LATERAL;
        velocity.dy *= DRAG_LATERAL;
        velocity.dw *= DRAG_RADIAL;
        
    }
}

fn unit_hover_system(
    mut query: Query<(&Unit, &mut Transform, &Body, &Velocity), (With<Velocity>, Without<Subunit>)>,
    time: Res<Time>
) {
    for (unit, mut transform, body, vel) in query.iter_mut() {
        // Randomly oscillate units around by some percentage of their size
        let t = time.seconds_since_startup() as f32;
        let second_order: f32 = f32::sin(t * 0.1 + 8. * unit.id as f32);
        transform.translation.x += SPRITE_SCALE * 0.0001 * body.size.y * f32::sin(t + 10. * unit.id as f32) * second_order;
        transform.translation.y += SPRITE_SCALE * 0.0001 * body.size.x * f32::sin(t + 5. * unit.id as f32) * second_order;
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
