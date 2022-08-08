#![allow(unused_variables)]
#![allow(unused_labels)]
#![allow(unused_imports)]
#![allow(dead_code)]

use bevy::ecs::entity;
use bevy::render::render_resource::Texture;
use bevy::{prelude::*};
use bevy::core::FixedTimestep;
use bevy_prototype_lyon::prelude::*;

use std::{sync::atomic::{AtomicU8, Ordering}, fmt::{self}, f32::consts::PI};

pub mod data;
pub use data::*;

pub mod quadtree;
pub use quadtree::*;

pub mod components;
pub use components::*;

pub mod spawner;
pub use spawner::*;

pub mod inputs;
use inputs::InputPlugin;

pub mod camera;
pub use camera::*;

pub mod loader;
pub use loader::*;

pub mod ui;
pub use ui::*;

// Package level variables
static NUMBER_OF_OWNERS: AtomicU8 = AtomicU8::new(0);

const DEBUG_GRAPHICS: bool = false;

// TODO parameterize and IO
const UI_ZORDER: f32 = 20.;
const UNIT_ZORDER: f32 = 100.;
const PROJECTILE_ZORDER: f32 = 200.;
const WORLD_ZORDER: f32 = 0.;

const MAP_W: i32 = 500;
const MAP_H: i32 = 500;

const SPRITE_SCALE: f32 = 0.05;

const USER_ID: u8 = 0;

type WindowSize = Vec2;

pub struct TextureServer {
    collection: std::collections::HashMap<String, Handle<TextureAtlas>>
}

impl TextureServer {
    pub fn new() -> Self {
        Self { collection: std::collections::HashMap::new() }
    }
    pub fn insert(&mut self, name: String, element: Handle<TextureAtlas>) {
        self.collection.insert(name, element);
    }
    pub fn get(&self, key: &String) -> Option<&Handle<TextureAtlas>> {
        self.collection.get(key)
    }
}

pub struct UnitPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum Stage {
    /// everything that handles input
    Kinematics,
}

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
            .insert_resource( TextureServer::new() )
            .add_plugin(ShapePlugin)
            .add_plugin(InputPlugin)
            .add_plugin(AssetLoaderPlugin)
            .add_event::<SpawnUnitEvent>()
            .add_startup_system(startup_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, map_system)
            .add_plugin(KinematicCameraPlugin)
            .add_system_set(SystemSet::new()  // Unit updates
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(turret_track_and_fire_system).label(Stage::Kinematics)
                .with_system(unit_movement_system).label(Stage::Kinematics)
                .with_system(projectile_movement_system)
                .with_system(capital_ship_repulsion_system)
                .with_system(capital_ship_destruction_system)
                .with_system(projectile_collision_system)
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
            .add_system(ui_show_hp_system)
            .add_system(explosion_to_spawn_system)
            .add_system(explosion_animation_system)
            // Mechanics
            .add_system(spawn_units_system)
            // .add_system(teamcolor_system) TODO
            ;
    }
}

// An player of a Unit in Konquer
#[derive(Clone)]
pub struct Player {
    id: u8,
    teamcolor: Color
}

impl Player {
    pub fn new() -> Player {
        // Create a unique Player ID each time new is called
        Player {
            id: NUMBER_OF_OWNERS.fetch_add(1, Ordering::Relaxed),
            teamcolor: Color::rgb(0., 0., 1.),  // TODO from table
        }
    }
}

fn startup_system(
    mut commands: Commands,
    windows: Res<Windows>,
    mut texture_server: ResMut<TextureServer>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary().unwrap();
    let window_size = WindowSize::new(window.width(), window.height());
	commands.insert_resource(window_size);
    texture_server.insert(String::from("data\\fx\\explo_a_sheet.png"), asset_server.load("data\\fx\\explo_a_sheet.png"));
}

