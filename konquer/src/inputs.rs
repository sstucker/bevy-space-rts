#[allow(unused_mut)]
#[allow(unused)]
#[allow(dead_code)]

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}};
use bevy::core::FixedTimestep;
use bevy_prototype_lyon::prelude::*;
// use leafwing_input_manager::prelude::*;

// TODO config
const SELECT_RECT_THRESH: f32 = 4.;  // The size of the smallest rectangle that will be evaluated

use crate::*;

pub struct InputPlugin;

pub enum MouseAction {
    // Position in world coordinates, shift
    NoAction,
    LeftClick(Vec2, bool),
    DoubleLeftClick(Vec2, bool),
    RightClick(Vec2, bool),
    DraggingSelection(Vec2, Vec2, bool),
    ReleasedSelection(Vec2, Vec2, bool)
}

pub struct InputActions {
    pub mouse: MouseAction,
}

impl InputActions {
    pub fn new() -> InputActions {
        InputActions {
            mouse: MouseAction::NoAction,
        }
    }
}

pub struct GameActions {

}

impl GameActions {
    pub fn new() -> GameActions {
        GameActions {
            
        }
    }
}

// InputActions --> GameActions --> Engine logic

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputActions::new())
            .insert_resource(GameActions::new())
            .add_system(ui_selection_rect_system)
            .add_system(unit_click_system)
            .add_system_set(SystemSet::new() // Input 
                .with_run_criteria(FixedTimestep::step(1. / 60.))  // VSYNC
                .with_system(inputs::input_mouse_system)
                .with_system(inputs::mouse_game_actions.after(input_mouse_system))
            );
    }
}

pub fn ui_selection_rect_system(
    mut commands: Commands,
    input_actions: Res<InputActions>,
    q_rect: Query<Entity, With<SelectionRect>>,
    q_projection: Query<&OrthographicProjection, With<Camera>>,
) {
    // Delete any existing selection rect UI 
    for rect in q_rect.iter() {
        commands.entity(rect).despawn();
    }
    let projection = q_projection.single();

    if let MouseAction::DraggingSelection(held_position, current_position, _) = input_actions.mouse {
        // Create SelectionRects
        let mut path_builder = PathBuilder::new();
        path_builder.move_to(held_position);
        path_builder.line_to(Vec2::new(held_position.x, current_position.y));
        path_builder.line_to(Vec2::new(current_position.x, current_position.y));
        path_builder.line_to(Vec2::new(current_position.x, held_position.y));
        path_builder.line_to(held_position);
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
    
}


pub fn mouse_game_actions(
    input_actions: Res<InputActions>,
) {

}


pub fn input_mouse_system(
    mb: Res<Input<MouseButton>>,
    kb: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
    q_map: Query<&Map, With<Map>>,
    mut input_actions: ResMut<InputActions>,
) {

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
            let (camera, camera_transform) = q_camera.single();
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
            if mb.just_released(MouseButton::Left) {
                match input_actions.mouse {
                    MouseAction::DraggingSelection(held, released, _) => {
                        if (held - released).length() > SELECT_RECT_THRESH {
                            input_actions.mouse = MouseAction::ReleasedSelection(held, m_pos, shift)
                        } else {
                            input_actions.mouse = MouseAction::LeftClick(m_pos, shift)
                        }
                        
                    }
                    MouseAction::NoAction => input_actions.mouse = MouseAction::LeftClick(m_pos, shift),
                    _ => ()
                }
            }
            else if mb.pressed(MouseButton::Left) {
                match input_actions.mouse {
                    MouseAction::NoAction => input_actions.mouse = MouseAction::DraggingSelection(m_pos, m_pos, shift),
                    MouseAction::DraggingSelection(held, _, _) => input_actions.mouse = MouseAction::DraggingSelection(held, m_pos, shift),
                    _ => ()
                }
            }
            else if mb.just_released(MouseButton::Right) {
                match input_actions.mouse {
                    MouseAction::NoAction => input_actions.mouse = MouseAction::RightClick(m_pos, shift),
                    MouseAction::DraggingSelection(_, _, _) => input_actions.mouse = MouseAction::NoAction,
                    _ => ()
                }
            }
        }
    }
    else {  // No mouse action
        input_actions.mouse = MouseAction::NoAction;
    }
}

