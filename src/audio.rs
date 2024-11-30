use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

#[derive(Component)]
pub struct AudioPhaseTheme;

#[derive(PartialEq)]
pub enum Sfx {
    Blink,
    EnemyShoot,
    EnemyShootBig,
    GreenPop,
    Hurt,
    Jump,
    MenuClick,
    OrbHitEdge,
    OrbHitTarget,
    PortalEnter,
    PortalExit,
    Pull,
    RedTarget,
    Roar,
    Shockwave,
    Shoot,
}

#[derive(PartialEq)]
pub enum SfxSource {
    Player,
    Enemy,
    AiPlayer,
}

#[derive(Component)]
pub struct PhaseAudio;

pub fn setup_audio(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        AudioPlayer(asset_server.load::<AudioSource>("sounds/phase_theme.ogg")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.5),
            ..default()
        },
        AudioPhaseTheme,
        PhaseAudio,
    ));
}

pub fn play_sfx(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    sfx: Sfx,
    source: SfxSource,
) {
    let path = match sfx {
        Sfx::Blink => "sounds/blink.ogg",
        Sfx::EnemyShoot => "sounds/enemy_shoot.ogg",
        Sfx::EnemyShootBig => "sounds/enemy_shoot_big.ogg",
        Sfx::GreenPop => "sounds/green_pop.ogg",
        Sfx::Hurt => "sounds/hurt.ogg",
        Sfx::Jump => "sounds/jump.ogg",
        Sfx::OrbHitEdge => "sounds/orb_hit_edge.ogg",
        Sfx::OrbHitTarget => "sounds/orb_hit_target.ogg",
        Sfx::PortalEnter => "sounds/portal_enter.ogg",
        Sfx::PortalExit => "sounds/portal_exit.ogg",
        Sfx::Pull => "sounds/pull.ogg",
        Sfx::RedTarget => "sounds/red_target.ogg",
        Sfx::Roar => "sounds/roar.ogg",
        Sfx::Shockwave => "sounds/shockwave.ogg",
        Sfx::Shoot => "sounds/shoot.ogg",
        Sfx::MenuClick => "sounds/menu_click.ogg",
    };

    let mut volume = match source {
        SfxSource::Player => 1.0,
        SfxSource::Enemy => 0.7,
        SfxSource::AiPlayer => 0.2,
    };

    match sfx {
        Sfx::Shoot | Sfx::EnemyShoot => {
            volume *= 0.2;
        }
        Sfx::MenuClick => {
            volume = 0.2;
        }
        Sfx::Roar => {
            volume *= 0.5;
        }
        _ => {}
    };

    if sfx == Sfx::Shoot && source == SfxSource::AiPlayer {
        return;
    }

    commands.spawn((
        AudioPlayer(asset_server.load::<AudioSource>(path)),
        PlaybackSettings::REMOVE.with_volume(Volume::new(volume)),
        PhaseAudio,
    ));
}
