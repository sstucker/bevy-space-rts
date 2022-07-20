#![allow(unused)]

use bevy::prelude::*;
use konquer;

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
) {
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());
	let window = windows.get_primary_mut().unwrap();
	let win_size = WinSize { w: window.width(), h: window.height() };
	commands.insert_resource(win_size);
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