// TODO add these as parameters for various units
const HEADING_THRESH_BURN: f32 = 0.8;  // Radians
const DRAG_LATERAL: f32 = 0.975;
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
                -0.001 * err.sqrt().min(1.).max(0.1)
            } else if cross < 0.0 {
                0.001 * err.sqrt().min(1.).max(0.1)
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

// Passes targets to subunits
fn turret_target_dispatcher(
    q_targeters: Query<(Entity, &Children, &Targets), (With<Targets>, Without<Subunit>)>,
    mut q_targets: Query<&mut Targets, With<Subunit>>
) {
    for (parent_entity, children, parent_targets) in q_targeters.iter() {
        for child in children.iter() {
            if let Ok(mut child_targets) = q_targets.get_mut(*child) {
                // child_targets.deque.clear();
                // TODO child target clearing
                // TODO prioritization based on distance
                'targets: for parent_target in parent_targets.deque.iter() {
                    for child_target in child_targets.deque.iter() {
                        if child_target.id() == parent_target.id() {
                            // Don't add a duplicate target
                            continue 'targets;
                        }
                    }
                    child_targets.deque.push_back(*parent_target);
                }
            }
        }
    }
}


const TURRET_ON_TARGET_THRESH: f32 = 0.001;  // radians
const TURRET_FIRE_THRESH: f32 = 10. * PI / 180.;  // radians

fn turret_track_and_fire_system(
    mut commands: Commands,
    mut q_turret: Query<(&mut Turret, &Parent, &Targets, &Subunit, &mut Velocity, &Body), With<Turret>>,
    projectiles: Res<ProjectileRegistry>,
    asset_server: Res<AssetServer>,
    q_body: Query<&Body>,
    q_unit: Query<&Unit>,
    q_velocity: Query<&Velocity, Without<Subunit>>,
    q_debug_graphics: Query<Entity, With<DebugTurretTargetLine>>,
    time: Res<Time>,
) {
    if DEBUG_GRAPHICS {
        for line in q_debug_graphics.iter() {
            commands.entity(line).despawn();
        }
    }
    for (mut turret, turret_parent, targets, subunit, mut turret_velocity, turret_body) in q_turret.iter_mut() {
        let parent_unit: &Unit = q_unit.get(turret_parent.0).unwrap();
        let parent_velocity: &Velocity = q_velocity.get(turret_parent.0).unwrap();
        let turret_parent_body = q_body.get(turret_parent.0).unwrap();
        for target_entity in targets.deque.iter() {
            if let Ok(target_body) = q_body.get(*target_entity) {
                let heading = Vec2::new(f32::cos(turret_body.position.z + turret_parent_body.position.z), f32::sin(turret_body.position.z + turret_parent_body.position.z));
                let abs_turret_pos = get_absolute_position(turret_body.position, turret_parent_body.position);
                let dist_to_dest = (target_body.position.truncate() - abs_turret_pos.truncate()).length();
                let target = (target_body.position.truncate() - abs_turret_pos.truncate()).normalize();
                let cross = target.x * heading.y - target.y * heading.x;
                let err = 1. - heading.dot(target);
                
                if cross.abs() > TURRET_ON_TARGET_THRESH {
                    turret_velocity.dw = 
                    if cross > 0.0 {
                        -0.1 * err.sqrt().min(1.)
                    } else if cross < 0.0 {
                        0.1 * err.sqrt().min(1.)
                    } else {
                        0.
                    };  
                }

                turret.tick(time.delta());
                if cross.abs() < TURRET_FIRE_THRESH && turret.ready() {
                    // Fire!
                    if let Some(projectile_data) = projectiles.get(&turret.projectile) {
                        let mut fire_projectile = |fire_from: Vec3| {
                            let mut ec = commands.spawn();
                            ec.insert(Projectile {
                                fired_from: fire_from.truncate(),
                                range: projectile_data.range,
                                player: parent_unit.player.clone(),
                                damage: projectile_data.damage
                            });
                            ec
                            .insert(Body::new(fire_from, Vec2::new(projectile_data.size[0], projectile_data.size[1])))
                            .insert(Velocity {
                                // dx: heading.x * projectile_data.velocity + parent_velocity.dx,
                                // dy: heading.y * projectile_data.velocity + parent_velocity.dy,
                                dx: heading.x * projectile_data.velocity,
                                dy: heading.y * projectile_data.velocity,
                                dw: 0.0
                            })
                            .insert_bundle( TransformBundle {
                                local: Transform {
                                    translation: Vec3::new( fire_from.x, fire_from.y, PROJECTILE_ZORDER ),
                                    rotation: Quat::from_rotation_z( fire_from.z ),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                for sprite_data in projectile_data.sprites.iter() {
                                    parent.spawn_bundle(sprite_bundle_from_data(sprite_data, &asset_server, PROJECTILE_ZORDER)).insert(ProjectileSprite);
                                }
                            });
                        };
                        for source in turret.get_sources().iter() {
                            let source_pos =
                                get_absolute_position(source.extend(0.), abs_turret_pos);
                            fire_projectile(source_pos);
                        }
                    }
                    turret.reload();
                }

                // DEBUG GRAPHICS
                if DEBUG_GRAPHICS {
                    let mut path_builder = PathBuilder::new();
                    path_builder.move_to(abs_turret_pos.truncate());
                    path_builder.line_to(target_body.position.truncate());
                    let line = path_builder.build();
                    commands.spawn_bundle(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Stroke(StrokeMode::new(
                            Color::rgba(1., 0., 0., 0.7),
                            1.  // Always draw the same thickness of UI elements regardless of zoom
                        )),
                        Transform { translation: Vec3::new(0., 0., 50.), ..Default::default() },
                    )).insert( DebugTurretTargetLine );                    
                    let mut path_builder = PathBuilder::new();
                    path_builder.move_to(abs_turret_pos.truncate());
                    path_builder.line_to(abs_turret_pos.truncate() + heading * 50.);
                    let line = path_builder.build();
                    commands.spawn_bundle(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Stroke(StrokeMode::new(
                            Color::rgba(0., 1., 0., 0.5),
                            3.  // Always draw the same thickness of UI elements regardless of zoom
                        )),
                        Transform { translation: Vec3::new(0., 0., 150.), ..Default::default() },
                    )).insert( DebugTurretTargetLine );
                }
            }
            else {
                println!("Could not get target body")
            }
        }
    }
}

