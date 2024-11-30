use bevy::prelude::*;

use std::collections::HashSet;

use crate::{
    ai::AiPlayer,
    audio::{play_sfx, Sfx, SfxSource},
    Player,
};

#[derive(Event)]
pub struct DamageFlashEvent {
    pub entity: Entity,
}

#[derive(Component)]
pub struct TintUntint {
    color: Color,
    tint_color: Color,
    untint_timer: Timer,
    tint_timer: Timer,
}

pub fn damage_flash_system(
    asset_server: Res<AssetServer>,
    mut events: EventReader<DamageFlashEvent>,
    mut commands: Commands,
    entities: Query<(Entity, Option<&Player>, Option<&AiPlayer>)>,
    mut sprites: Query<&mut Sprite, Without<TintUntint>>,
) {
    let mut touched = HashSet::new();

    for event in events.read() {
        if touched.contains(&event.entity) {
            continue;
        }
        if let Ok(sprite) = sprites.get_mut(event.entity) {
            let prev_color = sprite.color.clone();
            touched.insert(event.entity);
            commands.entity(event.entity).try_insert(TintUntint {
                color: prev_color,
                tint_color: Color::srgba(1.0, 0., 0., 0.7),
                tint_timer: Timer::from_seconds(0.2, TimerMode::Once),
                untint_timer: Timer::from_seconds(0.5, TimerMode::Once),
            });

            if let Ok((_, player, ai_player)) = entities.get(event.entity) {
                if ai_player.is_some() {
                    play_sfx(&mut commands, &asset_server, Sfx::Hurt, SfxSource::AiPlayer);
                } else if player.is_some() {
                    play_sfx(&mut commands, &asset_server, Sfx::Hurt, SfxSource::Player);
                }
            }
        }
    }
}

pub fn tint_untint_system(
    time: Res<Time>,
    mut commands: Commands,
    mut sprites: Query<(Entity, &mut TintUntint, &mut Sprite)>,
) {
    for (entity, mut tut, mut sprite) in &mut sprites {
        tut.tint_timer.tick(time.delta());
        tut.untint_timer.tick(time.delta());
        if !tut.tint_timer.finished() {
            sprite.color = tut.tint_color;
        } else {
            sprite.color = tut.color;
        }
        if tut.untint_timer.finished() {
            sprite.color = tut.color;
            commands.entity(entity).remove::<TintUntint>();
        }
    }
}
