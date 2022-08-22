#![allow(unused_variables)]
#![allow(unused_labels)]
#![allow(unused_imports)]
#![allow(dead_code)]

use bevy::ecs::entity;
use bevy::render::render_resource::Texture;
use bevy::{prelude::*};
use bevy::time::FixedTimestep;
use bevy_prototype_lyon::prelude::*;
use rand::Rng;

use std::ops::Div;
use std::time::Duration;
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
use inputs::{InputPlugin, MouseOverEvent};

pub mod camera;
pub use camera::*;

pub mod loader;
pub use loader::*;

pub mod ui;
pub use ui::*;

pub mod environment;
pub use environment::*;

pub mod animation;
pub use animation::*;

// Package level variables
static NUMBER_OF_OWNERS: AtomicU8 = AtomicU8::new(0);

const DEBUG_GRAPHICS: bool = true;

// TODO parameterize and IO
const UI_ABOVE_ZORDER: f32 = 500.;
const PROJECTILE_ZORDER: f32 = 150.;
const UNIT_ZORDER: f32 = 100.;
const THRUSTER_PARTICLE_ZORDER: f32 = 25.;
const PLANET_ZORDER: f32 = 50.;
const UI_BENEATH_ZORDER: f32 = 40.;
const WORLD_ZORDER: f32 = 20.;
const BACKGROUND_ZORDER: f32 = 0.;

const MAP_W: i32 = 2 * 16384;
const MAP_H: i32 = 2 * 16384;

const SPRITE_SCALE: f32 = 0.01;

const USER_ID: u8 = 0;

pub struct UnitPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum Stage {
    /// everything that handles input
    Kinematics,
}

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Msaa { samples: 1 })
            .add_plugin(ShapePlugin)
            .add_plugin(InputPlugin)
            .add_plugin(AssetLoaderPlugin)
            .add_event::<SpawnUnitEvent>()
            .add_event::<MouseOverEvent>()
            .add_startup_system(startup_system)
            .add_startup_system_to_stage(StartupStage::Startup, environment_startup_system)
            .add_startup_system_to_stage(StartupStage::Startup, camera_startup_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, background_startup_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, environment_appearance_startup_system)
            .add_system_set(SystemSet::new()  // Unit updates
                .with_run_criteria(FixedTimestep::step(1. / 60.))
                .with_system(turret_track_and_fire_system).label(Stage::Kinematics)
                .with_system(capital_movement_system).label(Stage::Kinematics)
                .with_system(primary_satellite_orbit_system)
                .with_system(secondary_satellite_orbit_system)
                // .with_system(tertiary_satellite_orbit_system).after(secondary_satellite_orbit_system)
                .with_system(projectile_movement_system)
                .with_system(capital_ship_repulsion_system)
                .with_system(capital_ship_destruction_system)
                .with_system(projectile_collision_system)
                .with_system(capital_pathing_system)
                // Graphics
                .with_system(ui_highlight_selected_system)
                .with_system(ui_show_path_system)
                .with_system(ui_show_hp_system)
                .with_system(ui_planet_system)
            )
            .add_system_set(SystemSet::new() // Input 
                .with_run_criteria(FixedTimestep::step(1. / 30.))
                .with_system(inputs::input_mouse_system)
                .with_system(camera_move_system)
            )
            .add_system(thruster_particle_emitter_system)
            .add_system(thruster_particle_system)
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
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary().unwrap();
}

// TODO add these as parameters for various units
const HEADING_THRESH_BURN: f32 = 0.5;  // Radians
const DRAG_LATERAL: f32 = 0.95;
const DRAG_RADIAL: f32 = 0.96;
const APPROACH_THRESHOLD_REAR: f32 = 100.;
const APPROACH_THRESHOLD_OMNI: f32 = 5.;
const THRESH_ARRIVAL: f32 = 50.;
const APPROACH_THRESH: f32 = 3000.;

const PRIMARY_ACCELERATION: f32 = 0.01;
const RADIAL_ACCELERATION: f32 = 0.008;

