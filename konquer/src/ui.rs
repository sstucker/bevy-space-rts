use std::f32::consts::E;

use bevy_prototype_lyon::prelude::*;
use bevy::{prelude::*, ecs::query, input::mouse::MouseMotion, text::FontLoader};

use crate::{*, inputs::MouseOverEvent};

pub fn ui_highlight_selected_system(
    mut commands: Commands,
    q_circ: Query<Entity, With<UnitSelectedCircle>>,
    q_units: Query<(Entity, &Body, &Unit), With<Selected>>,
    q_camera: Query<&OrthographicProjection, With<Camera>>,
) {
    for circ in q_circ.iter() {
        commands.entity(circ).despawn();
    }
    let projection = q_camera.single();
    for (entity, body, unit) in q_units.iter() {
        let mut ec = commands.entity(entity);
        let sel_color = match unit.player.id {
            USER_ID => Color::GREEN,
            _ => Color::RED
        };
        ec.with_children(|parent| {
            parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                sides: 60,
                feature: shapes::RegularPolygonFeature::Radius((body.size[0] + body.size[1]) * SPRITE_SCALE / 3.),
                ..shapes::RegularPolygon::default()
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::rgba(0., 0., 0., 0.)),
                outline_mode: StrokeMode::new(sel_color, 1. * projection.scale),
            },
            Transform { translation: Vec3::new(0., 0., -1.), ..Default::default() },
        )).insert(UnitSelectedCircle);
        });
    }
}

const HEALTHBAR_HEIGHT: f32 = 1.;
const HEALTHBAR_WIDTH: f32 = 20.;
const HEALTHBAR_MARGIN: f32 = 1.;

pub fn ui_show_hp_system(
    mut commands: Commands,
    q_units: Query<(&Unit, &Hp, &Body), With<Hp>>,
    q_healthbar: Query<Entity, With<HealthBar>>,
    q_camera: Query<&OrthographicProjection, With<Camera>>,
) {
    for bar in q_healthbar.iter() {
        commands.entity(bar).despawn();
    }
    let projection = q_camera.single();
    for (unit, hp, body) in q_units.iter() {
        let hp_color = match unit.player.id {
            USER_ID => Color::rgba(0.1, 1.0, 0.1, 1.0),
            _ => Color::rgba(1.0, 0.0, 0.0, 1.0)
        };
        let mut rect: Vec<Vec2> = vec!(
            Vec2::new(HEALTHBAR_WIDTH, 0.),
            Vec2::new(HEALTHBAR_WIDTH, HEALTHBAR_HEIGHT),
            Vec2::new(0., HEALTHBAR_HEIGHT),
            Vec2::new(0., 0.)
        );
        let mut path_builder = PathBuilder::new();
        let p = body.position.truncate() - Vec2::new(HEALTHBAR_WIDTH / 2., body.size.y * 1.2 * SPRITE_SCALE);
        path_builder.move_to(p);
        for v in rect.iter() {
            path_builder.line_to(p + *v);
        }
        let outline = path_builder.build();
        let fill = HEALTHBAR_WIDTH * (hp.current as f32 / hp.max as f32);
        path_builder = PathBuilder::new();
        rect[0].x = fill;
        rect[1].x = fill;
        for v in rect.iter() {
            path_builder.line_to(p + *v);
        }
        let healthbar = path_builder.build();
        commands.spawn_bundle(GeometryBuilder::build_as(
            &outline,
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::rgba(0.4, 0.4, 0.4, 0.5)),
                outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 1.),
            },
            Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER + 9.), ..Default::default() },
        )).insert( HealthBar );
        commands.spawn_bundle(GeometryBuilder::build_as(
            &healthbar,
            DrawMode::Outlined {
                fill_mode: FillMode::color(hp_color),
                outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 1.),
            },
            Transform { translation: Vec3::new(0., 0., UI_ABOVE_ZORDER + 10.), ..Default::default() },
        )).insert( HealthBar );
    }
}


