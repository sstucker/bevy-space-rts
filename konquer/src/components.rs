use bevy::{prelude::Component, math::{Vec2, Vec3}};

use crate::Owner;

#[derive(Component)]
pub struct Map {
    pub w: i32,  // The human-readable name of the unit
    pub h: i32,  // The owner of the unit
}

#[derive(Component)]
pub struct SelectionRect;

#[derive(Component)]
pub struct Body {
	pub position: Vec3,  // x, y, w
    pub size: Vec2, // x, y
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
pub struct OrthographicVelocity {
	pub dx: f32,
	pub dy: f32,
    pub dz: f32,
    pub dw: f32,  // Angular velocity
}

impl Default for OrthographicVelocity {
    fn default() -> Self {
        Self {
            dx: 0.,
            dy: 0.,
            dz: 0.,
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
}

impl Default for UnitControls {
    fn default() -> Self {
        Self {
            is_selected: false,
        }
    }
}

#[derive(Component)]
pub struct Unit {
    pub name: String,  // The human-readable name of the unit
    pub owner: Owner,  // The owner of the unit
    pub id: u8  // The global identifying number of the unit
}

