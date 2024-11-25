use crate::{
    audio::{play_sfx, Sfx, SfxSource},
    game::*,
};
use bevy::prelude::*;

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
pub const WAVE_GROWTH_DURATION: f32 = 3.2;
pub const WAVE_DAMAGE: f32 = 75.;

pub fn waves_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut waves: Query<(&mut Wave, &mut Visibility, &mut Transform)>,
) {
    for (mut wave, mut visibility, mut transform) in &mut waves {
        let mut visible = Visibility::Inherited;
        if !wave.visibility_start.finished() {
            wave.visibility_start.tick(time.delta());
            visible = Visibility::Hidden;

            if wave.visibility_start.just_finished() {
                play_sfx(
                    &mut commands,
                    &asset_server,
                    Sfx::Shockwave,
                    SfxSource::Enemy,
                );
            }
        } else {
            wave.growth.tick(time.delta());
        }

        if wave.growth.finished() {
            visible = Visibility::Hidden;
        }

        *visibility = visible;

        if visible == Visibility::Hidden {
            continue;
        }

        transform.scale = Vec3::splat(wave.growth.fraction());
    }
}