fn capital_movement_system(
    mut query: Query<(&mut Transform, &mut Body, &Velocity), Or<(With<Unit>, With<Subunit>)>>,
    // time: Res<Time>
) {
    for (mut transform, mut body, velocity) in query.iter_mut() {

        // Update
        body.position.x += velocity.dx;
        body.position.y += velocity.dy;
        body.position.z = (body.position.z + velocity.dw).rem_euclid(2. * PI);

        transform.translation.x = body.position.x;
        transform.translation.y = body.position.y;
        transform.rotation = Quat::from_rotation_z(body.position.z);

    }
}

fn capital_pathing_system(
    mut query: Query<(&mut UnitPath, &Body, &mut Velocity), With<UnitPath>>,
) {
    for (mut path, body, mut velocity ) in query.iter_mut() {
        if !path.path.is_empty() {  // For units with a destination
            let dist_to_dest = (path.path[0] - body.position.truncate()).length();
            let target = (path.path[0] - body.position.truncate()).normalize();
            let heading = Vec2::new(velocity.dx, velocity.dy).normalize();  // Unit vector of the direction of ship's travel
            let pointing = Vec2::new(f32::cos(body.position.z), f32::sin(body.position.z));  // Unit vector of ship's direction
            let cross = target.x * pointing.y - target.y * pointing.x;
            let mut heading_err = (1. - heading.dot(target)) / 2.;  // The angle between the ship's heading and the target -> [0, 1]
            if heading_err.is_infinite() {
                heading_err = 0.0;
            }
            let pointing_err = ((1. - pointing.dot(target)) / 2.).min(0.);  // The angle between the ship's nose and the target -> [0, 1]
            velocity.dw += 
            if cross > 0.0 {
                -RADIAL_ACCELERATION * pointing_err.max(0.001).powf(1. / 3.)
            } else if cross < 0.0 {
                RADIAL_ACCELERATION * pointing_err.max(0.001).powf(1. / 3.)
            } else {
                0.
            };
            // omni thrusters
            // velocity.dx += target.x * 0.0003;
            // velocity.dy += target.y * 0.0003;
            if cross.abs() < HEADING_THRESH_BURN {  // If we are close enough to the right heading to use rear thrusters
                // TODO get values from thrusters
                // Rear thrusters
                velocity.dx += (pointing.x * PRIMARY_ACCELERATION);
                velocity.dy += (pointing.y * PRIMARY_ACCELERATION);
                // velocity.dy += (heading.y * 0.0001) * (dist_to_dest / APPROACH_THRESHOLD_REAR).max(1.);
            }
            if dist_to_dest < body.collision_radius {
                // TODO interface
                path.path.pop_front();
            }
            // Apply drag
            // println!("Velocity is {}, {}", velocity.dx, velocity.dy);
            // println!("Heading err is {}", heading_err);
            // // println!("Pointing err is {}", pointing_err);
            // println!("distance is {}", dist_to_dest);
            // println!("Drag is {}, {}",
            //     dist_to_dest.div(APPROACH_THRESH).powf(1. / 256.).min(1.) - heading_err.div(64.),
            //     dist_to_dest.div(APPROACH_THRESH).powf(1. / 256.).min(1.) - heading_err.div(64.)
            // );
            // velocity.dx *= (dist_to_dest * (1.001 - heading_err)).div(APPROACH_THRESH).powf(1. / 256.).min(1.).max(0.95);
            // velocity.dy *= (dist_to_dest * (1.001 - heading_err)).div(APPROACH_THRESH).powf(1. / 256.).min(1.).max(0.95);
            velocity.dx *= (dist_to_dest.div(APPROACH_THRESH).powf(1. / 256.).min(1.) - heading_err.max(0.0001).div(64.));
            velocity.dy *= (dist_to_dest.div(APPROACH_THRESH).powf(1. / 256.).min(1.) - heading_err.max(0.0001).div(64.));
            velocity.dw *= DRAG_RADIAL;
        }
        else {
            // If there is no path, slow unit down
            velocity.dx *= DRAG_LATERAL;
            velocity.dy *= DRAG_LATERAL;
            velocity.dw *= DRAG_RADIAL;
        }
    }

}


