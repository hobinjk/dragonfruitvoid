use bevy::prelude::*;

use crate::game::Game;
use crate::mobs::{
    Hp,
    Boss,
};

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

fn set_cooldown_text_display(timer: &Timer, text: &mut Text, text_display: &TextDisplay, sprites: &mut Query<&mut Sprite>) {
    let dur = timer.duration().as_secs_f32();
    let elapsed = timer.elapsed_secs();
    let left = dur - elapsed;

    if left < 5. {
        text.sections[0].value = format!("{left:.1}");
    } else {
        text.sections[0].value = format!("{left:.0}");
    }

    if left < 0.001 {
        text.sections[0].style.color.set_a(0.0);
    } else {
        text.sections[0].style.color.set_a(1.0);
    }

    if let Some(sprite_handle) = text_display.sprite {
        let color = if left < 0.001 {
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        } else {
            Color::rgba(0.7, 0.7, 0.7, 0.7)
        };
        sprites.get_mut(sprite_handle).unwrap().color = color;
    }
}

pub fn text_system(game: Res<Game>,
               mut text_displays: Query<(&mut Text, &TextDisplay)>,
               mut sprites: Query<&mut Sprite>) {
    for (mut text, text_display) in &mut text_displays {
        match text_display.value {
            TextValue::Hp => {
                let hp = game.player.hp;
                text.sections[0].value = format!("{hp:.0}");
            },
            TextValue::CooldownBlink => {
                set_cooldown_text_display(&game.player.blink_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownDodge => {
                set_cooldown_text_display(&game.player.dodge_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::StatusJump => {
                set_cooldown_text_display(&game.player.jump, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownPortal => {
                set_cooldown_text_display(&game.player.portal_cooldown, &mut text, &text_display, &mut sprites);
            },
            TextValue::CooldownPull => {
                set_cooldown_text_display(&game.player.pull_cooldown, &mut text, &text_display, &mut sprites);
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
        text.sections[0].value = format!("{left:.0}");
    }
}
