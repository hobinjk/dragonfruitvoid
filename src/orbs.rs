use bevy::prelude::*;

use crate::game::GAME_TO_PX;

pub const ORB_TARGET_COLOR_BASE: Color = Color::rgb(0.5, 0.5, 0.5);
pub const ORB_TARGET_COLOR_ACTIVE: Color = Color::rgb(0.7, 1., 0.7);

pub const ORB_RADIUS: f32 = 190. * GAME_TO_PX;
pub const ORB_TARGET_RADIUS: f32 = 190. * GAME_TO_PX;
pub const ORB_VELOCITY_DECAY: f32 = 0.5;

#[derive(Component)]
pub struct OrbTarget(pub i32);
