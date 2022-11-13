use bevy::{
    prelude::*,
    time::Stopwatch,
};

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    StartMenu,
    PurificationOne,
    Jormag,
    Primordus, // -> big aoe and void zone
    Kralkatorrik, // -> line aoes
    PurificationTwo, // -> kill big boy without cleaving
    Mordremoth,
    Zhaitan, // -> noodles and grid aoe
    PurificationThree, // -> kill bigger boy without cleaving
    SooWonOne, // -> soowontwo minus big boys
    PurificationFour, // -> damage orb
    SooWonTwo,
    Failure,
    Success,
    Paused,
    PausedShowHint,
}

pub const LAYER_MAX: f32 = 110.;

pub const LAYER_PLAYER: f32 = 100.;
pub const LAYER_CURSOR: f32 = LAYER_PLAYER - 5.;
pub const LAYER_MOB: f32 = 20.;
pub const LAYER_WAVE: f32 = 15.;
pub const LAYER_AOE: f32 = 12.;
pub const LAYER_TARGET: f32 = 10.;
pub const LAYER_ROTATING_SOUP: f32 = 11.;
pub const LAYER_MAP: f32 = 0.;
pub const LAYER_VOID: f32 = 0.5;
pub const LAYER_UI: f32 = 1.;
pub const LAYER_TEXT: f32 = 2.;

pub const WIDTH: f32 = 1024.;
pub const HEIGHT: f32 = 1024.;
pub const GAME_WIDTH: f32 = 2849.;
pub const GAME_RADIUS: f32 = GAME_WIDTH / 2.;
pub const PX_TO_GAME: f32 = GAME_WIDTH / WIDTH;
pub const GAME_TO_PX: f32 = 1. / PX_TO_GAME;

pub const MAP_RADIUS: f32 = WIDTH / 2.;

pub const BULLET_COOLDOWN: f32 = 0.2;
pub const BULLET_SIZE: f32 = 10.;
pub const BULLET_DAMAGE: f32 = 0.3 / 1.2;
pub const BULLET_SPEED: f32 = 200.0;

pub const PLAYER_RADIUS: f32 = 20.;
pub const PLAYER_REGEN: f32 = 1.;

pub const CRAB_SIZE: f32 = 40.;

#[derive(Component)]
pub struct CursorMark;

#[derive(Component)]
pub struct PlayerTag;

pub struct Player {
    pub hp: f32,
    pub damage_taken: f32,
    pub shoot_cooldown: Timer,
    pub dodge_cooldown: Timer,
    pub blink_cooldown: Timer,
    pub portal_cooldown: Timer,
    pub pull_cooldown: Timer,
    pub jump_cooldown: Timer,
    pub invuln: Timer,
    pub jump: Timer,
    pub entity: Option<Entity>,
}

#[derive(Component)]
pub struct Bullet(pub f32);

#[derive(Component)]
pub struct EnemyBullet {
    pub damage: f32,
    pub knockback: f32,
}



impl Default for Player {
    fn default() -> Self {
        Player {
            hp: 100.,
            damage_taken: 0.,
            shoot_cooldown: Timer::from_seconds(BULLET_COOLDOWN, false),
            dodge_cooldown: Timer::from_seconds(10., false),
            blink_cooldown: Timer::from_seconds(16., false),
            portal_cooldown: Timer::from_seconds(60., false),
            jump_cooldown: Timer::from_seconds(0.6, false),
            pull_cooldown: Timer::from_seconds(20., false),
            invuln: Timer::from_seconds(0.75, false),
            jump: Timer::from_seconds(0.75, false),
            entity: None,
        }
    }
}

pub struct Game {
    pub player: Player,
    pub time_elapsed: Stopwatch,
    pub orb_target: i32,
    pub continuous: bool,
    pub echo_enabled: bool,
    pub hints_enabled: bool,
    pub hint: Option<&'static str>,
}

pub fn next_game_state(game_state: GameState) -> GameState {
    match game_state {
        GameState::PurificationOne => GameState::Jormag,
        GameState::Jormag => GameState::Primordus,
        GameState::Primordus => GameState::Kralkatorrik,
        GameState::Kralkatorrik => GameState::PurificationTwo,
        GameState::PurificationTwo => GameState::Mordremoth,
        GameState::Mordremoth => GameState::Zhaitan,
        GameState::Zhaitan => GameState::PurificationThree,
        GameState::PurificationThree => GameState::SooWonOne,
        GameState::SooWonOne => GameState::PurificationFour,
        GameState::PurificationFour => GameState::SooWonTwo,

        other => {
            warn!("next_game_state called for one not in the flow {:?}", other);
            other
        }
    }
}
