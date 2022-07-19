use std::path::Component;

use bevy::prelude::*;

pub mod components;
pub use components::*;

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
            .add_event::<SpawnUnitEvent>()
            .add_system(kill_system)
            .add_system(spawn_units_system);
    }
}

pub enum UnitType {
    DefaultUnit,
    Tank,
    Plane,
    Building,
}

pub struct SpawnUnitEvent {
    unit_type: UnitType,
}

impl SpawnUnitEvent {
    pub fn new(unit_type: UnitType) -> SpawnUnitEvent {
        SpawnUnitEvent { unit_type: unit_type }
    }
}

// Master decoder of units and their properties. TODO convert to table
fn spawn_units_system(
    mut ev_spawn: EventReader<SpawnUnitEvent>,
    mut commands: Commands,
) {
    for ev in ev_spawn.iter() {
        let commands = commands.spawn()
            .insert(Unit {} )
            .insert(Id { name: String::from("Unit1") });
        match &ev.unit_type {
            println("Spawning DefaultUnit");
            UnitType::DefaultUnit => {
                
                commands.insert(Hp { max: 100, current: 100 } );
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
    query: Query<(Entity, &Hp, &Id, With<Hp>)>
) {
    for (_entity, hp, id, _) in query.iter() {
        eprintln!("Entity {} has {} HP.", id.name, hp.current);
    }
}
    
