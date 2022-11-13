use bevy::prelude::*;
use crate::game::*;

#[derive(Component)]
pub struct Wave {
    pub visibility_start: Timer,
    pub growth: Timer,
}

impl Default for Wave {
    fn default() -> Wave {
        Wave {
            visibility_start: Timer::from_seconds(0., TimerMode::Once),
            growth: Timer::from_seconds(WAVE_GROWTH_DURATION, TimerMode::Once),
        }
    }
}

pub const WAVE_MAX_RADIUS: f32 = WIDTH / 2.;
pub const WAVE_VELOCITY: f32 = GAME_RADIUS / 3.2 * GAME_TO_PX;
pub const WAVE_GROWTH_DURATION: f32 = WAVE_MAX_RADIUS / WAVE_VELOCITY;
pub const WAVE_DAMAGE: f32 = 75.;

pub fn waves_system(
    time: Res<Time>,
    mut waves: Query<(&mut Wave, &mut Visibility, &mut Transform)>,
    ) {
    for (mut wave, mut visibility, mut transform) in &mut waves {
        let mut visible = true;
        if !wave.visibility_start.finished() {
            wave.visibility_start.tick(time.delta());
            visible = false;
        } else {
            wave.growth.tick(time.delta());
        }

        if wave.growth.finished() {
            visible = false;
        }

        visibility.is_visible = visible;

        if !visible {
            continue;
        }

        transform.scale = Vec3::splat(wave.growth.percent());
    }
}

