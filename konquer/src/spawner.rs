use std::iter::zip;

use bevy::{prelude::*};
use bevy_prototype_lyon::prelude::*;

use crate::*;

pub struct SpawnUnitEvent {
    unit_type: String,
    player: Player,
    position: Vec3,
}

impl SpawnUnitEvent {
    pub fn new(unit_type: String, player: Player, position: Vec3) -> SpawnUnitEvent {
        SpawnUnitEvent { unit_type: unit_type, player: player, position: position}
    }
}

pub fn spawn_units_system(
    mut ev_spawn: EventReader<SpawnUnitEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    unit_data_server: Res<UnitDataCollection>
) {
    'events: for ev in ev_spawn.iter() {
        if let Some(unit_data) = unit_data_server.get(&ev.unit_type) {
            println!("Spawning {} owned by Player {} at {}, {}", ev.unit_type, ev.player.id, ev.position.x, ev.position.y);

            let mut ec = commands.spawn();
            ec.insert(Unit::new(ev.unit_type.clone(), ev.player.clone()));
            println!("Unit based on platform {}", unit_data.platform["type"]);
            if let Some(unit_type) = unit_data.platform["type"].as_str()
            {
                match unit_type {
                    "capital" => {
                        // TODO error checking
                        let unit_hitpoints = unit_data.platform["hp"].as_u64().unwrap();
                        let unit_size = serde_to_bevy_vec2(unit_data.platform["size"].as_array().unwrap());

                        ec.insert( Velocity { ..Default::default() } );
                        ec.insert( Hp { max: unit_hitpoints, current: unit_hitpoints } );
                        ec.insert( Body::new(ev.position, unit_size) );
                        ec.insert( Targets::new() );
                        if ev.player.id == USER_ID {
                            ec.insert( Targeterable );
                            ec.insert( Movable );
                        }
                        else {
                            ec.insert( Targeteeable );
                        }
                        ec.insert( Selectable );
                        ec.insert( UnitPath::new() );
                        ec.insert_bundle( TransformBundle {
                            local: Transform {
                                translation: Vec3::new( ev.position.x, ev.position.y, UNIT_ZORDER ),
                                rotation: Quat::from_rotation_z( ev.position.z ),
                                ..Default::default()
                            },
                            ..Default::default()
                        });

                        ec.with_children(|parent| {

                            // Add sprites

                            // TODO function
                            for sprite_data in unit_data.platform["sprites"].as_array().unwrap().iter() {
                                let sprite_z = sprite_data["z-order"].as_f64().unwrap() as f32;
                                let sprite_size = serde_to_bevy_vec2(sprite_data["size"].as_array().unwrap());
                                println!("Adding sprite {}", sprite_data["texture"].as_str().unwrap());
                                parent.spawn_bundle(SpriteBundle {
                                    texture: asset_server.load(sprite_data["texture"].as_str().unwrap()),
                                    sprite: Sprite {
                                        custom_size: Some(sprite_size * SPRITE_SCALE),
                                        ..Default::default()
                                    },
                                    transform: Transform { translation: Vec3::new(0., 0., sprite_z), ..Default::default() },
                                    ..Default::default()
                                });
                            }

                            // Debug sprites
                            parent.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::rgba(1., 0., 0., 0.01),
                                    custom_size: Some(unit_size * SPRITE_SCALE),
                                    ..Default::default()
                                },
                                transform: Transform { translation: Vec3::new(0., 0., -1.), ..Default::default() },
                                ..Default::default()
                            }).insert(DebugRect);

                            parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                                sides: 30,
                                feature: shapes::RegularPolygonFeature::Radius((unit_size[0] + unit_size[1]) * SPRITE_SCALE / 4.),
                                ..shapes::RegularPolygon::default()
                            },
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::rgba(0., 1., 0., 0.1)),
                                outline_mode: StrokeMode::new(Color::rgba(0., 1., 0., 0.), 2.),
                            },
                            Transform { translation: Vec3::new(0., 0., -2.), ..Default::default() },
                            )).insert(DebugSelectionRadius);

                            // Add subunits
                            for (subunit, hardpoint) in zip(unit_data.loadout.iter(), 
                            unit_data.platform["hardpoints"].as_array().unwrap().iter()) {
                                match subunit["type"].as_str().unwrap() {
                                    "turret" => {
                                        println!("Adding turret...");
                                        add_turret(parent, subunit, hardpoint, &asset_server);
                                    },
                                    "thruster" => {
                                        println!("Adding thruster...");
                                    },
                                    _ => ()
                                }
                            }
                        });   
                    },
                    _ => ()
                }
            }

        }
        else {
            println!("Invalid Unit Type: {}.", ev.unit_type)
        }
        
        


        // let mut ec = commands.spawn();
        // ec.insert(Unit::new(ev.unit_type.to_string(), ev.player.clone()));
        // ec.insert( Velocity { ..Default::default() } );
        // match &ev.unit_type {

        // UnitType::Frigate1 => {
        //     // TODO bundle
        //     let unit_size = Vec2::new(1350., 762.);
        //     ec.insert( Hp { max: 100, current: 100 } );
        //     ec.insert( Body::new(ev.position, unit_size) );
        //     ec.insert( Targets::new() );
        //     if ev.player.id == USER_ID {
        //         ec.insert( Targeterable );
        //         ec.insert( Movable );
        //     }
        //     else {
        //         ec.insert( Targeteeable );
        //     }
        //     ec.insert( Selectable );
        //     ec.insert( UnitPath::new() );
        //     ec.insert_bundle( TransformBundle {
        //         local: Transform {
        //             translation: Vec3::new( ev.position.x, ev.position.y, UNIT_ZORDER ),
        //             rotation: Quat::from_rotation_z( ev.position.z ),
        //             ..Default::default()
        //         },
        //         ..Default::default()
        //     });

        //     // Add children
        //     ec.with_children(|parent| {
                
        //         // TODO thrusters

        //         // Hardpoints
        //         add_turret(parent, &asset_server, Vec3::new(460., 0., 0.));
        //         add_turret(parent, &asset_server, Vec3::new(-110., 0., 0.));

        //         // Debug sprites
        //         parent.spawn_bundle(SpriteBundle {
        //             sprite: Sprite {
        //                 color: Color::rgba(1., 0., 0., 0.01),
        //                 custom_size: Some(unit_size * SPRITE_SCALE),
        //                 ..Default::default()
        //             },
        //             transform: Transform { translation: Vec3::new(0., 0., -1.), ..Default::default() },
        //             ..Default::default()
        //         }).insert(DebugRect);
                
        //         parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
        //             sides: 30,
        //             feature: shapes::RegularPolygonFeature::Radius((unit_size[0] + unit_size[1]) * SPRITE_SCALE / 4.),
        //             ..shapes::RegularPolygon::default()
        //         },
        //         DrawMode::Outlined {
        //             fill_mode: FillMode::color(Color::rgba(0., 1., 0., 0.1)),
        //             outline_mode: StrokeMode::new(Color::rgba(0., 1., 0., 0.), 2.),
        //         },
        //         Transform { translation: Vec3::new(0., 0., -2.), ..Default::default() },
        //         )).insert(DebugSelectionRadius);

        //         // Sprites
        //         parent.spawn_bundle(SpriteBundle {
        //             texture: asset_server.load("ship1.png"),
        //             sprite: Sprite {
        //                 custom_size: Some(unit_size * SPRITE_SCALE),
        //                 ..Default::default()
        //             },
        //             transform: Transform { translation: Vec3::new(0., 0., UNIT_ZORDER), ..Default::default() },
        //             ..Default::default()
        //         }).insert(MainSprite);

        //     });
        // }
        // other => {
        //     println!("Unit not spawned.\n");
        // }

        // };
    }
}