pub fn ui_show_path_system(
    mut commands: Commands,
    q_paths: Query<Entity, With<UnitPathDisplay>>,
    q_units: Query<(&Unit, &UnitPath, &Body), With<UnitPath>>,
    q_camera: Query<&OrthographicProjection, With<Camera>>,
) {
    for path in q_paths.iter() {
        commands.entity(path).despawn();
    }
    let projection = q_camera.single();
    for (unit, path, body) in q_units.iter() {
        if !path.path.is_empty() && unit.player.id == USER_ID {  // Only show paths for friendlies for now
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(body.position.truncate());
            for point in path.path.iter() {
                path_builder.line_to(*point);
            }
            let line = path_builder.build();
            commands.spawn_bundle(GeometryBuilder::build_as(
                &line,
                DrawMode::Stroke(StrokeMode::new(
                    Color::rgba(1., 1., 0., 0.3),
                    1. * projection.scale  // Always draw the same thickness of UI elements regardless of zoom
                )),
                Transform { translation: Vec3::new(0., 0., UI_BENEATH_ZORDER), ..Default::default() },
            )).insert( UnitPathDisplay );
        }
    }
}

pub fn ui_fps_system(

) {
    
}

pub fn ui_planet_system(
    mut commands: Commands,
    q_orbit_ui: Query<Entity, With<PlanetOrbitUI>>, 
    mut q_info_ui: Query<(&mut Visibility, &mut Transform), (With<PlanetInfoUI>, Without<EnvironmentalSatellite>)>, 
    q_planets: Query<(Entity, &EnvironmentalSatellite, &Children, &Orbit, &Transform), With<EnvironmentalSatellite>>, 
    q_primary_sat: Query<&PrimarySatellite>, 
    q_transform: Query<&Transform, Without<PlanetInfoUI>>, 
    q_camera: Query<&OrthographicProjection, With<Camera>>,
    mouseover_lel: Local<Events<MouseOverEvent>>,
    mouseover_events: ResMut<Events<MouseOverEvent>>,
) {
    let mut mouseover_el = mouseover_lel.get_reader();
    let scale_factor = q_camera.single().scale;
    // println!("Scale factor is {}", scale_factor);
    if !mouseover_el.is_empty(&mouseover_events) {
        for entity in q_orbit_ui.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for (mut vis, _) in q_info_ui.iter_mut() {
            vis.is_visible = false;
        }
    }
    for event in mouseover_el.iter(&mouseover_events) {
        for (e_planet, planet, children, orbit, planet_transform) in q_planets.iter() {
            if event.pos.distance(planet_transform.translation.truncate()) < planet.radius {
                let mut orbit_center = q_transform.get(orbit.parent).unwrap().translation;
                orbit_center.z = WORLD_ZORDER + 1.;

                // Display planet's orbit
                commands.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                    sides: 128,
                    feature: shapes::RegularPolygonFeature::Radius(orbit.radius),
                    ..shapes::RegularPolygon::default()
                    },
                    DrawMode::Outlined {
                        fill_mode: FillMode::color(Color::rgba(0., 0., 0., 0.)),
                        outline_mode: StrokeMode::new(Color::rgba(0.1, 1., 1., 1.), 2. * scale_factor),
                    },
                    Transform { translation: orbit_center, ..Default::default() },
                )).insert(PlanetOrbitUI);

                // Display planet's orbital radius
                if let Ok(primary_sat) = q_primary_sat.get(e_planet) {
                    commands.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                        sides: 128,
                        feature: shapes::RegularPolygonFeature::Radius(primary_sat.gravity_radius),
                        ..shapes::RegularPolygon::default()
                        },
                        DrawMode::Outlined {
                            fill_mode: FillMode::color(Color::rgba(0.1, 0.1, 0.1, 0.9)),
                            outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 1.),
                        },
                        Transform { translation: planet_transform.translation.truncate().extend(WORLD_ZORDER + 2.), ..Default::default() },
                    )).insert(PlanetOrbitUI);
                }


                // Unhide the info
                for child in children.iter() {
                    if let Ok((mut vis, mut transform)) = q_info_ui.get_mut(*child) {
                        transform.translation = Vec3::new(100., -100., 0.) * scale_factor.min(4.5);
                        transform.scale = Vec3::new(1., 1., 0.) * scale_factor;
                        vis.is_visible = true;
                    }
                }
                return
            }
        }
    }
}
