use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use std::{sync::atomic::{AtomicU8, Ordering}, fmt::{self}};

pub mod components;
pub use components::*;

// Package level variables
static NUMBER_OF_OWNERS: AtomicU8 = AtomicU8::new(0);
static NUMBER_OF_UNITS: AtomicU8 = AtomicU8::new(0);

const UNIT_ZORDER: f32 = 10.;
const SCALE: f32 = 0.5;

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
            .add_system(kill_system)
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
        Owner { id: NUMBER_OF_OWNERS.fetch_add(1, Ordering::Relaxed) }
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
        match &ev.unit_type {
        UnitType::DefaultUnit => {
            ec.insert(Hp { max: 100, current: 100 } );
            ec.insert_bundle(
                GeometryBuilder::build_as(
                    &shapes::RegularPolygon {
                        sides: 4,
                        feature: shapes::RegularPolygonFeature::Radius(10.0),
                        ..shapes::RegularPolygon::default()
                    },
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::CYAN),
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

fn kill_system(
    query: Query<(Entity, &Unit, &Hp, With<Hp>)>
) {
    for (_entity, unit, hp, _) in query.iter() {
        eprintln!("Entity {}{} is owned by {} and has {} HP.", unit.name, unit.id, unit.owner.id, hp.current);
    }
}
