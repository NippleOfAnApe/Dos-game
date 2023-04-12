use bevy::prelude::*;
use crate::{GameState, Rules};
use crate::game::{GameItem, GameplayState, PlayerName};
use crate::menu::{TEXT_COLOR, NORMAL_BUTTON, PRESSED_BUTTON, HOVERED_BUTTON};
use num::FromPrimitive;         //access enum values via integer

#[derive(Component)]
enum InGameButtonAction {
    Skip,
    Menu,
}

#[derive(Default)]
struct GoMenu;

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<GoMenu>()
            .add_system(ui_setup.in_schedule(OnEnter(GameState::Game)))
            .add_systems((keyboard_action, ui_button_action, button_colors, go_to_menu).in_set(OnUpdate(GameState::Game)));
    }
}

fn ui_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rules: Res<Rules>,
) {
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(160.0), Val::Px(70.0)),
                    position: UiRect {
                        left: Val::Percent(3.0),
                        top: Val::Percent(10.0),
                        ..default()
                    },
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            InGameButtonAction::Menu,
            GameItem,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Menu",
                TextStyle {
                    font: asset_server.load("fonts/Vividly.otf"),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                }
            ));
            parent.spawn(TextBundle::from_section(
                "[ esc ]",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: TEXT_COLOR,
                }
            ));
        });

    if rules.no_skip || rules.num_players == 1 { return; }

    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(160.0), Val::Px(70.0)),
                    position: UiRect {
                        left: Val::Percent(10.0),
                        top: Val::Percent(85.0),
                        ..default()
                    },
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            InGameButtonAction::Skip,
            GameItem,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Skip",
                TextStyle {
                    font: asset_server.load("fonts/Vividly.otf"),
                    font_size: 40.0,
                    color: TEXT_COLOR,
                }
            ));
            parent.spawn(TextBundle::from_section(
                "[ space ]",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 20.0,
                    color: TEXT_COLOR,
                }
            ));
        });
}

fn ui_button_action(
    interaction_q: Query<(&Interaction, &InGameButtonAction), (Changed<Interaction>, With<Button>)>,
    rules: Res<Rules>,
    mut gameplay_rules: ResMut<GameplayState>,
    mut menu_event: EventWriter<GoMenu>,
) {
    for (interaction, button_action) in &interaction_q
    {
        if *interaction == Interaction::Clicked
        {
            match button_action
            {
                InGameButtonAction::Skip => {
                    if !gameplay_rules.player_drawn_card { return; }

                    gameplay_rules.player_drawn_card = false;

                    if !rules.clockwise { gameplay_rules.player_turn = PlayerName::from_usize(1).unwrap(); }
                    else { gameplay_rules.player_turn = PlayerName::from_usize(rules.num_players - 1).unwrap(); }
                }
                InGameButtonAction::Menu => menu_event.send_default(),
            }
        }
    }
}

fn keyboard_action(
    key: Res<Input<KeyCode>>,
    rules: Res<Rules>,
    mut gameplay_rules: ResMut<GameplayState>,
    mut menu_event: EventWriter<GoMenu>,
) {
    if key.just_pressed(KeyCode::Escape)
    {
        menu_event.send_default();
    }

    if key.just_pressed(KeyCode::Space) && gameplay_rules.player_drawn_card
    {
        gameplay_rules.player_drawn_card = false;
        if !rules.clockwise
        {
            gameplay_rules.player_turn = PlayerName::from_usize(1).unwrap();
        }
        else
        {
            gameplay_rules.player_turn = PlayerName::from_usize(rules.num_players - 1).unwrap();
        }
    }
}

fn button_colors(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Clicked => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn go_to_menu(
    mut next_state: ResMut<NextState<GameState>>,
    mut event: EventReader<GoMenu>,
) {
    for _ in event.iter()
    {
        next_state.set(GameState::Menu);
    }
}
