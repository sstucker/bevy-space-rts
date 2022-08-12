use bevy::{prelude::*, ecs::system::EntityCommands};
use bevy_prototype_lyon::prelude::*;
use rand::Rng;

use crate::*;

// TODO generative
const SOLAR_RADIUS: f32 = 400.;
const N_PRIMARY_SATELLITES: i32 = 5;
const MAX_SECONDARY_SATELLITES: i32 = 3;
const MAX_PRIMARY_SATELLITE_SIZE: f32 = 150.;
const MIN_PRIMARY_SATELLITE_SIZE: f32 = 100.;
const MAX_SECONDARY_SATELLITE_SIZE: f32 = 50.;
const MIN_SECONDARY_SATELLITE_SIZE: f32 = 25.;
const SECONDARY_RADII: f32 = 450.;  // TODO randomize
const ORBITAL_RATE: f32 = 0.001;
const ORBITAL_MARGIN: f32 = 100.;  // The distance between the furthest satellite and the edge of the map
const PLANET_NAMES: &'static [&'static str] = &["Garden", "Angus", "Orrin", "Heart", "Scrub", "Julia"];

// TODO fix dimensions
// TODO more planet mechanics
pub fn setup_environment_system(
    mut commands: Commands,
	asset_server: Res<AssetServer>,
	texture_server: Res<TextureServer>,
) { 
    // TODO make generative, staging from menu, settings, etc
    let map: Map = Map { w: MAP_W, h: MAP_H };
	commands.spawn().insert(map);

    // Insert sun. Do appearance stuff here but only for sun
    // Would be sick to do binary systems...
    let e_sun = commands.spawn().insert(Sun).insert_bundle( SpatialBundle {
        transform: Transform {
            translation: Vec3::new(MAP_W as f32 / 2., MAP_H as f32 / 2., PLANET_ZORDER),
            ..Default::default()
        },
        ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
            sides: 90,
            feature: shapes::RegularPolygonFeature::Radius(SOLAR_RADIUS),
            ..shapes::RegularPolygon::default()
        },
        DrawMode::Outlined {
            fill_mode: FillMode::color(Color::YELLOW),
            outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 1.),
        },
        Transform { translation: Vec3::new(0., 0., 0.), ..Default::default() },
        ));
    }).id();

    // Insert satellites
    let map_radius = (MAP_H as f32 / 2.).min(MAP_W as f32 / 2.);
    let mut radii: Vec<f32> = Vec::new();
    for i in 0..N_PRIMARY_SATELLITES {
        radii.push((i as f32 + 2.) * (map_radius - ORBITAL_MARGIN * 2.) / N_PRIMARY_SATELLITES as f32);
    }
    for (i, orbital_radius) in radii.iter().enumerate() {
        println!("Generating major satellite at orbital radius {}", orbital_radius);
        let r = rand::thread_rng().gen_range(MIN_PRIMARY_SATELLITE_SIZE..MAX_PRIMARY_SATELLITE_SIZE);
        let planet_name = PLANET_NAMES[i];
        let orbital_angle = rand::thread_rng().gen_range(0.0..(2.*PI));
        let orbital_rate = rand::thread_rng().gen_range(1.0..2.0) * ORBITAL_RATE;
        let position = Vec3::new(
            MAP_W as f32 / 2. + orbital_angle.cos() * orbital_radius,
            MAP_H as f32 / 2. + orbital_angle.sin() * orbital_radius,
            PLANET_ZORDER
        );
        let mut ec_planet = commands.spawn();
        ec_planet
        .insert(PrimarySatellite)
        .insert(Orbiter)
        .insert(EnvironmentalSatellite {
            name: planet_name.to_string(),
            class: "Planet".to_string(),
            radius: r,
            gravity_radius: r * 3.
        })
        .insert(Orbit {
            parent: e_sun,
            radius: orbital_radius.clone(),
            w: orbital_angle,
            rate: orbital_rate,  // Radians per frame
        })
        .insert_bundle(SpatialBundle {
            transform: Transform {
                translation: position,
                ..Default::default()
            },
            ..Default::default()      
        });
        let e_planet = ec_planet.id();
        for i in 0..rand::thread_rng().gen_range(0..MAX_SECONDARY_SATELLITES) {
        // for i in 0..2 {
            let s2_r = rand::thread_rng().gen_range(MIN_SECONDARY_SATELLITE_SIZE..MAX_SECONDARY_SATELLITE_SIZE);
            let s2_orbital_angle = rand::thread_rng().gen_range(0.0..(2.*PI));
            let s2_orbital_rate = rand::thread_rng().gen_range(4.0..5.0) * ORBITAL_RATE * 10.;
            let s2_orbital_radius = SECONDARY_RADII * (i as f32 + 1.);
            let mut s2_position = position.clone();
            s2_position.x += s2_orbital_angle.cos() * s2_orbital_radius;
            s2_position.y += s2_orbital_angle.sin() * s2_orbital_radius;
            let mut ec_moon = commands.spawn();
            ec_moon
            .insert(SecondarySatellite)
            .insert(Orbiter)
            .insert(EnvironmentalSatellite {
                name: planet_name.to_string() + " " + &i.to_string(),  // TODO generative moon names
                class: "Moon".to_string(),
                radius: s2_r,
                gravity_radius: s2_r * 2.
            })
            .insert(Orbit {
                parent: e_planet,
                radius: s2_orbital_radius,
                w: s2_orbital_angle,
                rate: s2_orbital_rate,  // Radians per frame
            })
            .insert_bundle(SpatialBundle {
                transform: Transform {
                    translation: s2_position,
                    ..Default::default()
                },
                ..Default::default()      
            });
            let e_moon = ec_moon.id();
        }
    }
}

