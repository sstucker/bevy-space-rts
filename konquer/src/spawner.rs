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

// TODO
// pub fn teamcolor_system(
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     query: Query<(&mut Handle<ColorMaterial>, &TeamSprite)>
// ) {
//     for (handle, teamsprite) in query.iter() {
//         materials.get_mut(handle).unwrap().color = teamsprite.color;
//         println!("Set color to {:?}", teamsprite.color);
//     }
// }

pub fn spawn_units_system(
    mut ev_spawn: EventReader<SpawnUnitEvent>,
    mut commands: Commands,
    texture_server: Res<TextureServer>,
    unit_data_server: Res<UnitDataCollection>
) {
    'events: for ev in ev_spawn.iter() {
        if let Some(unit_data) = unit_data_server.get(&ev.unit_type) {
            println!("Spawning {} owned by Player {} at {}, {}", ev.unit_type, ev.player.id, ev.position.x, ev.position.y);

            let mut ec = commands.spawn();
            ec.insert(Unit::new(ev.unit_type.clone(), ev.player.clone()));
            let unit_hitpoints = unit_data.platform.hp;  // TODO compute from subunit buffs
            let unit_size = Vec2::new(unit_data.platform.size[0], unit_data.platform.size[1]);
            let body = Body::new(ev.position, unit_size);
            ec.insert( Hp { max: unit_hitpoints, current: unit_hitpoints } );
            ec.insert( body );
            match unit_data.platform.class.clone() {
                PlatformClassData::Capital { range_radius, forward_burn_threshold, lateral_drag, radial_drag } => {
                    ec.insert( Velocity { ..Default::default() } );
                    // TODO error checking
                    ec.insert( Targets::new() );
                    if ev.player.id == USER_ID {
                        ec.insert( Targeterable );
                        ec.insert( Movable );
                    }
                    else {
                        ec.insert( Targeteeable );
                    }
                    ec.insert( CapitalShip );
                    ec.insert( Selectable );
                    ec.insert( UnitPath::new() );
                    // Unit master transform
                    ec.insert_bundle( SpatialBundle {
                        transform: Transform {
                            translation: Vec3::new( ev.position.x, ev.position.y, UNIT_ZORDER ),
                            rotation: Quat::from_rotation_z( ev.position.z ),
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                    ec.with_children(|parent| {

                        // Add sprites
                        for sprite_data in unit_data.platform.sprites.iter() {
                            parent.spawn_bundle(sprite_bundle_from_data(sprite_data, &texture_server, 0.))
                                .insert(MainSprite);
                        }
                        // Teamcolor sprite
                        parent.spawn_bundle(
                            sprite_bundle_from_data(&unit_data.platform.teamcolor_sprite, &texture_server, 0.)
                        )
                        .insert(TeamSprite { color: ev.player.teamcolor } );

                        if DEBUG_GRAPHICS {
                            // Debug sprites
                            parent.spawn_bundle(SpriteBundle {
                                sprite: Sprite {
                                    color: Color::rgba(1., 0., 0., 0.01),
                                    custom_size: Some(unit_size * SPRITE_SCALE),
                                    ..Default::default()
                                },
                                transform: Transform { translation: Vec3::new(0., 0., UNIT_ZORDER - 1.), ..Default::default() },
                                ..Default::default()
                            }).insert(DebugRect);

                            parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                                sides: 30,
                                feature: shapes::RegularPolygonFeature::Radius(body.selection_radius),
                                ..shapes::RegularPolygon::default()
                            },
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::rgba(0., 0., 1., 0.1)),
                                outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 2.),
                            },
                            Transform { translation: Vec3::new(0., 0., UNIT_ZORDER - 2.), ..Default::default() },
                            )).insert(DebugSelectionRadius);

                            parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                                sides: 30,
                                feature: shapes::RegularPolygonFeature::Radius(body.collision_radius),
                                ..shapes::RegularPolygon::default()
                            },
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::rgba(1., 0., 0.5, 0.1)),
                                outline_mode: StrokeMode::new(Color::rgba(0., 1., 0., 0.), 2.),
                            },
                            Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER -3.), ..Default::default() },
                            )).insert(DebugCollisionRadius);

                            parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                                sides: 30,
                                feature: shapes::RegularPolygonFeature::Radius(body.repulsion_radius),
                                ..shapes::RegularPolygon::default()
                            },
                            DrawMode::Outlined {
                                fill_mode: FillMode::color(Color::rgba(0., 1., 0., 0.1)),
                                outline_mode: StrokeMode::new(Color::rgba(0., 1., 0., 0.), 2.),
                            },
                            Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER - 4.), ..Default::default() },
                            )).insert(DebugRepulsionRadius);
                        }

                        // Add subunits
                        for (subunit, hardpoint) in zip(
                            unit_data.loadout.iter(), 
                            unit_data.platform.hardpoints.iter()
                        ) {
                            add_subunit(parent, subunit, hardpoint, &texture_server);
                        }
                    });   
                },
                PlatformClassData::Depot { forward_thrust } => {

                }
            }
        }
        else {
            eprintln!("Unit type {:?} not recognized.", &ev.unit_type);
        }
    }
}

