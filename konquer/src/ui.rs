use bevy_prototype_lyon::prelude::*;
use bevy::prelude::*;

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
        ec.with_children(|parent| {
            parent.spawn_bundle(GeometryBuilder::build_as(&shapes::RegularPolygon {
                sides: 30,
                feature: shapes::RegularPolygonFeature::Radius((body.size[0] + body.size[1]) / 3.),
                ..shapes::RegularPolygon::default()
            },
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::rgba(0., 0., 0., 0.)),
                outline_mode: StrokeMode::new(Color::GREEN, 20. * projection.scale),
            },
            Transform { translation: Vec3::new(0., 0., UI_ZORDER), ..Default::default() },
        )).insert(UnitSelectedCircle);
        });
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