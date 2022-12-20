use bevy::{
    prelude::*,
    sprite::{Mesh2dHandle, MaterialMesh2dBundle},
};

use crate::game::*;
use crate::collisions::{collisions_players_waves_system, CollisionRadius};
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
pub struct PuddleSpawn {
    pub visibility_start: Timer,
    pub mesh: Mesh2dHandle,
    pub material: ColorMaterial,
}

#[derive(Component)]
pub struct Puddle {
    pub drop: Timer,
    pub target: Entity,
}

fn spread_aoe_spawn_system(
    time: ResMut<Time>,
    players: Query<Entity, With<Player>>,
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
            for player in &players {
                spawn_aoe(&mut commands, &spread_aoe_spawn.aoe_desc, Vec3::new(0., 0., LAYER_WAVE), Aoe {
                    visibility_start: None,
                    detonation: Timer::from_seconds(SPREAD_DETONATION, TimerMode::Once),
                    damage: SPREAD_DAMAGE,
                    linger: None,
                }, Some(AoeFollow { target: player }));
            }
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


fn get_puddle_target_sorted_players(
    players: &Query<(Entity, &Transform), With<Player>>,
    ) -> Vec<Entity> {

    let mut players_by_dist: Vec<(Entity, &Transform)> = vec![];
    for player in players {
        players_by_dist.push(player)
    }

    players_by_dist.sort_by(|a, b| {
        let pos_a = a.1.translation;
        let pos_b = b.1.translation;
        pos_a.length_squared().total_cmp(&pos_b.length_squared())
    });
    players_by_dist.iter().map(|a| a.0).collect()
}

fn puddle_spawns_system(
    time: Res<Time>,
    players: Query<(Entity, &Transform), With<Player>>,
    mut puddle_spawns: Query<(Entity, &mut PuddleSpawn)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    for (entity, mut puddle_spawn) in &mut puddle_spawns {
        puddle_spawn.visibility_start.tick(time.delta());
        if !puddle_spawn.visibility_start.finished() {
            continue;
        }

        commands.entity(entity).despawn_recursive();

        for &entity_player in get_puddle_target_sorted_players(&players).iter().take(2) {
            commands.spawn(MaterialMesh2dBundle {
                mesh: puddle_spawn.mesh.clone(),
                material: materials.add(puddle_spawn.material.clone()),
                transform: Transform::from_xyz(0., 0., 0.,),
                ..default()
            }).insert(Puddle {
                drop: Timer::from_seconds(6., TimerMode::Once),
                target: entity_player,
            })
            .insert(CollisionRadius(PUDDLE_RADIUS))
            .insert(Soup {
                damage: 0.,
                duration: None,
            });
        }
    }
}

fn puddles_system(time: Res<Time>,
    players: Query<&Transform, (With<Player>, Without<Puddle>)>,
    mut puddles: Query<(&mut Puddle, &mut Soup, &mut Transform, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ) {
    for (mut puddle, mut soup, mut transform, material) in &mut puddles {
        if puddle.drop.finished() {
            continue;
        }

        puddle.drop.tick(time.delta());
        if puddle.drop.percent() < 4. / 6. {
            if let Ok(transform_player) = players.get(puddle.target) {
                transform.translation = transform_player.translation;
            }
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
        .with_system(puddle_spawns_system)
        .with_system(puddles_system)
}

