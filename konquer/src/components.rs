use std::{collections::VecDeque, marker::PhantomData};
use bevy::{prelude::{Component, Entity, Color}, math::{Vec2, Vec3}, ecs::{archetype::Archetypes, component::ComponentId}};
use std::{sync::atomic::{AtomicU8, Ordering}};
use crate::{Player, SPRITE_SCALE};


pub fn get_components_for_entity<'a>(
    entity: &Entity,
    archetypes: &'a Archetypes,
) -> Option<impl Iterator<Item = ComponentId> + 'a> {
    for archetype in archetypes.iter() {
        if archetype.entities().contains(entity) {
            return Some(archetype.components());
        }
    }
    None
}

#[derive(Component, Clone, Copy)]
pub struct Map {
    pub w: i32,  // The human-readable name of the unit
    pub h: i32,  // The player of the unit
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
pub struct DebugCollisionRadius;

#[derive(Component)]
pub struct DebugRepulsionRadius;

#[derive(Component)]
pub struct DebugCollisionCheckLine;

#[derive(Component)]
pub struct GridLine;

#[derive(Component)]
pub struct MainSprite;


#[derive(Component)]
pub struct TeamSprite {
    pub color: Color
}

#[derive(Component)]
pub struct UnitPathDisplay;

#[derive(Component)]
pub struct CapitalShip;

#[derive(Component)]
pub struct Turret {
    pub reload_time: f32
}

#[derive(Component, Clone, Copy)]
pub struct Body {
	pub position: Vec3,  // x, y, w
    pub size: Vec2, // x, y
    pub selection_radius: f32,
    pub collision_radius: f32,
    pub repulsion_radius: f32
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
            repulsion_radius: (size.x + size.y) * SPRITE_SCALE / 2.5,
            selection_radius: (size.x + size.y) * SPRITE_SCALE / 4.,
            collision_radius: (size.x + size.y) * SPRITE_SCALE / 5.,
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
    pub max: u64,
    pub current: u64,
}

#[derive(Component)]
pub struct Shield {
    pub max: u64,
    pub current: u64,
}

#[derive(Component)]
pub struct UnitControls {
    pub is_selected: bool,
    pub is_clickable: bool,
    pub is_movable: bool,
}

// TODO Selected by an player?
#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Selectable;


/*
Movable means can be given path via manual UI modification. Some units like
strikecraft or other Players units move along paths but are not "Movable" by players.
*/
#[derive(Component)]
pub struct Movable;

pub type UnitPathNodes = VecDeque<Vec2>;

#[derive(Component)]
pub struct UnitPath {
    pub path: UnitPathNodes,
}

impl UnitPath {
    pub fn new() -> UnitPath {
        UnitPath { path: UnitPathNodes::new() }
    }
}

// Wrapper for Unit references
pub struct KindedEntity<T>(Entity, PhantomData<T>);

#[derive(Component)]
pub struct DebugTurretTargetLine;

/*
Targeteeable means can be targeted.
*/
#[derive(Component)]
pub struct Targeteeable;

/*
Targeterable means can be given targets via manual UI modification, not by parents, other Players, or AI.
*/
#[derive(Component)]
pub struct Targeterable;

#[derive(Component)]
pub struct Targets {
    // pub deque: VecDeque<KindedEntity<Unit>>  // Deque of targets
    pub deque: VecDeque<Entity>  // TODO figure out how to get KindedEntity pattern to work
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
    pub player: Player,  // The player of the unit
    pub id: u8,  // The global identifying number of the unit
}

/*
Anything that can be manually constructed, fire or move is a unit. Passive structures are not units.
*/
impl Unit {
    pub fn new(name: String, player: Player) -> Unit {
        Unit {
            name: name,
            player: player,
            id: NUMBER_OF_UNITS.fetch_add(1, Ordering::Relaxed)
        }
    }
}

/*
A subunit is a child of single unit. Usually this is a module with its own sprite and
behavior, like a turret, engine, or additional hitbox. FlockMembers are NOT Subunits.
Subunits are translated and rotated by independent systems.
*/
#[derive(Component)]
pub struct Subunit {
    pub relative_position: Vec3,
}

impl Subunit {
    pub fn get_absolute_position(&self, subunit_position: Vec3, parent_position: Vec3) -> Vec3 {
        let mut abs_pos: Vec3 = Vec3::from(parent_position);
        abs_pos.x += subunit_position.x * f32::cos(parent_position.z) - subunit_position.y * f32::sin(parent_position.z);
        abs_pos.y += subunit_position.x * f32::sin(parent_position.z) + subunit_position.y * f32::cos(parent_position.z);
        abs_pos
    }
}
