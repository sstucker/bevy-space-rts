use serde::{Deserialize, Serialize};
use bevy::prelude::*;

use crate::Subunit;

// -- Subunit --------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct SubunitData {
    pub name: String,
    pub class: SubunitClassData,
    pub subclass: String,
    pub hardpoint_size: i64,
    pub size: Vec<f32>,
    pub sprites: Vec<SpriteData>
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag="name")]
pub enum SubunitClassData {
    Turret {
        reload_time: u64,
        acceleration: f32,
        fire_range: f32,
        angle_on_target: f32,
        projectile: String,
        firing_pattern: String,
        sources: Vec<Vec<f32>>
    },
    Thruster {
        forward_thrust: f32
    }
}

// -- Platform --------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct PlatformData {
    pub name: String,
    pub class: PlatformClassData,
    pub subclass: String,
    pub hp: u64,
    pub size: Vec<f32>,
    pub sight_radius: f32,
    pub teamcolor_sprite: SpriteData,
    pub sprites: Vec<SpriteData>,
    pub hardpoints: Vec<HardpointData>
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag="name")]
pub enum PlatformClassData {
    Capital {
        range_radius: f32,
        forward_burn_threshold: f32,
        lateral_drag: f32,
        radial_drag: f32
    },
    Depot {
        forward_thrust: f32
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HardpointData {
    pub class_name: String,
    pub hardpoint_size: i64,
    pub z_order: f32,
    pub position: Vec<f32>
}

// Misc

#[derive(Serialize, Deserialize, Clone)]
pub struct SpriteData {
    pub texture: String,
    pub size: Vec<f32>,
    pub z_order: f32
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectileData {
    pub name: String,
    pub class: String,
    pub subclass: String,
    pub velocity: f32,
    pub range: f32,
    pub size: Vec<f32>,
    pub sprites: Vec<SpriteData>
}


// Units are created from platforms with loadouts via Assemblies

pub struct UnitData {
    pub name: String,
    pub platform: PlatformData,
    pub loadout: Vec<SubunitData>
}

impl UnitData {
    pub fn new(name: String, platform: PlatformData, loadout: Vec<SubunitData>) -> Self {
        Self {
            name,
            platform,
            loadout
        }
    }
}
