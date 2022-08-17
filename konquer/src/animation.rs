use bevy::{prelude::*, ecs::system::EntityCommands};
use bevy_prototype_lyon::prelude::*;

use crate::*;

pub fn explosion_to_spawn_system(
	mut commands: Commands,
	query: Query<(Entity, &ExplosionToSpawn)>,
    texture_server: Res<TextureServer>
) {
	for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
		// spawn the explosion sprite
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_server.get(&String::from("data/fx/explo_a_sheet.png")).typed::<TextureAtlas>(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        // despawn the explosionToSpawn
        commands.entity(explosion_spawn_entity).despawn();
    }
}

pub fn explosion_animation_system(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
	for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
		if timer.0.finished() {
			sprite.index += 1; // move to next sprite cell
			if sprite.index >= 16 {  // TODO manage fx sheets
				commands.entity(entity).despawn()
			}
		}
	}
}