// TODO strongly type
type TurretData = serde_json::Value;
type HardpointData = serde_json::Value;
// type SpriteData = serde_json::Value;

fn add_turret(parent: &mut ChildBuilder, subunit_data: &TurretData, hardpoint_data: &HardpointData, asset_server: &Res<AssetServer>) {
    let subunit_size = serde_to_bevy_vec2(subunit_data["size"].as_array().unwrap());
    let subunit_pos = serde_to_bevy_vec3(hardpoint_data["position"].as_array().unwrap());
    let mut ec = parent.spawn();
    for sprite_data in subunit_data["sprites"].as_array().unwrap().iter() {
        let sprite_z = (sprite_data["z-order"].as_f64().unwrap() + hardpoint_data["z-order"].as_f64().unwrap()) as f32;
        let sprite_size = serde_to_bevy_vec2(sprite_data["size"].as_array().unwrap());
        ec.insert_bundle(SpriteBundle {
            texture: asset_server.load(sprite_data["texture"].as_str().unwrap()),
            sprite: Sprite {
                custom_size: Some(sprite_size * SPRITE_SCALE),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(subunit_pos.x * SPRITE_SCALE, subunit_pos.y * SPRITE_SCALE, sprite_z),
                rotation: Quat::from_rotation_z( subunit_pos.z ),
                 ..Default::default()
                },
            ..Default::default()
        });
    }
    ec
    .insert(
        Body::new(
            Vec3::new(subunit_pos.x * SPRITE_SCALE, subunit_pos.y * SPRITE_SCALE, subunit_pos.z),
            subunit_size
        )
    )
    .insert(Unit::new(
        String::from(subunit_data["name"].as_str().unwrap()),
        Player { id: USER_ID, teamcolor: Color::rgb(0., 1., 0.) }  // TODO colors
    ))
    .insert(Subunit { relative_position: Vec3::new(subunit_pos.x, subunit_pos.y, 0.) } )
    .insert(Velocity { ..Default::default() })
    .insert(Range {
        sight: subunit_data["range-sight"].as_f64().unwrap() as f32,
        fire: subunit_data["range-fire"].as_f64().unwrap() as f32
    })
    .insert(Targets::new())
    .insert(Turret {
        reload_time: subunit_data["reload-time"].as_f64().unwrap() as f32
    });

}

pub fn serde_to_bevy_vec2(value_vector: &Vec<serde_json::Value>) -> Vec2 {
    Vec2::new(value_vector[0].as_f64().unwrap() as f32, value_vector[1].as_f64().unwrap() as f32)
}

pub fn serde_to_bevy_vec3(value_vector: &Vec<serde_json::Value>) -> Vec3 {
    Vec3::new(
        value_vector[0].as_f64().unwrap() as f32,
        value_vector[1].as_f64().unwrap() as f32,
        value_vector[2].as_f64().unwrap() as f32
    )
}