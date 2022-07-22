use std::collections::VecDeque;

use bevy::{prelude::{Component, Entity}, math::{Vec2, Vec3}};

use std::{sync::atomic::{AtomicU8, Ordering}};

use crate::{Owner, SPRITE_SCALE};

#[derive(Component, Clone, Copy)]
pub struct Map {
    pub w: i32,  // The human-readable name of the unit
    pub h: i32,  // The owner of the unit
}

#[derive(Component)]
pub struct SelectionRect;

#[derive(Component)]
pub struct UnitSelectedCircle;

#[derive(Component)]
pub struct DebugRect;

#[derive(Component)]
pub struct DebugSelectionRadius;

#[derive(Component)]
pub struct GridLine;

#[derive(Component)]
pub struct MainSprite;

#[derive(Component)]
pub struct UnitPathDisplay;

#[derive(Component)]
pub struct Turret {
    pub reload_time: f32
}

#[derive(Component)]
pub struct Body {
	pub position: Vec3,  // x, y, w
    pub size: Vec2, // x, y
    pub selection_radius: f32
}

#[derive(Component)]
pub struct Thruster {
	pub unidirectional_thrust: f32,
    pub omnidirectional_thrust: f32
}

impl Body {
    pub fn new(position: Vec3, size: Vec2) -> Body {
        Body {
            position: position,
            size: size,
            selection_radius: (size.x + size.y) / 4. * SPRITE_SCALE
        }
    }
}

#[derive(Component)]
pub struct Velocity {
	pub dx: f32,
	pub dy: f32,
    pub dw: f32,  // Angular velocity
}

impl Default for Velocity {
    fn default() -> Self {
        Self {
            dx: 0.,
            dy: 0.,
            dw: 0.,
        }
    }
}

#[derive(Component)]
pub struct Hp {
    pub max: u8,
    pub current: u8,
}

#[derive(Component)]
pub struct Shield {
    pub max: u8,
    pub current: u8,
}

#[derive(Component)]
pub struct UnitControls {
    pub is_selected: bool,
    pub is_movable: bool,
}

impl UnitControls {

    pub fn new(movable: bool) -> UnitControls {
        let uc = UnitControls {
            is_selected: false,
            is_movable: movable,
        };
        uc
    }
}

#[derive(Component)]
pub struct UnitPath {
    pub path: VecDeque<Vec2>,
}

impl UnitPath {
    pub fn new() -> UnitPath {
        UnitPath { path: VecDeque::new() }
    }
}

type EntityID = u32;

#[derive(Component)]
pub struct Targets {
    pub deque: VecDeque<Entity>  // Deque of targets
}

impl Targets {
    pub fn new() -> Targets {
        Targets { deque: VecDeque::new() }
    }
}

#[derive(Component)]
pub struct Range {
    pub sight: f32,  // The human-readable name of the unit
    pub fire: f32  // Range at which the unit can fire
}


static NUMBER_OF_UNITS: AtomicU8 = AtomicU8::new(0);


#[derive(Component)]
pub struct Unit {
    pub name: String,  // The human-readable name of the unit
    pub owner: Owner,  // The owner of the unit
    pub id: u8,  // The global identifying number of the unit
}

impl Unit {
    pub fn new(name: String, owner: Owner) -> Unit {
        Unit {
            name: name,
            owner: owner,
            id: NUMBER_OF_UNITS.fetch_add(1, Ordering::Relaxed)
        }
    }
}

#[derive(Component)]
pub struct Subunit;
