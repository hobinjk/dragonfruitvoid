use bevy::prelude::*;

use crate::{
    ai::AiRole,
    audio::{play_sfx, PhaseAudio, Sfx, SfxSource},
    game::{Game, GameState, MenuState, Player, HEIGHT, WIDTH},
};

#[derive(Component)]
pub struct MenuContainer;

#[derive(Component)]
pub enum ButtonNextState {
    GoTo(GameState),
    StartContinuous(),
    Resume(),
    Restart(),
    Exit(),
}

#[derive(Component)]
pub enum ButtonOnOff {
    Hints(),
    Echo(usize),
    UnlimitedRange(),
    Puddles(),
    Greens(),
    AI(),
    AIBars(),
    Role(),
    Audio(),
}

#[derive(Event)]
pub struct RestartEvent {
    game_state: GameState,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn setup_menu_system(
    mut commands: Commands,
    game: Res<Game>,
    asset_server: Res<AssetServer>,
    players: Query<Entity, With<Player>>,
    audio_sinks: Query<Entity, (With<AudioSink>, With<PhaseAudio>)>,
) {
    let button_width = Val::Px(350.0);
    let button_height = Val::Px(65.0);
    let button_margin = UiRect::all(Val::Px(10.));

    let button_node = Node {
        width: button_width,
        height: button_height,
        // center button
        margin: button_margin,
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 40.0,
        color: Color::srgb(0.9, 0.9, 0.9),
    };

    commands
        .spawn((Node {
            width: Val::Px(WIDTH),
            height: Val::Px(HEIGHT),
            flex_direction: FlexDirection::Row,
            // horizontally center children
            justify_content: JustifyContent::Center,
            // vertically center children
            align_items: AlignItems::Center,
            ..default()
        },))
        .with_children(|container| {
            container
                .spawn((Node {
                    width: Val::Px(WIDTH / 2.),
                    height: Val::Px(HEIGHT),
                    flex_direction: FlexDirection::Column,
                    // horizontally center children
                    justify_content: JustifyContent::Center,
                    // vertically center children
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|container| {
                    let phases = vec![
                        ("The Whole Fight", ButtonNextState::StartContinuous()),
                        (
                            "Purification One",
                            ButtonNextState::GoTo(GameState::PurificationOne),
                        ),
                        ("Jormag", ButtonNextState::GoTo(GameState::Jormag)),
                        ("Primordus", ButtonNextState::GoTo(GameState::Primordus)),
                        (
                            "Kralkatorrik",
                            ButtonNextState::GoTo(GameState::Kralkatorrik),
                        ),
                        (
                            "Purification Two",
                            ButtonNextState::GoTo(GameState::PurificationTwo),
                        ),
                        ("Mordremoth", ButtonNextState::GoTo(GameState::Mordremoth)),
                        ("Zhaitan", ButtonNextState::GoTo(GameState::Zhaitan)),
                        (
                            "Purification Three",
                            ButtonNextState::GoTo(GameState::PurificationThree),
                        ),
                        ("Soo-Won One", ButtonNextState::GoTo(GameState::SooWonOne)),
                        (
                            "Purification Four",
                            ButtonNextState::GoTo(GameState::PurificationFour),
                        ),
                        ("Soo-Won Two", ButtonNextState::GoTo(GameState::SooWonTwo)),
                    ];

                    for (label, state) in phases {
                        container
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                state,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(label, text_style.clone()));
                            });
                    }
                });

            container
                .spawn((Node {
                    width: Val::Px(WIDTH / 2.),
                    height: Val::Px(HEIGHT),
                    flex_direction: FlexDirection::Column,
                    // horizontally center children
                    justify_content: JustifyContent::Center,
                    // vertically center children
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|container| {
                    let echo_eggs = if game.echo_enabled { 0 } else { 17 };

                    let phases = vec![
                        ("Sound", ButtonOnOff::Audio(), game.audio_enabled),
                        ("Hints", ButtonOnOff::Hints(), game.hints_enabled),
                        ("Friends", ButtonOnOff::AI(), game.ai_enabled),
                        ("Friend Info", ButtonOnOff::AIBars(), game.ai_bars_enabled),
                        ("Require Greens", ButtonOnOff::Greens(), game.greens_enabled),
                        ("Spawn Reds", ButtonOnOff::Puddles(), game.puddles_enabled),
                        (
                            "Unlimited Range",
                            ButtonOnOff::UnlimitedRange(),
                            game.unlimited_range_enabled,
                        ),
                        (
                            "Ender's Echo",
                            ButtonOnOff::Echo(echo_eggs),
                            game.echo_enabled,
                        ),
                    ];

                    container
                        .spawn((
                            Button,
                            button_node.clone(),
                            BackgroundColor(NORMAL_BUTTON),
                            ButtonOnOff::Role(),
                        ))
                        .with_children(|parent| {
                            let value = player_role_to_string(&game.player_role);
                            parent.spawn(TextBundle::from_section(
                                format!("Role: {}", value),
                                text_style.clone(),
                            ));
                        });

                    for (label, state, onoff_enabled) in phases {
                        container
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                state,
                            ))
                            .with_children(|parent| {
                                let onoff = if onoff_enabled { "ON" } else { "OFF" };
                                parent.spawn(TextBundle::from_section(
                                    format!("{}: {}", label, onoff),
                                    text_style.clone(),
                                ));
                            });
                    }
                });
        })
        .insert(MenuContainer);