fn explosion_to_spawn_system(
	mut commands: Commands,
	query: Query<(Entity, &ExplosionToSpawn)>,
    texture_server: Res<TextureServer>
) {
	for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
		// spawn the explosion sprite
		if let Some(texture) = texture_server.get(&String::from("data\\fx\\explo_a_sheet.png")) {
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: texture.clone(),
                    transform: Transform {
                        translation: explosion_to_spawn.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Explosion)
                .insert(ExplosionTimer::default());
    
            // despawn the explosionToSpawn
            commands.entity(explosion_spawn_entity).despawn();
        }
	}
}

fn explosion_animation_system(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
	for (entity, mut timer, mut sprite) in query.iter_mut() {
		// println!("Timer tickin");
        timer.0.tick(time.delta());
		if timer.0.finished() {
			sprite.index += 1; // move to next sprite cell
			if sprite.index >= 16 {  // TODO manage fx sheets
				commands.entity(entity).despawn()
			}
		}
	}
}

fn capital_ship_destruction_system(
    mut commands: Commands,
    q_capitals: Query<(Entity, &Body, &Hp), With<CapitalShip>>
) {
    for (entity, body, hp) in q_capitals.iter() {
        if hp.current == 0 {
            commands.spawn().insert(ExplosionToSpawn(body.position));
            commands.entity(entity).despawn_recursive();
        }
    }
}


