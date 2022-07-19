use bevy::prelude::Component;

#[derive(Component)]
pub struct Position {
	pub x: f32,
	pub y: f32,
    pub w: f32,  // 
}

#[derive(Component)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
    pub w: f32,  // Angular velocity
}

#[derive(Component)]
pub struct Hp {
    pub max: u8,
    pub current: u8,
}

#[derive(Component)]
pub struct Id {
    pub name: String,
}

#[derive(Component)]
pub struct Shield {
    pub max: u8,
    pub current: u8,
}

#[derive(Component)]
pub struct Unit {
    pub owner: u8,
}

