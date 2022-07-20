#![allow(unused)]

use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use konquer;

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
			width: 500.0,
			height: 500.0,
			..Default::default()
		})
		.add_plugins(DefaultPlugins)
		.add_plugin(DebugLinesPlugin::default())
		.add_plugin(konquer::UnitPlugin)
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
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());
	let window = windows.get_primary_mut().unwrap();
	let win_size = WinSize { w: window.width(), h: window.height() };
	commands.insert_resource(win_size);
	// Draw grid
	// for x in (0..win_size.w as i8).step_by(10) {
	// 	lines.line_colored(
	// 		Vec3::new(-400.0, 0.0, 0.5),
	// 		Vec3::new(400.0, 0.0, 0.5),
	// 		0.9,
	// 		Color::GREEN,
	// 	);
	// }
	println!("Window is {}x{}", window.width(), window.height());
	for y in (-250..250).step_by(10) {
		println!("Drawing HLine at y={}", y);
		lines.line_colored(
			Vec3::new(-window.width() / 2., y as f32, 0.5),
			Vec3::new(window.width() / 2., y as f32, 0.5),
			9999999.,
			Color::GREEN,
		);
	}
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