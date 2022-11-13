use bevy::prelude::*;

use std::collections::HashSet;

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
    mut events: EventReader<DamageFlashEvent>,
    mut commands: Commands,
    mut sprites: Query<&mut Sprite, Without<TintUntint>>,
    ) {
    let mut touched = HashSet::new();

    for event in events.iter() {
        if touched.contains(&event.entity) {
            continue;
        }
        if let Ok(sprite) = sprites.get_mut(event.entity) {
            let prev_color = sprite.color.clone();
            touched.insert(event.entity);
            commands.entity(event.entity).insert(TintUntint {
                color: prev_color,
                tint_color: Color::rgba(1.0, 0., 0., 0.7),
                tint_timer: Timer::from_seconds(0.2, false),
                untint_timer: Timer::from_seconds(0.5, false),
            });
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
