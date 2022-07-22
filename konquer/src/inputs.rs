#[allow(unused_mut)]
#[allow(unused)]
#[allow(dead_code)]

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}};
use bevy::core::FixedTimestep;
use bevy_prototype_lyon::prelude::*;

use crate::*;


pub fn input_mouse_system(
    mut commands: Commands,
    mb: Res<Input<MouseButton>>,
    kb: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut ui: ResMut<Ui>,
    mut q_units: Query<(&mut Children, &mut UnitControls, &Transform, &Body, &Unit), With<Unit>>,
    q_camera: Query<(&OrthographicProjection, &Camera, &GlobalTransform), With<Camera>>,
    q_rect: Query<Entity, With<SelectionRect>>,
    q_map: Query<&Map, With<Map>>,
) {
    // Delete any existing selection rect UI 
    for rect in q_rect.iter() {
        commands.entity(rect).despawn();
    }
    
    // On click
    if mb.pressed(MouseButton::Left)
    || mb.just_released(MouseButton::Left)
    || mb.pressed(MouseButton::Right)
    || mb.just_released(MouseButton::Right)
     {  
        let window = windows.get_primary().unwrap();
        let map = q_map.single();
        if let Some(w_pos) = window.cursor_position() {  // If cursor is in window

            let shift: bool = kb.pressed(KeyCode::RShift) || kb.pressed(KeyCode::LShift);

            // Convert cursor position to world position
            let (projection, camera, camera_transform) = q_camera.single();
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);
            let ndc: Vec2 = (w_pos / window_size) * 2. - Vec2::ONE;
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
            let mut m_pos: Vec2 = ndc_to_world.project_point3(ndc.extend(-1.)).truncate();
            
            // Prevent selection from exceeding bounds of the world
            if m_pos[0] < -map.w as f32 / 2. {
                m_pos[0] = -map.w as f32 / 2.;
            }
            else if m_pos[0] > map.w as f32 / 2. {
                m_pos[0] = map.w as f32 / 2.;
            }
            if m_pos[1] < -map.h as f32 / 2. {
                m_pos[1] = -map.h as f32 / 2.;
            }
            else if m_pos[1] > map.h as f32 / 2. {
                m_pos[1] = map.h as f32 / 2.;
            }
            
            // If drawing selection rectangle
            if mb.pressed(MouseButton::Left) {
                
                if ui.held_position[0].is_nan() {
                    ui.held_position = m_pos;
                    println!("Held position is {}, {}", ui.held_position[0], ui.held_position[1]);
                }

                // Draw selection rect
                let mut path_builder = PathBuilder::new();
                path_builder.move_to(ui.held_position);
                path_builder.line_to(Vec2::new(ui.held_position[0], m_pos[1]));
                path_builder.line_to(Vec2::new(m_pos[0], m_pos[1]));
                path_builder.line_to(Vec2::new(m_pos[0], ui.held_position[1]));
                path_builder.line_to(ui.held_position);
                let line = path_builder.build();
                commands.spawn_bundle(GeometryBuilder::build_as(
                    &line,
                    DrawMode::Stroke(StrokeMode::new(
                        Color::YELLOW,
                        2.0 * projection.scale  // Always draw the same thickness of UI elements regardless of zoom
                    )),
                    Transform { ..Default::default()  },
                )).insert( SelectionRect );
            }

            // If we released mouse, check if we clicked unit or evaluate the selection box
            else if mb.just_released(MouseButton::Left) {
                // If we clicked a unit, select it instead of evaluating the rectangle
                let mut clicked_unit = false;
                for (_, mut controls, _, body, unit) in q_units.iter_mut() {
                    if ((m_pos - body.position.truncate()).length() < body.selection_radius) && unit.owner.id == USER_ID {  // TODO multiplayer
                        println!("Clicked Unit {} {} away", unit.id, (m_pos - body.position.truncate()).length());
                        if !clicked_unit {
                            if shift {
                                controls.is_selected = ! controls.is_selected;
                            } else {
                                controls.is_selected = true;
                            }
                            clicked_unit = true;
                        }
                    }
                    else {
                        if !shift {
                            controls.is_selected = false;
                        }
                    }
                }
                if clicked_unit {ui.held_position = Vec2::new(f32::NAN, f32::NAN); return; }

                let bb: Vec4 = Vec4::new(
                    ui.held_position[0].min(m_pos[0]),
                    ui.held_position[1].max(m_pos[1]),
                    ui.held_position[0].max(m_pos[0]),
                    ui.held_position[1].min(m_pos[1]),
                );
                println!("Box evaluated! ({}, {}), ({}, {})", bb.x, bb.y, bb.z, bb.w);
                for (_, mut controls, _, body, unit) in q_units.iter_mut() {
                    if (bb.x <= body.position.x && body.position.x <= bb.z) && (bb.y >= body.position.y && body.position.y >= bb.w) {
                        println!("Unit {} at {}, {} in bounding box.", unit.id, body.position.x, body.position.x);
                        if unit.owner.id == USER_ID { // TODO multiplayer
                            println!("Unit {} is now selected!", unit.owner.id);
                            if shift {
                                controls.is_selected = ! controls.is_selected;
                            } else {
                                controls.is_selected = true;
                            }
                        }  
                    }
                    else {
                        if !shift {
                            controls.is_selected = false;
                        }
                    }
                }
                ui.held_position = Vec2::new(f32::NAN, f32::NAN);
            }
            // Right click means assign path or target to units
            else if mb.just_released(MouseButton::Right) {
                println!("Right click at {}, {}, shift {}", m_pos.x, m_pos.y, shift);
                for (_, _, _, body, unit) in q_units.iter() {
                    if ((m_pos - body.position.truncate()).length() < body.selection_radius) && unit.owner.id != USER_ID {
                        println!("Clicked enemy unit {}!", unit.id);
                        return;
                    }
                }
                // Assign a path to selected units
                for (_, mut controls, _, body, unit) in q_units.iter_mut() {
                    if controls.is_selected && controls.is_movable {
                        if shift {
                            controls.path.push_back(m_pos);
                        }
                        else {
                            controls.path.clear();
                            controls.path.push_back(m_pos);
                        }
                    }
                }
            }
        }
    }
}
