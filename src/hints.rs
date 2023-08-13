use bevy::{
    prelude::*,
};

use crate::game::*;

#[derive(Component)]
pub struct ScheduledHint {
    start: Timer,
    hint: &'static str,
}

pub struct HintTiming {
    pub start: f32,
    pub hint: &'static str,
}

const HINTS_ALL_PHASES: [HintTiming; 1] = [
    HintTiming {
        start: 0.,
        hint: "Welcome to The Dragonfruitvoid!

Move with WASD and shoot with left mouse
button. Your skills are 4: Pull, E: Blink,
V: Dodge, Space: Jump, and
R: Portal (advanced).",
    },
];

const HINT_INDIVIDUAL_PHASE: HintTiming = HintTiming {
    start: 0.1,
    hint: "Previous phases may have hints
related to this phase",
};

fn phase_hints(phase: &GameState) -> Vec<HintTiming> {
    match phase {
        GameState::PurificationOne => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "Push the white orb through the green
targets with your bullets! Prevent the
crabs from reaching the orbs by killing
them (also with your bullets). The black
circles are THE VOID and will damage you.",
                },
            ]
        },
        GameState::Jormag => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "The big red circle is a boss. Pew pew
it to victory!",
                },
                HintTiming {
                    start: 5.5,
                    hint: "Red puddles will follow you for a short
time before they drop on the ground and
become a damaging zone. Move quickly to
drop them out of the way. Dodge out to
avoid taking too much damage.",
                },
                HintTiming {
                    start: 15.5,
                    hint: "Green circles need to be soaked. Pick one
of the three to stand in before it
explodes.",
                },
            ]
        },
        GameState::Primordus => {
            vec![
                HintTiming {
                    start: 13.5,
                    hint: "Orange circles are AoEs. Run away!",
                },
            ]
        },
        GameState::Kralkatorrik => {
            vec![
                HintTiming {
                    start: 5.5,
                    hint: "Space is limited in this phase. Watch
out for the lines of VOID.",
                },
            ]
        },
        GameState::PurificationTwo => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "Push the orb to the targets again. Watch
out, the timecaster is a nasty enemy who
needs to be removed or they'll push away
the orb."
                },
                HintTiming {
                    start: 4.,
                    hint: "The orb shoots bees out now, you
know how it is with bees.",
                },
            ]
        },
        GameState::Mordremoth => {
            vec![
                HintTiming {
                    start: 21.,
                    hint: "Shockwaves incoming! These blue waves
need to be dodged, jumped over, or blinked
through to prevent massive damage.",
                },
            ]
        },
        GameState::Zhaitan => {
            vec![
                HintTiming {
                    start: 5.5,
                    hint: "Noodles attack the area near them.
Kill them before they kill you!",
                },
            ]
        },
        GameState::PurificationThree => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "Push the orb to the targets. There is
now a big saltspray dragon. Do what you do
to dragons, you heartless monster.",
                },
            ]
        },
        GameState::SooWonOne => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "Get ready for everything from every
phase all at once!",
                },
            ]
        },
        GameState::PurificationFour => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "Time to get revenge on the orb by
killing it! Beware, your bullets still push.",
                },
            ]
        },
        GameState::SooWonTwo => {
            vec![
                HintTiming {
                    start: 1.,
                    hint: "Everything from every phase all at
once part two: Electric Boogaloo. Don't
let the Obliterator or Goliath hit you!",
                },
            ]
        },
        GameState::Nothing => {
            vec![]
        },
    }
}

pub fn setup_hints(
    commands: &mut Commands,
    game: &ResMut<Game>,
    state: Res<State<GameState>>,
    ) {
    if !game.hints_enabled {
        return;
    }

    let mut hints: Vec<HintTiming> = vec![];

    // Reset all cooldowns and invuln timings
    if !game.continuous {
        hints.extend(HINTS_ALL_PHASES);
        match state.get() {
            GameState::PurificationOne |
            GameState::Jormag
                => {
            }
            _ => {
                hints.push(HINT_INDIVIDUAL_PHASE);
            }
        }
    } else {
        if *state.get() == GameState::PurificationOne {
            hints.extend(HINTS_ALL_PHASES);
        }
    }

    hints.extend(phase_hints(&state.get()));

    for hint in &hints {
        commands.spawn(ScheduledHint {
            start: Timer::from_seconds(hint.start, TimerMode::Once),
            hint: hint.hint,
        });
    }
}

pub fn scheduled_hint_system(
    time: Res<Time>,
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut hints: Query<(Entity, &mut ScheduledHint)>,
    ) {
    for (entity, mut hint) in &mut hints {
        hint.start.tick(time.delta());
        if !hint.start.finished() {
            continue;
        }
        game.hint = Some(hint.hint);
        next_menu_state.set(MenuState::PausedShowHint);
        commands.entity(entity).despawn_recursive();
        break;
    }
}
