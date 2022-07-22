#![allow(unused_variables)]
#![allow(unused_imports)]

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use konquer::{self, Map, GridLine};

// Temp Const

const WINDOW_W: i32 = 500;
const WINDOW_H: i32 = 500;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
		.insert_resource(WindowDescriptor {
			title: "rust-rts ".to_string() + env!("CARGO_PKG_VERSION"),
			width: WINDOW_W as f32,
			height: WINDOW_H as f32,
			..Default::default()
		})
		.add_plugins(DefaultPlugins)
		.add_plugin(konquer::UnitPlugin)
		.add_plugin(konquer::KinematicCameraPlugin)
        // .add_startup_system(startup_system)
		.add_startup_system(test_system)
		.run();
}

// fn startup_system(
// 	mut commands: Commands,
// 	asset_server: Res<AssetServer>,
// 	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
// 	// mut windows: ResMut<Windows>,
// ) {


// }

fn test_system(
	mut test_spawner: EventWriter<konquer::SpawnUnitEvent>,
) {
	let player1 = konquer::Player::new();
	let player2 = konquer::Player::new();

	test_spawner.send(konquer::SpawnUnitEvent::new(
		konquer::UnitType::DefaultUnit, player1.clone(), Vec3::new(-50., -50., 0.)
	));
	test_spawner.send(konquer::SpawnUnitEvent::new(
		konquer::UnitType::DefaultUnit, player1.clone(), Vec3::new(0., 0., 0.)
	));
	test_spawner.send(konquer::SpawnUnitEvent::new(
		konquer::UnitType::DefaultUnit, player1.clone(), Vec3::new(0., -100., 0.5)
	));
	test_spawner.send(konquer::SpawnUnitEvent::new(
		konquer::UnitType::DefaultUnit, player2.clone(), Vec3::new(100., 100., 0.3)
	));
}