fn projectile_collision_system(
    mut commands: Commands,
    q_debug: Query<Entity, With<DebugProjectileCollisionCheckLine>>,
    mut q_units: Query<(Entity, &Unit, &mut Hp, &Body), With<Unit>>,
    q_projectiles: Query<(Entity, &Projectile, &Body), With<Projectile>>,
) { 
    if DEBUG_GRAPHICS {
        for line in q_debug.iter() {
            commands.entity(line).despawn();
        }
    }
    let mut qtree = CollisionQuadtree::new(0, Rectangle2D { x: 0., y: 0., width: MAP_W as f32, height: MAP_H as f32 });
    for (entity, _, body) in q_projectiles.iter() {
        qtree.insert(EntityBody { entity: entity, position: body.position.truncate(), radius: body.collision_radius })
    }
    let mut colliders: Vec<EntityBody> = Vec::new();
    for (unit_e, unit, mut unit_hp, unit_body) in q_units.iter_mut() {
        colliders.clear();
        qtree.retrieve(unit_body.position.truncate(), unit_body.collision_radius, &mut colliders);
        for projectile_eb in colliders.iter() {
            if let Ok((projectile_e, projectile, projectile_body)) = q_projectiles.get(projectile_eb.entity) {
                if unit.player.id != projectile.player.id {  // Friendly fire off
                    
                    let distance = unit_body.position.truncate().distance(projectile_body.position.truncate());
                    if distance < (projectile_body.collision_radius + unit_body.collision_radius) {
                        unit_hp.current = (unit_hp.current as f32 - projectile.damage) as u64;
                        println!("Unit now has {} hp", unit_hp.current);
                        commands.entity(projectile_e).despawn_recursive();
                    }
                    if DEBUG_GRAPHICS {
                        let mut path_builder = PathBuilder::new();
                        path_builder.move_to(projectile_eb.position);
                        path_builder.line_to(unit_body.position.truncate());
                        let line = path_builder.build();
                        commands.spawn_bundle(GeometryBuilder::build_as(
                            &line,
                            DrawMode::Stroke(StrokeMode::new(
                                Color::rgba(1., 0., 1., 0.5),
                                1.
                            )),
                            Transform { translation: Vec3::new(0., 0., 50.), ..Default::default() },
                        )).insert( DebugProjectileCollisionCheckLine );  
                    }    
                }
            }
        }
    }
}

// Capital ships do not collide, but instead repel one another
fn capital_ship_repulsion_system(
    mut commands: Commands,
    q_debug: Query<Entity, With<DebugCollisionCheckLine>>,
    q_capitals: Query<(Entity, &Body), With<CapitalShip>>,
    mut q_velocity: Query<&mut Velocity, With<CapitalShip>>,
) { 
    if DEBUG_GRAPHICS {
        for line in q_debug.iter() {
            commands.entity(line).despawn();
        }
    }
    let mut qtree = CollisionQuadtree::new(0, Rectangle2D { x: 0., y: 0., width: MAP_W as f32, height: MAP_H as f32 });
    for (entity, body) in q_capitals.iter() {
        qtree.insert(EntityBody { entity: entity, position: body.position.truncate(), radius: body.repulsion_radius })
    }
    let mut colliders: Vec<EntityBody> = Vec::new();
    for (entity, body) in q_capitals.iter() {
        colliders.clear();
        qtree.retrieve(body.position.truncate(), body.repulsion_radius, &mut colliders);
        for e in colliders.iter() {
            if e.entity.id() != entity.id() {
                let distance = body.position.truncate().distance(e.position);
                if distance < (e.radius + body.repulsion_radius) {
                    let heading = (body.position.truncate() - e.position).normalize();
                    if let Ok(mut v1) = q_velocity.get_mut(e.entity) {
                        v1.dx -= heading.x * 1. / (distance * 4.);
                        v1.dy -= heading.y * 1. / (distance * 4.);
                    }
                    if let Ok(mut v2) = q_velocity.get_mut(entity) {
                        v2.dx += heading.x * 1. / (distance * 4.);
                        v2.dy += heading.y * 1. / (distance * 4.);      
                    }
                } 
                if DEBUG_GRAPHICS {
                    let mut path_builder = PathBuilder::new();
                    path_builder.move_to(e.position);
                    path_builder.line_to(body.position.truncate());
                    let line = path_builder.build();
                    commands.spawn_bundle(GeometryBuilder::build_as(
                        &line,
                        DrawMode::Stroke(StrokeMode::new(
                            Color::rgba(1., 1., 1., 0.5),
                            1.
                        )),
                        Transform { translation: Vec3::new(0., 0., 50.), ..Default::default() },
                    )).insert( DebugCollisionCheckLine );  
                }
            }
        }
    }
}

