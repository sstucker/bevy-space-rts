#![allow(unused)]

use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use konquer::{self, Map};

// Temp Const

const WINDOW_W: i32 = 500;
const WINDOW_H: i32 = 500;

const MAP_W: i32 = 1200;
const MAP_H: i32 = 1200;


#[derive(Debug)]
pub struct WinSize {
	pub w: f32,
	pub h: f32,
}

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
		.add_plugin(DebugLinesPlugin::default())
		.add_plugin(konquer::UnitPlugin)
		.add_plugin(konquer::KinematicCameraPlugin)
        .add_startup_system(startup_system)
		.add_startup_system(test_system)
		.run();
}

fn startup_system(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut texture_atlases: ResMut<Assets<TextureAtlas>>,
	mut windows: ResMut<Windows>,
	mut lines: ResMut<DebugLines>
) {
	let window = windows.get_primary_mut().unwrap();
	let win_size = WinSize { w: window.width(), h: window.height() };
	commands.insert_resource(win_size);

	let map: konquer::Map = konquer::Map { w: MAP_W, h: MAP_H };

	// Draw map grid
	for y in (-map.h / 2..map.h / 2).step_by(10) {
		lines.line_colored(
			Vec3::new(-MAP_W as f32 / 2., y as f32, 0.5),
			Vec3::new(MAP_W as f32 / 2., y as f32, 0.5),
			9999999.,
			Color::Rgba { red:0.1, green: 0.1, blue: 0.1, alpha: 1. },
		);
	}
	for x in (-map.w / 2..map.w / 2).step_by(10) {
		lines.line_colored(
			Vec3::new(x as f32, -MAP_H as f32 / 2., 0.5),
			Vec3::new(x as f32, MAP_H as f32 / 2., 0.5),
			9999999.,
			Color::Rgba { red:0.1, green: 0.1, blue: 0.1, alpha: 1. },
		);
	}

	commands.insert_resource(map);

}

fn test_system(
	mut test_spawner: EventWriter<konquer::SpawnUnitEvent>,
) {
	let owner1 = konquer::Owner::new();
	let owner2 = konquer::Owner::new();

	test_spawner.send(konquer::SpawnUnitEvent::new(
		konquer::UnitType::DefaultUnit, owner1, Vec3::new(-20., 0., 0.)
	));
	test_spawner.send(konquer::SpawnUnitEvent::new(
		konquer::UnitType::DefaultUnit, owner2, Vec3::new(20., 0., 0.5)
	));
}