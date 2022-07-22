use std::f32::consts::E;

use bevy::{prelude::*, input::mouse::{MouseMotion, MouseButtonInput}, ecs::query};
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

// 
pub struct ActionEvent {
    user: Player,
    action: Action,
}

pub enum Action {
    AddTargets(Vec<KindedEntity<Unit>>, Vec<KindedEntity<Unit>>),
    ClearTargets(Vec<KindedEntity<Unit>>),
}

// InputActions --> GameActions --> Engine logic

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(InputActions::new())
            .add_event::<ActionEvent>()
            .add_system(ui_selection_rect_system)
            .add_system_set(SystemSet::new() // Input 
                .with_run_criteria(FixedTimestep::step(1. / 60.))  // VSYNC
                .with_system(inputs::input_mouse_system)
                .with_system(inputs::decode_action_system)
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
            if m_pos.x < -map.w as f32 / 2. {
                m_pos.x = -map.w as f32 / 2.;
            }
            else if m_pos.x > map.w as f32 / 2. {
                m_pos.x = map.w as f32 / 2.;
            }
            if m_pos.y < -map.h as f32 / 2. {
                m_pos.y = -map.h as f32 / 2.;
            }
            else if m_pos.y > map.h as f32 / 2. {
                m_pos.y = map.h as f32 / 2.;
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

pub fn decode_action_system(
    mut commands: Commands,
    input_actions: ResMut<InputActions>,
    q_selectable: Query<(Entity, &Body), With<Selectable>>,
    q_selected: Query<&Selected>,
) {
    // Decode mouse actions and enqueue game actions
    match input_actions.mouse {
        MouseAction::LeftClick(click_point, shift) => {
            println!("Left click at {}, {}", click_point.x, click_point.y);
            // For all selectable units
            let mut selected_a_unit = false;
            for (entity, body) in q_selectable.iter() {
                // Only select one unit per action... TODO consider ZORDER?
                if !selected_a_unit && (click_point - body.position.truncate()).length() < body.selection_radius {
                    // If the clicked unit is already selected
                    if let Ok(_) = q_selected.get(entity) {
                        if shift {
                            // We need to toggle
                            commands.entity(entity).remove::<Selected>();
                            return;
                        }
                        // else we do nothing except deselect all other units
                    }
                    // If the clicked unit is not selected
                    else {
                        selected_a_unit = true;  // Only select one
                        commands.entity(entity).insert(Selected);
                    }
                }
                else if !shift {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        },
        MouseAction::DoubleLeftClick(p, shift) => {
            println!("Double left click at {}, {}", p.x, p.y)
        },
        MouseAction::RightClick(p, shift) => {
            println!("Right click at {}, {}", p.x, p.y)
        },
        MouseAction::DraggingSelection(p1, p2, shift) => {
            // println!("Selection dragging at {}, {}", p2.x, p2.y)
        },
        MouseAction::ReleasedSelection(p1, p2, shift) => {
            println!("Selection released at {}, {}", p2.x, p2.y);
            let bb: Vec4 = Vec4::new(
                p1.x.min(p2.x),
                p1.y.max(p2.y),
                p1.x.max(p2.x),
                p1.y.min(p2.y),
            );
            println!("Box evaluated! ({}, {}), ({}, {})", bb.x, bb.y, bb.z, bb.w);
            for (entity, body) in q_selectable.iter() {
                if (bb.x <= body.position.x && body.position.x <= bb.z) && (bb.y >= body.position.y && body.position.y >= bb.w) {
                    if let Ok(_) = q_selected.get(entity) {
                        if shift {
                            commands.entity(entity).remove::<Selected>();
                        }
                    }
                    else {
                        commands.entity(entity).insert(Selected);
                    }
                }
                else if !shift {
                    commands.entity(entity).remove::<Selected>();
                }
            }
        },
        MouseAction::NoAction => ()
    }
    
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