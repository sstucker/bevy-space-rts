use std::f32::consts::E;

use bevy_prototype_lyon::prelude::*;
use bevy::{prelude::*, ecs::query};

use crate::*;

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
            Transform { translation: Vec3::new(0., 0., UI_ZORDER), ..Default::default() },
        )).insert(UnitSelectedCircle);
        });
    }
}

const HEALTHBAR_HEIGHT: f32 = 6.;
const HEALTHBAR_WIDTH: f32 = 80.;
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
            Transform { translation: Vec3::new(0., 0., UI_ZORDER + 9.), ..Default::default() },
        )).insert( HealthBar );
        commands.spawn_bundle(GeometryBuilder::build_as(
            &healthbar,
            DrawMode::Outlined {
                fill_mode: FillMode::color(hp_color),
                outline_mode: StrokeMode::new(Color::rgba(0., 0., 0., 0.), 1.),
            },
            Transform { translation: Vec3::new(0., 0., UI_ZORDER + 10.), ..Default::default() },
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
                Transform { translation: Vec3::new(0., 0., 5.), ..Default::default() },
            )).insert( UnitPathDisplay );
        }
    }
}