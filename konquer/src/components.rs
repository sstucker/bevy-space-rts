use std::{collections::VecDeque, marker::PhantomData, time::Duration};
use bevy::{prelude::{Component, Entity, Color}, math::{Vec2, Vec3}, ecs::{archetype::Archetypes, component::ComponentId}, time::Timer};
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

// An orbiter is any satellite or body with satellites.
#[derive(Component, Clone, Copy)]
pub struct Orbiter;

#[derive(Component, Clone, Copy)]
pub struct Sun;

// Planets
#[derive(Component, Clone, Copy)]
pub struct PrimarySatellite;

// Moons
#[derive(Component, Clone, Copy)]
pub struct SecondarySatellite;

// Space stations, asteroid belts
#[derive(Component, Clone, Copy)]
pub struct TertiarySatellite;

#[derive(Component, Clone)]
pub struct PlanetUI;

#[derive(Component, Clone)]
pub struct Planet {
    pub name: String,
    pub radius: f32,
    pub gravity_radius: f32
}


#[derive(Component, Clone, Copy)]
pub struct Orbit {
    pub parent: Entity,  // TypedEntity pattern!
    pub radius: f32,
    pub w: f32,
    pub rate: f32,
}

#[derive(Component, Clone, Copy)]
pub struct Map {
    pub w: i32,  // Map width
    pub h: i32,  // Map height
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
pub struct DebugProjectileCollisionCheckLine;

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
pub struct ProjectileSprite;

#[derive(Component)]
pub struct Projectile {
    pub player: Player,
    pub fired_from: Vec2,
    pub range: f32,
    pub damage: f32
}


#[derive(Component)]
pub struct Turret {
    pub name: String,
    pub projectile: String,
    pub range: f32,
    pub reload_time: u64,
    pub timer: Timer,
    pub firing_pattern: String,
    pub sources: Vec<Vec2>,
    source_index: usize
}

impl Turret {
    pub fn new(name: String, projectile: String, range: f32, reload_time: u64, firing_pattern: String, sources: Vec<Vec2>) -> Self {
        Self {
            name,
            projectile,
            reload_time,
            range,
            timer: Timer::new(Duration::from_millis(reload_time), false),
            firing_pattern,
            sources,
            source_index: 0
        }
    }
    pub fn get_sources(&self) -> Vec<Vec2> {
        match self.firing_pattern.as_str() {
            "alternating" => {
                let mut v: Vec<Vec2> = Vec::new();
                v.push(self.sources[self.source_index]);
                return v
            },
            _ => return self.sources.clone()
        }
    }
    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }
    pub fn ready(&self) -> bool {
        self.timer.finished()
    }
    pub fn reload(&mut self) {
        self.timer = Timer::new(Duration::from_millis(self.reload_time), false);
        self.source_index = (self.source_index + 1).rem_euclid(self.sources.len());
    }

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
            collision_radius: (size.x + size.y) * SPRITE_SCALE / 3.,
            selection_radius: (size.x + size.y) * SPRITE_SCALE / 3.,
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
pub struct HealthBar;

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


// Targets is a deque of Entity instances that is cycled through by each unit
#[derive(Component)]
pub struct Targets {
    // pub deque: VecDeque<KindedEntity<Unit>>  // Deque of targets
    deque: VecDeque<Entity>  // TODO figure out how to get KindedEntity pattern to work
}

impl Targets {
    pub fn new() -> Targets {
        Targets { deque: VecDeque::new() }
    }
    pub fn clear(&mut self) {
        self.deque.clear();
    }
    pub fn get_target(&mut self) -> Option<Entity> {
        if self.deque.len() > 0 {
            return Some(self.deque[0].clone())
        }
        return None
    }
    pub fn move_to_next(&mut self) {
        if self.deque.len() > 0 {
            self.deque.pop_front();
        }
    }
    pub fn get_all(&self) -> VecDeque<Entity> {
        self.deque.clone()
    }
    pub fn len(&self) -> usize {
        self.deque.len()
    }
    pub fn add_target(&mut self, new_target: Entity) {
        for current_target in self.deque.iter() {
            if current_target.id() == new_target.id() {
                // Cannot add repeats to the target collection
                return
            }
        }
        self.deque.push_back(new_target);
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
    pub player: Player,  // The player of the unit TOO pass around references
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


#[derive(Component)]
pub struct Explosion;

#[derive(Component)]
pub struct ExplosionToSpawn(pub Vec3);

#[derive(Component)]
pub struct ExplosionTimer(pub Timer);

impl Default for ExplosionTimer {
	fn default() -> Self {
		Self(Timer::from_seconds(0.05, true))
	}
}
