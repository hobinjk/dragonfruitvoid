use bevy::prelude::*;

use crate::game::*;
use crate::collisions::collisions_players_waves_system;
use crate::aoes::*;
use crate::mobs::*;
use crate::greens::*;
use crate::waves::*;
use crate::ui::boss_healthbar_system;

pub const SPREAD_DAMAGE: f32 = 10.;
const SPREAD_DETONATION: f32 = 5.;
pub const SPREAD_RADIUS: f32 = 240. * GAME_TO_PX;

const PUDDLE_DAMAGE: f32 = 20.;
pub const PUDDLE_RADIUS: f32 = 450. * GAME_TO_PX;

#[derive(Component)]
pub struct SpreadAoeSpawn {
    pub timers: Vec<Timer>,
    pub aoe_desc: AoeDesc,
}

#[derive(Component)]
pub struct Puddle {
    pub visibility_start: Timer,
    pub drop: Timer,
}

fn spread_aoe_spawn_system(
    time: ResMut<Time>,
    players: Query<Entity, With<PlayerTag>>,
    mut commands: Commands,
    mut spread_aoe_spawns: Query<&mut SpreadAoeSpawn>,
    ) {
    for mut spread_aoe_spawn in &mut spread_aoe_spawns {
        let mut do_spawn = false;
        for timer in &mut spread_aoe_spawn.timers {
            timer.tick(time.delta());

            if timer.just_finished() {
                do_spawn = true;
            }
        }

        if do_spawn {
            spawn_aoe(&mut commands, &spread_aoe_spawn.aoe_desc, Vec3::new(0., 0., LAYER_WAVE), Aoe {
                visibility_start: None,
                detonation: Timer::from_seconds(SPREAD_DETONATION, false),
                damage: SPREAD_DAMAGE,
                linger: None,
            }, Some(AoeFollow { target: players.single() }));
        }
    }
}

pub fn boss_existence_check_system(
    bosses: Query<&Boss>,
    game: Res<Game>,
    mut state: ResMut<State<GameState>>,
    ) {
    if let Ok(_) = bosses.get_single() {
        return;
    }

    let cur_state = state.current().clone();
    if game.continuous && cur_state != GameState::SooWonTwo {
        state.set(next_game_state(cur_state)).unwrap();
    } else {
        state.push(GameState::Success).unwrap();
    }
}

fn puddles_system(time: Res<Time>,
    players: Query<&Transform, (With<PlayerTag>, Without<Puddle>)>,
    mut puddles: Query<(&mut Puddle, &mut Soup, &mut Transform, &mut Visibility, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    for (mut puddle, mut soup, mut transform, mut visibility, material) in &mut puddles {
        if puddle.drop.finished() {
            continue;
        }

        if !puddle.visibility_start.finished() {
            puddle.visibility_start.tick(time.delta());
        } else {
            puddle.drop.tick(time.delta());

            if puddle.drop.percent() < 4. / 6. {
                let transform_player = players.single();
                transform.translation = transform_player.translation;
            }
        }

        if puddle.visibility_start.just_finished() {
            visibility.is_visible = true;
        }

        if puddle.drop.just_finished() {
            soup.damage = PUDDLE_DAMAGE;
            materials.get_mut(material).unwrap().color.set_a(0.9);
        } else if puddle.drop.percent() > 4. / 6. {
            materials.get_mut(material).unwrap().color.set_a(0.7);
        }
    }
}

pub fn build_update_boss_phase(phase: GameState) -> SystemSet {
    SystemSet::on_update(phase)
        .with_system(collisions_players_waves_system)
        .with_system(greens_system)
        .with_system(greens_detonation_system)
        .with_system(spread_aoe_spawn_system)
        .with_system(aoes_system)
        .with_system(aoes_detonation_system)
        .with_system(aoes_follow_system)
        .with_system(waves_system)
        .with_system(boss_existence_check_system)
        .with_system(boss_healthbar_system)
        .with_system(puddles_system)
}