fn unit_movement_system(
    mut query: Query<(&mut Transform, &mut Body, &mut Velocity), Or<(With<Unit>, With<Subunit>)>>,
    // time: Res<Time>
) {
    for (mut transform, mut body, mut velocity) in query.iter_mut() {

        // Update
        body.position.x += velocity.dx;
        body.position.y += velocity.dy;
        body.position.z = (body.position.z + velocity.dw).rem_euclid(2. * PI);

        // println!("Entity at {}, {}, {}", body.position.x, body.position.y, body.position.z);

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
        transform.rotation = Quat::from_rotation_z(body.position.z);

        // Apply drag
        velocity.dx *= DRAG_LATERAL;
        velocity.dy *= DRAG_LATERAL;
        velocity.dw *= DRAG_RADIAL;
        
    }
}

fn projectile_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Projectile, &mut Transform, &mut Body, &Velocity), With<Projectile>>,
    // time: Res<Time>
) {
    for (entity, projectile, mut transform, mut body, velocity) in query.iter_mut() {

        if projectile.fired_from.distance(body.position.truncate()) > projectile.range {
            commands.entity(entity).despawn_recursive()
        }
        else
        {
            // Update
            body.position.x += velocity.dx;
            body.position.y += velocity.dy;
            body.position.z = (body.position.z + velocity.dw).rem_euclid(2. * PI);

            // println!("Projectile at {}, {}, {}", body.position.x, body.position.y, body.position.z);

            transform.translation.x = body.position.x;
            transform.translation.y = body.position.y;
            transform.rotation = Quat::from_rotation_z(body.position.z);
        }
    }
}

fn map_system(
    mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut q_camera: Query<(&OrthographicProjection, &mut Transform), With<Camera>>,
) {
    
    commands.insert_resource( CollisionQuadtree::new(0, Rectangle2D { x: 0., y: 0., width: MAP_W as f32, height: MAP_H as f32 }) );
    
    let map: Map = Map { w: MAP_W, h: MAP_H };
	let mut ec = commands.spawn();
    ec.insert(map);

    // Draw map grid
	let (projection, mut transform) = q_camera.single_mut();
    
    transform.translation.x = MAP_W as f32 / 2.;
    transform.translation.y = MAP_H as f32 / 2.;

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
        for y in (0..map.h).step_by(100) {
            draw_gridline(Vec2::new(0., y as f32), Vec2::new(MAP_W as f32, y as f32));
        }
        for x in (0..map.w).step_by(100) {
            draw_gridline(Vec2::new(x as f32, 0.), Vec2::new(x as f32, MAP_H as f32));
        }
    });
}

pub fn get_absolute_position(subunit_position: Vec3, parent_position: Vec3) -> Vec3 {
    let mut abs_pos: Vec3 = Vec3::from(parent_position);
    abs_pos.x += subunit_position.x * f32::cos(parent_position.z) - subunit_position.y * f32::sin(parent_position.z);
    abs_pos.y += subunit_position.x * f32::sin(parent_position.z) + subunit_position.y * f32::cos(parent_position.z);
    abs_pos.z += subunit_position.z;
    return abs_pos
}