fn thruster_particle_system(
    mut commands: Commands,
    mut q_particles: Query<(Entity, &mut Particle), With<Particle>>,
    time: Res<Time>
) {
    for (e_particle, mut particle) in q_particles.iter_mut() {
        particle.timer.tick(time.delta());
        if particle.timer.just_finished() {
            commands.entity(e_particle).despawn();
        }
    }
}

fn thruster_particle_emitter_system(
    mut commands: Commands,
    q_units: Query<(&Children, &Body, &Transform, &Velocity), (With<Unit>, Without<Thruster>)>,  // TODO display based on THRUST, not velocity
    mut q_thrusters: Query<(&Thruster, &mut ParticleEmitter, &Body, &Transform), With<Thruster>>,
    time: Res<Time>,
    texture_server: Res<TextureServer>
) {
    for (children, unit_body, unit_transform, unit_velocity) in q_units.iter() {
        for child in children {
            if let Ok((thruster, mut emitter, thruster_body, thruster_transform)) = q_thrusters.get_mut(*child) {
                emitter.tick(time.delta());
                // TODO emitter.batch_size
                if emitter.ready() {
                    let n: usize = 10;
                    let emitter_pos = get_absolute_position(
                        thruster_body.position,
                        unit_body.position
                    );
                    let unit_velocity_mag = Vec2::new(unit_velocity.dx, unit_velocity.dy).length();
                    for _ in 0..n {
                        let e_particle = commands.spawn()
                            .insert(Particle::new(
                                Duration::from_millis(emitter.lifetime)
                            )).insert_bundle(SpriteBundle {
                                texture: texture_server.get(&emitter.sprite).typed::<Image>(),
                                sprite: Sprite { 
                                    // color: Color::rgb(emitter.color[0], emitter.color[1], emitter.color[2], emitter.color[3]),
                                    custom_size: Some(Vec2::new(64. * (unit_velocity_mag * 10.).min(1.), 64.) * SPRITE_SCALE),
                                    ..Default::default()
                                },
                                transform: Transform {
                                    translation: emitter_pos.truncate().extend(0.)
                                     + Vec3::new(
                                        emitter.position_variance * 2. * (rand::random::<f32>() - 0.5),
                                        emitter.position_variance * 2. * (rand::random::<f32>() - 0.5),
                                        unit_transform.translation.z + thruster_transform.translation.z + 1.,  // Particles are always emitted from a thruster's first layer
                                    ),
                                    // rotation: Quat::from_rotation_z(emitter_pos.z),
                                    // rotation: Quat::from_rotation_z(emitter_pos.z + emitter.angle_variance * 2. * rand::random::<f32>() - 1.0),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }).id();
                    }
                    emitter.set(1);
                }
            }
        }
    }
}

const TURRET_ON_TARGET_THRESH: f32 = 0.001;  // radians
const TURRET_FIRE_THRESH: f32 = 10. * PI / 180.;  // radians

fn turret_track_and_fire_system(
    mut commands: Commands,
    mut q_turret: Query<(&mut Turret, &Parent, &Subunit, &mut Velocity, &Body), With<Turret>>,
    mut q_targets: Query<&mut Targets>,
    projectiles: Res<ProjectileRegistry>,
    texture_server: Res<TextureServer>,
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
    for (mut turret, turret_parent, subunit, mut turret_velocity, turret_body) in q_turret.iter_mut() {
        let parent_unit: &Unit = q_unit.get(turret_parent.get()).unwrap();
        let mut targets = q_targets.get_mut(turret_parent.get()).unwrap();
        let parent_velocity: &Velocity = q_velocity.get(turret_parent.get()).unwrap();
        let turret_parent_body = q_body.get(turret_parent.get()).unwrap();
        if let Some(target_entity) = targets.get_target() {
            if let Ok(target_body) = q_body.get(target_entity) {
                let heading = Vec2::new(f32::cos(turret_body.position.z + turret_parent_body.position.z), f32::sin(turret_body.position.z + turret_parent_body.position.z));
                let abs_turret_pos = get_absolute_position(turret_body.position, turret_parent_body.position);
                let distance_to_target = (target_body.position.truncate() - abs_turret_pos.truncate()).length();
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
                if cross.abs() < TURRET_FIRE_THRESH && turret.ready() && distance_to_target < turret.range {
                    // Fire!
                    if let Some(projectile_data) = projectiles.get(&turret.projectile) {
                        let mut fire_projectile = |fire_from: Vec3| {
                            if DEBUG_GRAPHICS {
                                let mut path_builder = PathBuilder::new();
                                path_builder.move_to(fire_from.truncate());
                                path_builder.line_to(fire_from.truncate() + heading * 20.);
                                let line = path_builder.build();
                                commands.spawn_bundle(GeometryBuilder::build_as(
                                    &line,
                                    DrawMode::Stroke(StrokeMode::new(
                                        Color::rgba(1., 1., 1., 1.),
                                        1.  // Always draw the same thickness of UI elements regardless of zoom
                                    )),
                                    Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER), ..Default::default() },
                                )).insert( DebugTurretTargetLine );
                            }          
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
                            .insert_bundle( 
                                SpatialBundle {
                                    transform: Transform {
                                            translation: Vec3::new( fire_from.x, fire_from.y, PROJECTILE_ZORDER ),
                                            rotation: Quat::from_rotation_z( fire_from.z ),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                            )
                            .with_children(|parent| {
                                for sprite_data in projectile_data.sprites.iter() {
                                    parent.spawn_bundle(sprite_bundle_from_data(sprite_data, &texture_server, PROJECTILE_ZORDER)).insert(ProjectileSprite);
                                }
                            });
                        };
                        for source in turret.get_sources().iter() {
                            let source_pos =
                                get_absolute_position(source.extend(0.) * SPRITE_SCALE, abs_turret_pos);
                            fire_projectile(source_pos);
                        }
                    }
                    turret.reload();
                }

                // DEBUG GRAPHICS
                if DEBUG_GRAPHICS {
                    for source in turret.get_sources().iter() {
                        let source_pos =
                            get_absolute_position(source.extend(0.) * SPRITE_SCALE, abs_turret_pos);
                        let mut path_builder = PathBuilder::new();
                        path_builder.move_to(source_pos.truncate());
                        path_builder.line_to(source_pos.truncate() + heading * 50.);
                        let line = path_builder.build();
                        commands.spawn_bundle(GeometryBuilder::build_as(
                            &line,
                            DrawMode::Stroke(StrokeMode::new(
                                Color::rgba(1., 1., 1., 0.8),
                                0.5  // Always draw the same thickness of UI elements regardless of zoom
                            )),
                            Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER + 1.), ..Default::default() },
                        )).insert( DebugTurretTargetLine );
                    }
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
                        Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER), ..Default::default() },
                    )).insert( DebugTurretTargetLine );                    
                }
            }
            else {
                targets.move_to_next();
                println!("Could not get target body. The target has likely been despawned. There are {} targets", targets.len());
            }
        }
        else {
            // println!("There were no targets.");
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
                            Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER + 2.), ..Default::default() },
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
                        v1.dx -= 0.1 * heading.x * 1. / (distance * 4.);
                        v1.dy -= 0.1 * heading.y * 1. / (distance * 4.);
                    }
                    if let Ok(mut v2) = q_velocity.get_mut(entity) {
                        v2.dx += 0.1 * heading.x * 1. / (distance * 4.);
                        v2.dy += 0.1 * heading.y * 1. / (distance * 4.);      
                        v2.dy += 0.1 * heading.y * 1. / (distance * 4.);      
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
                        Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER + 3.), ..Default::default() },
                    )).insert( DebugCollisionCheckLine );  
                }
            }
        }
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

pub fn rotate_vector(p: Vec2, w: f32) -> Vec2 {
    Vec2::new(
        p.x * w.cos() - p.y * w.sin(),
        p.x * w.sin() + p.y * w.cos()
    )
}

pub fn get_absolute_position(subunit_position: Vec3, parent_position: Vec3) -> Vec3 {
    let mut abs_pos: Vec3 = Vec3::from(parent_position);
    abs_pos.x += subunit_position.x * f32::cos(parent_position.z) - subunit_position.y * f32::sin(parent_position.z);
    abs_pos.y += subunit_position.x * f32::sin(parent_position.z) + subunit_position.y * f32::cos(parent_position.z);
    abs_pos.z += subunit_position.z;
    return abs_pos
}
