use bevy::prelude::*;
use crate::{despawn_screen, GameState, DisplayQuality, Rules};

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// State used for the current menu screen
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    SettingsDisplay,
    SettingsRules,
    #[default]
    Disabled,
}

// Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnMainMenuScreen;

// Tag component used to tag entities added on the display settings menu screen
#[derive(Component)]
struct OnDisplaySettings;

// Tag component used to tag entities added on the rules settings menu screen
#[derive(Component)]
struct OnRulesSettings;

// Tag component used to mark which setting is currently selected
#[derive(Component)]
struct SelectedOption;

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    SettingsDisplay,
    SettingsRules,
    BackToMainMenu,
}

#[derive(Component)]
enum RulesButtonAction
{
    DecreasePlayers,
    IncreasePlayers,
    ToggleStackable,
}

#[derive(Component)]
struct PlayersNumberText;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // At start, the menu is not enabled. This will be changed in `menu_setup` when
            // entering the `GameState::Menu` state.
            // Current screen in the menu is handled by an independent state from `GameState`
            .add_state::<MenuState>()
            .add_system(menu_setup.in_schedule(OnEnter(GameState::Menu)))
            // Systems to handle the main menu screen
            .add_systems((
                main_menu_setup.in_schedule(OnEnter(MenuState::Main)),
                despawn_screen::<OnMainMenuScreen>.in_schedule(OnExit(MenuState::Main)),
            ))
            // Systems to handle the display settings screen
            .add_systems((
                display_settings_menu_setup.in_schedule(OnEnter(MenuState::SettingsDisplay)),
                setting_button::<DisplayQuality>.in_set(OnUpdate(MenuState::SettingsDisplay)),
                despawn_screen::<OnDisplaySettings>.in_schedule(OnExit(MenuState::SettingsDisplay)),
            ))
            // Systems to handle the sound settings screen
            .add_systems((
                rules_settings_menu_setup.in_schedule(OnEnter(MenuState::SettingsRules)),
                rules_button_action.in_set(OnUpdate(MenuState::SettingsRules)),
                despawn_screen::<OnRulesSettings>.in_schedule(OnExit(MenuState::SettingsRules)),
            ))
            // Common systems to all screens that handles buttons behaviour
            .add_systems((menu_action, button_system, test).in_set(OnUpdate(GameState::Menu)));
    }
}


fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

// This system handles changing all buttons color based on mouse interaction
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

// This system updates the settings when a new value for a setting is selected, and marks
// the button as the one currently selected
fn setting_button<T: Resource + Component + PartialEq + Copy>(
    interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    mut selected_query: Query<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    for (interaction, button_setting, entity) in &interaction_query
    {
        if *interaction == Interaction::Clicked && *setting != *button_setting
        {
            let (previous_button, mut previous_color) = selected_query.single_mut();
            *previous_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = *button_setting;
        }
    }
}

fn rules_button_action(
    interaction_q: Query<(&Interaction, &RulesButtonAction, Entity), (Changed<Interaction>, With<Button>)>,
    mut text_q: Query<&mut Text, With<PlayersNumberText>>,
    mut rules: ResMut<Rules>,
    mut commands: Commands,
) {
    for (interaction, button_action, entity) in &interaction_q
    {
        let mut text_number = text_q.single_mut();
        if *interaction == Interaction::Clicked
        {
            match button_action {
                RulesButtonAction::DecreasePlayers => {
                    if rules.num_players > 1 { rules.num_players -= 1; } else { rules.num_players = 1 }
                    text_number.sections[0].value = format!("{}", rules.num_players)
                },
                RulesButtonAction::IncreasePlayers => {
                    rules.num_players += 1;
                    text_number.sections[0].value = format!("{}", rules.num_players)
                },
                RulesButtonAction::ToggleStackable => {
                    rules.stackable_cards = !rules.stackable_cards;
                    if rules.stackable_cards { commands.entity(entity).insert(SelectedOption); }
                    else { commands.entity(entity).remove::<SelectedOption>(); }
                }
            }
        }
    }
}

fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>)
{
    let font = asset_server.load("fonts/Vividly.otf");
    // Common style for all buttons on the screen
    let button_style = Style {
        size: Size::new(Val::Px(250.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: font.clone(),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::DARK_GREEN.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Game name
                    parent.spawn(
                        TextBundle::from_section(
                            "Dos",
                            TextStyle {
                                font: font.clone(),
                                font_size: 100.0,
                                color: Color::GOLD,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(100.0)),
                            ..default()
                        }),
                    );
                    // Play
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Play",
                                button_text_style.clone(),
                            ));
                        });
                    // Display
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::SettingsDisplay,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Display",
                                button_text_style.clone(),
                            ));
                        });
                    // Rules
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::SettingsRules,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Rules",
                                button_text_style.clone(),
                            ));
                        });
                });
        });
}

fn display_settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    display_quality: Res<DisplayQuality>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: asset_server.load("fonts/Vividly.otf"),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnDisplaySettings,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::DARK_GREEN.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Create a new `NodeBundle`, this time not setting its `flex_direction`. It will
                    // use the default value, `FlexDirection::Row`, from left to right.
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::DARK_GREEN.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Display a label for the current setting
                            parent.spawn(TextBundle::from_section(
                                "Display Quality",
                                button_text_style.clone(),
                            ));
                            // Display a button for each possible value
                            for quality_setting in [
                                DisplayQuality::Low,
                                DisplayQuality::Medium,
                                DisplayQuality::High,
                            ] {
                                let mut entity = parent.spawn(ButtonBundle {
                                    style: Style {
                                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                                        ..button_style.clone()
                                    },
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                });
                                entity.insert(quality_setting).with_children(|parent| {
                                    parent.spawn(TextBundle::from_section(
                                        format!("{quality_setting:?}"),
                                        button_text_style.clone(),
                                    ));
                                });
                                if *display_quality == quality_setting {
                                    entity.insert(SelectedOption);
                                }
                            }
                        });
                    // Display the back button to return to the settings screen
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::BackToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Back", button_text_style));
                        });
                });
        });
}

fn rules_settings_menu_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rules: Res<Rules>,
) {
    let button_style = Style {
        size: Size::new(Val::Px(200.0), Val::Px(65.0)),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font: asset_server.load("fonts/Vividly.otf"),
        font_size: 40.0,
        color: TEXT_COLOR,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnRulesSettings,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::DARK_GREEN.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Stackable cards
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::DARK_GREEN.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Stackable cards",
                                button_text_style.clone(),
                            ));
                            let mut entity = parent.spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(40.0), Val::Px(40.0)),
                                    ..button_style.clone()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            });
                            entity.insert(RulesButtonAction::ToggleStackable);
                            if rules.stackable_cards {entity.insert(SelectedOption);}

                        });
                    // Number of Players
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::DARK_GREEN.into(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Players: ",
                                button_text_style.clone(),
                            ));
                            parent.spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(40.0), Val::Px(40.0)),
                                    ..button_style.clone()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            }).insert(RulesButtonAction::DecreasePlayers).with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "-",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        ..button_text_style.clone()
                                    }
                                ));
                            });
                            parent.spawn((TextBundle::from_section(
                                format!("{}", rules.num_players),
                                button_text_style.clone(),
                            ), PlayersNumberText));
                            parent.spawn(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(40.0), Val::Px(40.0)),
                                    ..button_style.clone()
                                },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            }).insert(RulesButtonAction::IncreasePlayers).with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "+",
                                    TextStyle {
                                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                        ..button_text_style.clone()
                                    }
                                ));
                            });
                        });
                    // Back to menu
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::BackToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Back", button_text_style));
                        });
                });
        });
}

fn menu_action(
    interaction_query: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query
    {
        if *interaction == Interaction::Clicked
        {
            match menu_button_action
            {
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                    menu_state.set(MenuState::Disabled);
                },
                MenuButtonAction::SettingsDisplay => menu_state.set(MenuState::SettingsDisplay),
                MenuButtonAction::SettingsRules => menu_state.set(MenuState::SettingsRules),
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
            }
        }
    }
}

fn test(
    key: Res<Input<KeyCode>>,
    rules: Res<Rules>,
) {
    if key.just_pressed(KeyCode::Q)
    {
        info!("{:?}", *rules);
    }
}
