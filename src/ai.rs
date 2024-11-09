use bevy::prelude::*;
use std::collections::HashSet;
use std::ops::{Add, Sub};

use crate::boss_phase::{Puddle, PuddleSpawn};
use crate::game::Player;
use crate::greens::StackGreen;
use crate::mobs::Enemy;
use crate::{
    Bullet, HasHit, PhaseEntity, Velocity, BULLET_SIZE, BULLET_SPEED, GAME_TO_PX, LAYER_BULLET,
    MAP_RADIUS, PLAYER_RADIUS,
};

pub enum AiRole {
    Virt1,
    Virt2,
    Herald1,
    Herald2,
    Ham1,
    Ham2,
    Dps1,
    Dps2,
    Dps3,
    Dps4,
}

#[derive(Component)]
pub struct AiPlayer {
    pub role: AiRole,
}

enum Action {
    Move(Vec3),
    Shoot(Vec3),
    Rest,
}

struct Thought {
    utility: f32,
    action: Action,
}

fn think_dont_fall_off_edge(player_pos: &Vec3) -> Thought {
    let safe_map_radius = MAP_RADIUS - PLAYER_RADIUS * 1.1;
    if player_pos.length_squared() < safe_map_radius * safe_map_radius {
        return Thought {
            utility: 0.,
            action: Action::Rest,
        };
    }

    Thought {
        utility: 1.,
        action: Action::Move(Vec3::new(-player_pos.x, -player_pos.y, 0.)),
    }
}

fn think_shoot_enemy(
    player_pos: Vec3,
    enemies: &Query<(&Enemy, &Transform), Without<Player>>,
) -> Thought {
    let mut closest_enemy: Option<(f32, Vec3)> = None;
    for (_enemy, transform_enemy) in enemies {
        let enemy_pos: Vec3 = transform_enemy.translation;
        let dist_sq = enemy_pos.sub(player_pos).length_squared();
        closest_enemy = match closest_enemy {
            None => Some((dist_sq, enemy_pos)),
            Some((closest_dist_sq, closest_pos)) => {
                if closest_dist_sq < dist_sq {
                    Some((closest_dist_sq, closest_pos))
                } else {
                    Some((dist_sq, enemy_pos))
                }
            }
        }
    }

    match closest_enemy {
        None => Thought {
            utility: 0.,
            action: Action::Rest,
        },
        Some((_, closest_pos)) => Thought {
            utility: 0.1,
            action: Action::Shoot(closest_pos.sub(player_pos)),
        },
    }
}

pub fn player_ai_system(
    time: Res<Time>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut Player, &AiPlayer, &mut Transform)>,
    enemies: Query<(&Enemy, &Transform), Without<Player>>,
    greens: Query<(&StackGreen, &Children)>,
    puddle_spawns: Query<&PuddleSpawn>,
    puddles: Query<&Puddle>,
) {
    let speed = 250.0 * GAME_TO_PX * time.delta_seconds();
    for (entity_player, mut player, _ai, mut transform) in &mut players {
        let player_pos = transform.translation;
        for (green, green_children) in &greens {}
        for puddle_spawn in &puddle_spawns {}
        for puddle in &puddles {}

        let thoughts: Vec<Thought> = vec![
            think_dont_fall_off_edge(&player_pos),
            think_shoot_enemy(player_pos, &enemies),
        ];

        let best_thought = thoughts
            .iter()
            .reduce(|a, b| if a.utility > b.utility { a } else { b })
            .unwrap_or(&Thought {
                utility: 0.0,
                action: Action::Rest,
            });

        match best_thought.action {
            Action::Rest => {}
            Action::Move(dir) => {
                let mut movement = dir.clamp_length(0., speed);
                movement.z = 0.;
                transform.translation = transform.translation.add(movement);
            }
            Action::Shoot(dir) => {
                let mut vel = dir.clone();
                vel.z = 0.;
                vel = vel.clamp_length(BULLET_SPEED, BULLET_SPEED);

                commands
                    .spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.89, 0.39, 0.95),
                            custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                            ..default()
                        },
                        transform: Transform::from_xyz(player_pos.x, player_pos.y, LAYER_BULLET),
                        ..default()
                    })
                    .insert(Velocity(vel))
                    .insert(Bullet {
                        age: 0.,
                        firer: entity_player,
                    })
                    .insert(HasHit(HashSet::new()))
                    .insert(PhaseEntity);
                player.shoot_cooldown.reset();
            }
        }
    }
}

pub fn move_scorer_system() {}