// Inserts graphical children
pub fn setup_environment_appearance_system(
    mut commands: Commands,
    query: Query<(Entity, &EnvironmentalSatellite, &Orbit), With<EnvironmentalSatellite>>
) {
    for (entity, planet, orbit) in query.iter() {
        let r_color = [Color::SALMON, Color::PURPLE, Color::AQUAMARINE, Color::BEIGE, Color::DARK_GREEN, Color::PINK][rand::thread_rng().gen_range(0..5)];
        let mut ec = commands.entity(entity);
        ec.with_children(|parent| {
            // Planet
            parent.spawn_bundle(GeometryBuilder::build_as(
                &shapes::RegularPolygon {
                sides: 60,
                feature: shapes::RegularPolygonFeature::Radius(planet.radius),
                ..shapes::RegularPolygon::default()
                },
                DrawMode::Outlined {
                    fill_mode: FillMode::color(r_color),
                    outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 1.),
                },
                Transform { translation: Vec3::new(0., 0., 0.), ..Default::default() },
            ));
        });
    }
    // ec
    //     // UI
    //     let ui_pos = position + Vec2::new(70.0, 50.0);
    //     let text_style = TextStyle {
    //         font: fonts.h2.clone(),
    //         font_size: 30.0 * scale_factor,
    //         color: Color::WHITE,
    //     };
    //     // Display planet's orbit
    //     commands.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
    //         sides: 60,
    //         feature: shapes::RegularPolygonFeature::Radius(orbit.radius),
    //         ..shapes::RegularPolygon::default()
    //         },
    //         DrawMode::Outlined {
    //             fill_mode: FillMode::color(Color::rgba(0., 0., 0., 0.)),
    //             outline_mode: StrokeMode::new(Color::rgba(0.1, 1., 1., 1.), 2. * scale_factor),
    //         },
    //         Transform { translation: orbit_center, ..Default::default() },
    //     )).insert(PlanetUI);

    //     // Display planet's name and information
    //     commands.spawn().insert(PlanetUI)
    //     .insert_bundle(SpatialBundle {
    //         transform: Transform {
    //             translation: Vec3::new(ui_pos.x, ui_pos.y, PLANET_ZORDER + 1.),
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .with_children(|parent| {
    //         parent.spawn_bundle(Text2dBundle {
    //             text: Text::from_section(&planet.name.to_uppercase(), text_style.clone())
    //                 .with_alignment(TextAlignment::CENTER),
    //             ..default()
    //         });
    //     });
    // });
}