pub fn unit_click_system(
    input_actions: ResMut<InputActions>,
) {
    // Decode mouse actions and enqueue game actions
    match input_actions.mouse {
        MouseAction::LeftClick(p, shift) => println!("Left click at {}, {}", p.x, p.y),
        MouseAction::DoubleLeftClick(p, shift) => println!("Double left click at {}, {}", p.x, p.y),
        MouseAction::RightClick(p, shift) => println!("Right click at {}, {}", p.x, p.y),
        MouseAction::DraggingSelection(p1, p2, shift) => println!("Selection dragging at {}, {}", p2.x, p2.y),
        MouseAction::ReleasedSelection(p1, p2, shift) => println!("Selection released at {}, {}", p2.x, p2.y),
        MouseAction::NoAction => ()
    }
    // let mut clicked_unit = false;
    // for (mut controls, _, body, unit) in q_units.iter_mut() {
    //     if ((m_pos - body.position.truncate()).length() < body.selection_radius) && unit.owner.id == USER_ID {  // TODO multiplayer
    //         println!("Clicked Unit {} {} away", unit.id, (m_pos - body.position.truncate()).length());
    //         if !clicked_unit {
    //             if shift {
    //                 controls.is_selected = ! controls.is_selected;
    //             } else {
    //                 controls.is_selected = true;
    //             }
    //             clicked_unit = true;
    //         }
    //     }
    //     else {
    //         if !shift {
    //             controls.is_selected = false;
    //         }
    //     }
    // }
    // if clicked_unit {input_state.held_position = Vec2::new(f32::NAN, f32::NAN); return; }

    // let bb: Vec4 = Vec4::new(
    //     input_state.held_position[0].min(m_pos[0]),
    //     input_state.held_position[1].max(m_pos[1]),
    //     input_state.held_position[0].max(m_pos[0]),
    //     input_state.held_position[1].min(m_pos[1]),
    // );
    // println!("Box evaluated! ({}, {}), ({}, {})", bb.x, bb.y, bb.z, bb.w);
    // for (mut controls, _, body, unit) in q_units.iter_mut() {
    //     if (bb.x <= body.position.x && body.position.x <= bb.z) && (bb.y >= body.position.y && body.position.y >= bb.w) {
    //         println!("Unit {} at {}, {} in bounding box.", unit.id, body.position.x, body.position.x);
    //         if unit.owner.id == USER_ID { // TODO multiplayer
    //             println!("Unit {} is now selected!", unit.owner.id);
    //             if shift {
    //                 controls.is_selected = ! controls.is_selected;
    //             } else {
    //                 controls.is_selected = true;
    //             }
    //         }  
    //     }
    //     else {
    //         if !shift {
    //             controls.is_selected = false;
    //         }
    //     }
    // }

    // println!("Right click at {}, {}, shift {}", m_pos.x, m_pos.y, shift);
    // for (_, _, body, unit) in q_units.iter() {
    //     if ((m_pos - body.position.truncate()).length() < body.selection_radius) && unit.owner.id != USER_ID {
    //         println!("Clicked enemy unit {}!", unit.id);

    //         return;
    //     }
    // }
    // // If  a path to selected units
    // for (mut path, controls) in q_path.iter_mut() {
    //     if controls.is_selected && controls.is_movable {
    //         if shift {
    //             // TODO interface
    //             path.path.push_back(m_pos);
    //         }
    //         else {
    //             path.path.clear();
    //             path.path.push_back(m_pos);
    //         }
    //     }
    // }

}