// TODO strongly type?
type TurretData = serde_json::Value;

fn add_subunit(
    parent: &mut ChildBuilder,
    subunit_data: &SubunitData,
    hardpoint_data: &HardpointData,
    texture_server: &Res<TextureServer>
) {
    let subunit_size = Vec2::new(subunit_data.size[0], subunit_data.size[1]);
    let subunit_pos = Vec3::new(hardpoint_data.position[0], hardpoint_data.position[1], hardpoint_data.position[2]);
    let mut ec = parent.spawn();
    ec.insert_bundle( SpatialBundle {
        transform: Transform {
            translation: Vec3::new(subunit_pos.x * SPRITE_SCALE, subunit_pos.y * SPRITE_SCALE, hardpoint_data.z_order),
            rotation: Quat::from_rotation_z( subunit_pos.z ),
                ..Default::default()
            },
        ..Default::default()
    });
    ec.with_children(|parent2| {
    for sprite_data in subunit_data.sprites.iter() {
        let sprite_size = Vec2::new(sprite_data.size[0], sprite_data.size[1]);
            parent2.spawn_bundle(SpriteBundle {
                texture: texture_server.get(&sprite_data.texture).typed::<Image>(),
                sprite: Sprite {
                    custom_size: Some(sprite_size * SPRITE_SCALE),
                    ..Default::default()
                },
                transform: Transform { translation: Vec3::new(0., 0., sprite_data.z_order), ..Default::default() },
                ..Default::default()
            });
        }
    });
    ec
    .insert(
        Body::new(
            Vec3::new(subunit_pos.x * SPRITE_SCALE, subunit_pos.y * SPRITE_SCALE, subunit_pos.z),
            subunit_size
        )
    )
    .insert(Subunit { relative_position: Vec3::new(subunit_pos.x, subunit_pos.y, 0.) } );
    match subunit_data.class.clone() {
        SubunitClassData::Turret { reload_time, acceleration, fire_range, angle_on_target, projectile, firing_pattern, sources } => {
            let mut vsources: Vec<Vec2> = Vec::with_capacity(sources.len());
            for source in sources.iter() {
                vsources.push(Vec2::new(source[0], source[1]));
            }
            ec.insert(Turret::new(
                String::from(&subunit_data.name),
                String::from(projectile),
                fire_range,
                reload_time,
                firing_pattern,
                vsources
            ))
            .insert(Velocity { ..Default::default() });
        },
        SubunitClassData::Thruster { forward_thrust, particle_lifetime, particle_position_variance, particle_angle_variance, particle_velocity_variance, particle_color, particle_sprite } => {
            ec.insert(Thruster {
                omnidirectional_thrust: 0.001,
                unidirectional_thrust: forward_thrust,
            }).insert(ParticleEmitter::new_thruster_emitter(
                particle_lifetime,
                particle_position_variance,
                particle_angle_variance,
                particle_sprite
            ));
        }
    }
}

pub fn sprite_bundle_from_data(sprite_data: &SpriteData, texture_server: &Res<TextureServer>, z_order: f32) -> SpriteBundle {
    let sprite_z = sprite_data.z_order + z_order;
    let sprite_size = Vec2::new(sprite_data.size[0], sprite_data.size[1]);
    SpriteBundle {
        texture: texture_server.get(&sprite_data.texture).typed::<Image>(),
        sprite: Sprite {
            custom_size: Some(sprite_size * SPRITE_SCALE),
            ..Default::default()
        },
        transform: Transform { translation: Vec3::new(0., 0., sprite_z), ..Default::default() },
        ..Default::default()
    }
}