use bevy::{prelude::*, ecs::system::EntityCommands};
use bevy_prototype_lyon::prelude::*;
use rand::Rng;

use crate::*;

// TODO generative
const SOLAR_RADIUS: f32 = 400.;
const N_PRIMARY_SATELLITES: i32 = 5;
const MAX_SECONDARY_SATELLITES: i32 = 3;
const SECONDARY_RADII: f32 = 450.;  // TODO randomize
const ORBITAL_RATE: f32 = 0.0001;
const ORBITAL_MARGIN: f32 = 200.;  // The distance between the furthest satellite and the edge of the map
const PLANET_NAMES: &'static [&'static str] = &["Garden", "Angus", "Orrin", "Heart", "Scrub", "Julia"];
pub const ORBITAL_RADIUS_RATIO: f32 = 8.;  // The ratio of a Planet's radius to its inertial and territorial zone

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
    let sat_size = (map_radius - ORBITAL_MARGIN * 3.) / (2. * N_PRIMARY_SATELLITES as f32);
    let mut radii: Vec<f32> = Vec::new();
    for i in 0..N_PRIMARY_SATELLITES {
        radii.push((i + 1) as f32 * sat_size + (2. * ORBITAL_MARGIN));
    }
    for (i, orbital_radius) in radii.iter().enumerate() {
        let n_moons: i32 = rand::thread_rng().gen_range(0..MAX_SECONDARY_SATELLITES);
        println!("Generating major satellite at orbital radius {}", orbital_radius);
        let r = rand::thread_rng().gen_range(0.5..1.0) * sat_size / ORBITAL_RADIUS_RATIO;
        let planet_name = PLANET_NAMES[i];
        let orbital_angle = rand::thread_rng().gen_range(0.0..(2.*PI));
        let orbital_rate = rand::thread_rng().gen_range(1.0..3.0) * ORBITAL_RATE;
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
            radius: r
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
        if n_moons > 0 {
            let moon_r = (r - (r / ORBITAL_RADIUS_RATIO)) / n_moons as f32;
            for j in 0..n_moons {
            // for i in 0..2 {
                let s2_r = rand::thread_rng().gen_range(0.7..1.0) * moon_r;
                let s2_orbital_angle = rand::thread_rng().gen_range(0.0..(2.*PI));
                let s2_orbital_rate = rand::thread_rng().gen_range(4.0..5.0) * ORBITAL_RATE * 10.;
                let s2_orbital_radius = (j + 1) as f32 * moon_r * 6.;
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
                    radius: s2_r
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
}

// Inserts graphical children
pub fn setup_environment_appearance_system(
    mut commands: Commands,
    query: Query<(Entity, &EnvironmentalSatellite, &Transform, &Orbit), With<EnvironmentalSatellite>>,
    q_transform: Query<&Transform>,
    fonts: Res<Fonts>
) {
    for (entity, planet, planet_transform, orbit) in query.iter() {
        let r_color = [Color::SALMON, Color::PURPLE, Color::AQUAMARINE, Color::BEIGE, Color::DARK_GREEN, Color::PINK][rand::thread_rng().gen_range(0..5)];
        let mut ec = commands.entity(entity);
        let text_style = TextStyle {
            font: fonts.h2.clone(),
            font_size: 30.0,
            color: Color::WHITE,
        };
        let orbit_center = q_transform.get(orbit.parent).unwrap().translation;
        ec.with_children(|parent| {
            
            // Planet Sprites
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
    
            // Display planet's name and information
            parent.spawn().insert(PlanetInfoUI)
            .insert_bundle(SpatialBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., PLANET_ZORDER + 1.),
                    ..Default::default()
                },
                visibility: Visibility { is_visible: false },
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(Text2dBundle {
                    text: Text::from_section(&planet.name.to_uppercase(), text_style.clone())
                        .with_alignment(TextAlignment::CENTER),
                    ..default()
                });
            }).insert(PlanetInfoUI);

        });
    }
}

pub fn primary_satellite_orbit_system(
    q_s0: Query<&Transform, Without<PrimarySatellite>>,
    mut q_s1: Query<(Entity, &mut Transform, &mut Orbit), With<PrimarySatellite>>
) { 
    for (entity, mut orbiter_transform, mut orbit) in q_s1.iter_mut() {
        if let Ok(parent_transform) = q_s0.get(orbit.parent) {
            let parent_position = parent_transform.translation.truncate();
            // Move orbiter
            orbit.w += orbit.rate;
            orbiter_transform.translation = Vec3::new(
                parent_position.x + orbit.w.cos() * orbit.radius,
                parent_position.y + orbit.w.sin() * orbit.radius,
                orbiter_transform.translation.z
            )
        }
    }
}

pub fn secondary_satellite_orbit_system(
    q_s1: Query<&Transform, (With<PrimarySatellite>, Without<SecondarySatellite>)>,
    mut q_s2: Query<(Entity, &mut Transform, &mut Orbit), With<SecondarySatellite>>
) { 
    for (entity, mut orbiter_transform, mut orbit) in q_s2.iter_mut() {
        if let Ok(parent_transform) = q_s1.get(orbit.parent) {
            let parent_position = parent_transform.translation.truncate();
            orbit.w += orbit.rate;
            orbiter_transform.translation = Vec3::new(
                parent_position.x + orbit.w.cos() * orbit.radius,
                parent_position.y + orbit.w.sin() * orbit.radius,
                orbiter_transform.translation.z
            )
        }
    }
}

pub fn setup_background_system(
    mut commands: Commands,
	texture_server: Res
) {
    'texture: for (i, entry) in glob::glob(&("data/bg/*.png")).expect("Fatal: Invalid pattern").enumerate() {
        match entry {
            Ok(path) => {
                if let Some(s) = path.to_str() {
                    let path_s = String::from(s).replace("\"", "").replace("assets\\", "");
                    println!("Adding background layer {}: {}", i, path_s);
                }
            },
            Err(e) => eprintln!("{:?}", e)
        }
}