    for entity in &players {
        commands.entity(entity).despawn_recursive();
    }

    for entity in &audio_sinks {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn setup_pause_menu_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_width = Val::Px(350.0);
    let button_height = Val::Px(65.0);
    let button_margin = UiRect::all(Val::Px(10.));

    let button_node = Node {
        width: button_width,
        height: button_height,
        // center button
        margin: button_margin,
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 40.0,
        color: Color::srgb(0.9, 0.9, 0.9),
    };

    commands
        .spawn((
            Node {
                width: Val::Px(WIDTH),
                height: Val::Px(HEIGHT),
                flex_direction: FlexDirection::Column,
                // horizontally center children
                justify_content: JustifyContent::Center,
                // vertically center children
                align_items: AlignItems::Center,
                ..default()
            },
            MenuContainer,
        ))
        .with_children(|container| {
            let buttons = vec![
                ("Resume", ButtonNextState::Resume()),
                ("Exit", ButtonNextState::Exit()),
            ];

            for (label, state) in buttons {
                container
                    .spawn((
                        Button,
                        button_node.clone(),
                        BackgroundColor(NORMAL_BUTTON),
                        state,
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(label, text_style.clone()));
                    });
            }
        });
}

pub fn update_menu_system(
    game_state: Res<State<GameState>>,
    mut res_next_game_state: ResMut<NextState<GameState>>,
    mut res_next_menu_state: ResMut<NextState<MenuState>>,
    mut game: ResMut<Game>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonNextState),
        (Changed<Interaction>, With<Button>),
    >,
    mut restart_events: EventWriter<RestartEvent>,
) {
    for (interaction, mut color, next_state) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                play_sfx(
                    &mut commands,
                    &asset_server,
                    Sfx::MenuClick,
                    SfxSource::Player,
                );

                *color = PRESSED_BUTTON.into();
                match next_state {
                    ButtonNextState::GoTo(next_state) => {
                        game.continuous = false;
                        res_next_game_state.set(next_state.clone());
                        res_next_menu_state.set(MenuState::Unpaused);
                    }
                    ButtonNextState::StartContinuous() => {
                        game.continuous = true;
                        res_next_game_state.set(GameState::PurificationOne);
                        res_next_menu_state.set(MenuState::Unpaused);
                    }
                    ButtonNextState::Resume() => {
                        res_next_menu_state.set(MenuState::Unpaused);
                    }
                    ButtonNextState::Restart() => {
                        res_next_game_state.set(GameState::Nothing);
                        res_next_menu_state.set(MenuState::StartMenu);
                        restart_events.send(RestartEvent {
                            game_state: *game_state.get(),
                        });
                    }
                    ButtonNextState::Exit() => {
                        res_next_game_state.set(GameState::Nothing);
                        res_next_menu_state.set(MenuState::StartMenu);
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn restart_event_system(
    mut events: EventReader<RestartEvent>,
    mut res_next_game_state: ResMut<NextState<GameState>>,
    mut res_next_menu_state: ResMut<NextState<MenuState>>,
) {
    for event in events.read() {
        res_next_game_state.set(event.game_state);
        res_next_menu_state.set(MenuState::Unpaused);
    }
}

fn player_role_to_string(role: &Option<AiRole>) -> String {
    if let Some(role) = role {
        role.to_string()
    } else {
        "Spectator".to_string()
    }
}

pub fn update_menu_onoff_system(
    mut game: ResMut<Game>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut global_volume: ResMut<GlobalVolume>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &Children,
            &mut ButtonOnOff,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut texts: Query<&mut Text>,
) {
    for (interaction, mut color, children, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                play_sfx(
                    &mut commands,
                    &asset_server,
                    Sfx::MenuClick,
                    SfxSource::Player,
                );

                *color = PRESSED_BUTTON.into();

                match *button {
                    ButtonOnOff::Hints() => {
                        game.hints_enabled = !game.hints_enabled;
                        let onoff = if game.hints_enabled { "ON" } else { "OFF" };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Hints: {}", onoff);
                            }
                        }
                    }
                    ButtonOnOff::Greens() => {
                        game.greens_enabled = !game.greens_enabled;
                        let onoff = if game.greens_enabled { "ON" } else { "OFF" };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Require Greens: {}", onoff);
                            }
                        }
                    }
                    ButtonOnOff::Puddles() => {
                        game.puddles_enabled = !game.puddles_enabled;
                        let onoff = if game.puddles_enabled { "ON" } else { "OFF" };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Spawn Reds: {}", onoff);
                            }
                        }
                    }
                    ButtonOnOff::UnlimitedRange() => {
                        game.unlimited_range_enabled = !game.unlimited_range_enabled;
                        let onoff = if game.unlimited_range_enabled {
                            "ON"
                        } else {
                            "OFF"
                        };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Unlimited Range: {}", onoff);
                            }
                        }
                    }
                    ButtonOnOff::AI() => {
                        game.ai_enabled = !game.ai_enabled;
                        let onoff = if game.ai_enabled { "ON" } else { "OFF" };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Friends: {}", onoff);
                            }
                        }
                    }
                    ButtonOnOff::AIBars() => {
                        game.ai_bars_enabled = !game.ai_bars_enabled;
                        let onoff = if game.ai_bars_enabled { "ON" } else { "OFF" };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Friend Info: {}", onoff);
                            }
                        }
                    }

                    ButtonOnOff::Echo(ref mut val) => {
                        let mut label = "Ender's Echo";
                        if *val > 0 {
                            let bonus_labels = [
                                "Stop",
                                "No",
                                "Bad idea",
                                "Don't!",
                                "Mortals",
                                "You believe",
                                "Yourselves",
                                "Saviors",
                                "Naturally",
                                "You seek",
                                "To write",
                                "The conclusion",
                                "Of your legend",
                                "But there is",
                                "No conclusion",
                                "More natural than",
                                "DEATH",
                            ];
                            label = bonus_labels[17 - *val];
                            *val -= 1;
                        } else {
                            game.echo_enabled = !game.echo_enabled;
                        }

                        let onoff = if game.echo_enabled { "ON" } else { "OFF" };

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("{}: {}", label, onoff);
                            }
                        }
                    }

                    ButtonOnOff::Role() => {
                        let next_role = if let Some(role) = game.player_role {
                            match role {
                                AiRole::Virt1 | AiRole::Virt2 => Some(AiRole::Herald1),
                                AiRole::Herald1 | AiRole::Herald2 => Some(AiRole::Ham1),
                                AiRole::Ham1 | AiRole::Ham2 => Some(AiRole::Dps1),
                                AiRole::Dps1 | AiRole::Dps2 | AiRole::Dps3 | AiRole::Dps4 => None,
                            }
                        } else {
                            Some(AiRole::Virt1)
                        };
                        game.player_role = next_role;

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value =
                                    format!("Role: {}", player_role_to_string(&next_role));
                            }
                        }
                    }

                    ButtonOnOff::Audio() => {
                        game.audio_enabled = !game.audio_enabled;
                        let onoff = if game.audio_enabled { "ON" } else { "OFF" };

                        if game.audio_enabled {
                            *global_volume = GlobalVolume::new(1.0);
                        } else {
                            *global_volume = GlobalVolume::new(0.0);
                        }

                        for &child in children.iter() {
                            if let Ok(mut text) = texts.get_mut(child) {
                                text.sections[0].value = format!("Sound: {}", onoff);
                            }
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn cleanup_menu_system(mut commands: Commands, containers: Query<(Entity, &MenuContainer)>) {
    for (entity, _) in &containers {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn setup_success_system(
    game: Res<Game>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let success_message = if game.continuous {
        "You win!"
    } else {
        "Phase cleared!"
    };

    setup_result_screen(
        success_message,
        Color::srgb(0.3, 1.0, 0.3),
        game,
        &mut commands,
        asset_server,
    );
}

fn setup_result_screen(
    result_message: &str,
    result_color: Color,
    game: Res<Game>,
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
) {
    let button_width = Val::Px(350.0);
    let button_height = Val::Px(65.0);
    let button_margin = UiRect::all(Val::Px(10.));

    let button_node = Node {
        width: button_width,
        height: button_height,
        // center button
        margin: button_margin,
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 40.0,
        color: Color::srgb(0.9, 0.9, 0.9),
    };

    commands
        .spawn((
            Node {
                width: Val::Px(WIDTH),
                height: Val::Px(HEIGHT / 2.),
                margin: UiRect::all(Val::Auto), // UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(HEIGHT / 4.)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MenuContainer,
        ))
        .with_children(|big_container| {
            big_container
                .spawn((
                    Node {
                        width: Val::Px(WIDTH),
                        height: Val::Px(240.0),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0., 0., 0., 0.6)),
                ))
                .with_children(|parent| {
                    let text_style = TextStyle {
                        font: asset_server.load("trebuchet_ms.ttf"),
                        font_size: 80.,
                        color: result_color,
                    };

                    let text_style_small = TextStyle {
                        font: text_style.font.clone(),
                        font_size: 40.,
                        color: result_color,
                    };

                    let minutes = (game.time_elapsed.elapsed_secs() / 60.).floor() as i32;
                    let seconds = (game.time_elapsed.elapsed_secs() % 60.).floor() as i32;
                    let milliseconds =
                        ((game.time_elapsed.elapsed_secs() % 1.) * 1000.).floor() as i32;

                    let time_str = format!("{}:{:02}.{:03}", minutes, seconds, milliseconds);
                    parent.spawn(TextBundle::from_sections([
                        TextSection::new(result_message, text_style.clone()),
                        TextSection::new(
                            format!("\nTime: {}\n", time_str),
                            text_style_small.clone(),
                        ),
                        TextSection::new(
                            format!("Damage Taken: {}", game.player_damage_taken as i32),
                            text_style_small.clone(),
                        ),
                    ]));
                });

            big_container
                .spawn((Node {
                    width: Val::Px(WIDTH),
                    height: Val::Px(100.),
                    // horizontally center children
                    justify_content: JustifyContent::Center,
                    // vertically center children
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    let buttons = vec![
                        ("Restart", ButtonNextState::Restart()),
                        ("Exit", ButtonNextState::Exit()),
                    ];

                    for (label, state) in buttons {
                        parent
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                state,
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(label, text_style.clone()));
                            });
                    }
                });
        });
}

pub fn setup_failure_system(
    game: Res<Game>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    setup_result_screen(
        "You died :(",
        Color::srgb(0.9, 0.2, 0.2),
        game,
        &mut commands,
        asset_server,
    );
}

pub fn setup_show_hint_system(
    game: Res<Game>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let hint_text = game.hint.unwrap();

    let button_width = Val::Px(240.0);
    let button_height = Val::Px(65.0);
    let button_margin = UiRect::all(Val::Px(10.));

    let button_node = Node {
        width: button_width,
        height: button_height,
        // center button
        margin: button_margin,
        // horizontally center child text
        justify_content: JustifyContent::Center,
        // vertically center child text
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextStyle {
        font: asset_server.load("trebuchet_ms.ttf"),
        font_size: 28.0,
        color: Color::srgb(0.9, 0.9, 0.9),
    };

    commands
        .spawn((
            Node {
                width: Val::Px(WIDTH / 2.),
                height: Val::Px(HEIGHT / 2.),
                margin: UiRect::all(Val::Auto), // UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(0.), Val::Px(HEIGHT / 4.)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MenuContainer,
        ))
        .with_children(|big_container| {
            big_container
                .spawn((
                    Node {
                        width: Val::Px(WIDTH / 2. - 20.),
                        height: Val::Px(240.0),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0., 0., 0., 0.8)),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_sections([TextSection::new(
                        hint_text,
                        text_style.clone(),
                    )]));
                });

            big_container
                .spawn((Node {
                    width: Val::Px(WIDTH),
                    height: Val::Px(100.),
                    // horizontally center children
                    justify_content: JustifyContent::Center,
                    // vertically center children
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|parent| {
                    let buttons = vec![("Continue", ButtonNextState::Resume())];

                    for (label, state) in buttons {
                        parent
                            .spawn((
                                Button,
                                button_node.clone(),
                                BackgroundColor(NORMAL_BUTTON),
                                state,
                            ))
                            .with_children(|button| {
                                button.spawn(TextBundle::from_section(label, text_style.clone()));
                            });
                    }
                });
        });
}
