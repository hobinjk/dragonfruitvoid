use bevy::prelude::*;

use crate::ai::AiPlayer;
use crate::game::Player;
use crate::mobs::{Boss, Hp};

pub enum TextValue {
    Hp,
    CooldownDodge,
    CooldownBlink,
    CooldownPortal,
    CooldownPull,
    StatusJump,
}

#[derive(Component)]
pub struct TextDisplay {
    pub value: TextValue,
    pub sprite: Option<Entity>,
}

#[derive(Component)]
pub struct BossHealthbar;

#[derive(Component)]
pub struct BossHealthbarText;

#[derive(Component)]
pub struct PlayerHealthbar {
    pub player: Entity,
}

pub enum PlayerCooldown {
    Blink,
    Jump,
    Dodge,
}

impl PlayerCooldown {
    pub fn color(&self) -> Color {
        match self {
            PlayerCooldown::Blink => Color::srgb(0.89, 0.39, 0.95),
            PlayerCooldown::Jump => Color::srgb(0.3, 0.9, 0.9),
            PlayerCooldown::Dodge => Color::srgb(0.9, 0.9, 0.3),
        }
    }
}

#[derive(Component)]
pub struct PlayerCooldownBar {
    pub player: Entity,
    pub cooldown: PlayerCooldown,
}

#[derive(Component)]
pub struct Gauge {
    pub value: f32,
    pub hide_when_full: bool,
}

#[derive(Component)]
pub struct GaugeBar;

fn set_cooldown_text_display(
    timer: &Timer,
    text: &mut Text,
    text_color: &mut TextColor,
    text_display: &TextDisplay,
    sprites: &mut Query<&mut Sprite>,
) {
    let dur = timer.duration().as_secs_f32();
    let elapsed = timer.elapsed_secs();
    let left = dur - elapsed;

    if left < 5. {
        text.0 = format!("{left:.1}");
    } else {
        text.0 = format!("{left:.0}");
    }

    if left < 0.001 {
        text_color.set_alpha(0.0);
    } else {
        text_color.set_alpha(1.0);
    }

    if let Some(sprite_handle) = text_display.sprite {
        let color = if left < 0.001 {
            Color::srgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::srgba(0.7, 0.7, 0.7, 0.7)
        };
        sprites.get_mut(sprite_handle).unwrap().color = color;
    }
}

pub fn player_text_system(
    players: Query<&Player, Without<AiPlayer>>,
    mut text_displays: Query<(&mut Text, &mut TextColor, &TextDisplay)>,
    mut sprites: Query<&mut Sprite>,
) {
    for player in &players {
        for (mut text, mut text_color, text_display) in &mut text_displays {
            match text_display.value {
                TextValue::Hp => {
                    let hp = player.get_hp().clamp(0., 100.);
                    text.0 = format!("{hp:.0}");
                }
                TextValue::CooldownBlink => {
                    set_cooldown_text_display(
                        &player.blink_cooldown,
                        &mut text,
                        &mut text_color,
                        &text_display,
                        &mut sprites,
                    );
                }
                TextValue::CooldownDodge => {
                    set_cooldown_text_display(
                        &player.dodge_cooldown,
                        &mut text,
                        &mut text_color,
                        &text_display,
                        &mut sprites,
                    );
                }
                TextValue::StatusJump => {
                    set_cooldown_text_display(
                        &player.jump,
                        &mut text,
                        &mut text_color,
                        &text_display,
                        &mut sprites,
                    );
                }
                TextValue::CooldownPortal => {
                    set_cooldown_text_display(
                        &player.portal_cooldown,
                        &mut text,
                        &mut text_color,
                        &text_display,
                        &mut sprites,
                    );
                }
                TextValue::CooldownPull => {
                    set_cooldown_text_display(
                        &player.pull_cooldown,
                        &mut text,
                        &mut text_color,
                        &text_display,
                        &mut sprites,
                    );
                }
            }
        }
    }
}

pub fn boss_healthbar_system(
    bosses: Query<(&Boss, &Hp)>,
    mut boss_healthbars: Query<&mut Transform, With<BossHealthbar>>,
    mut texts: Query<&mut Text, With<BossHealthbarText>>,
) {
    if let Err(_) = bosses.get_single() {
        return;
    }

    let (boss, boss_hp) = bosses.single();
    let remaining = boss_hp.0 / boss.max_hp;
    for mut transform in &mut boss_healthbars {
        transform.scale.x = remaining;
    }
    for mut text in &mut texts {
        let left = remaining * 100.;
        text.0 = format!("{left:.0}");
    }
}

pub fn player_healthbar_update_gauge_system(
    players: Query<&Player>,
    mut gauges: Query<(&mut Gauge, &PlayerHealthbar), Without<Player>>,
) {
    for (mut gauge, player_healthbar) in &mut gauges {
        if let Ok(player) = players.get(player_healthbar.player) {
            let remaining = player.get_hp() / 100.;
            gauge.value = remaining.clamp(0., 1.);
        }
    }
}

pub fn player_cooldown_update_gauge_system(
    players: Query<&Player>,
    mut gauges: Query<(&mut Gauge, &PlayerCooldownBar)>,
) {
    for (mut gauge, player_cooldown_bar) in &mut gauges {
        if let Ok(player) = players.get(player_cooldown_bar.player) {
            let remaining = match player_cooldown_bar.cooldown {
                PlayerCooldown::Jump => player.jump_cooldown.fraction(),
                PlayerCooldown::Blink => player.blink_cooldown.fraction(),
                PlayerCooldown::Dodge => player.dodge_cooldown.fraction(),
            };
            gauge.value = remaining.clamp(0., 1.);
        }
    }
}

pub fn update_gauge_visibility_system(mut gauges: Query<(&Gauge, &mut Visibility)>) {
    for (gauge, mut visibility) in &mut gauges {
        if gauge.hide_when_full && gauge.value > 0.99 {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Inherited;
        }
    }
}

pub fn update_gauge_bars_system(
    gauges: Query<(&Gauge, &Children)>,
    mut gauge_bars: Query<&mut Transform, With<GaugeBar>>,
) {
    for (gauge, children) in &gauges {
        for &child in children.iter() {
            if let Ok(mut bar_transform) = gauge_bars.get_mut(child) {
                bar_transform.scale.x = gauge.value;
            }
        }
    }
}
