use std::fmt::{self, Debug, Display};   // Conver enum variants into string
use bevy::prelude::*;
use crate::{despawn_screen, GameState, LOBBY_PLAYERS};
use rand::{seq::SliceRandom, thread_rng};
use num_derive::FromPrimitive;  //derive a trait on enum to access it with integer
use num::FromPrimitive;         //access enum values via integer

const HAND_SIZE: usize = 7;
const PLAYERS_DISTANCE: f32 = 380.0;
const CARDS_ENEMY_SCALE: Vec3 = Vec3::new(0.6, 0.6, 0.0);
const CARD_ENEMY_SPACING: f32 = 12.0;
const CARDS_PLAYER_SCALE: Vec3 = Vec3::new(0.8, 0.8, 0.0);
const CARD_PLAYER_SPACING: f32 = 50.0;
const NAME_TEXT_OFFSET_X: f32 = -50.0;
const NAME_TEXT_OFFSET_Y: f32 = 150.0;
const DECK_DISCARD_DISTANCE: f32 = 100.0;

#[derive(Component, Clone, Copy, Debug, FromPrimitive)]
enum Rank {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Skip,
    Reverse,
    Draw2,
    Wild,
    WildDraw4
}

#[derive(Component, Clone, Copy, Debug, FromPrimitive)]
enum Suit {
    Red,
    Blue,
    Yellow,
    Green,
}

#[derive(Debug, Component, PartialEq, Eq, Clone, Copy, FromPrimitive)]
enum PlayerName{
    MainPlayer,
    Player1,
    Player2,
    Player3,
    Player4,
    Player5,
    Player6,
    Player7,
    Player8,
    Player9,
    Void,
}

#[derive(Clone, Copy, Component, Debug)]
struct Card {
    rank: Rank,
    suite: Suit,
    pos: Option<Vec3>,
    owner: Option<PlayerName>,
}

#[derive(Bundle)]
struct CardBundle {
    card: Card,
    #[bundle]
    sprite: SpriteBundle,
}

#[derive(Component, Debug)]
struct Deck;

#[derive(Component, Debug)]
struct DiscardPile(Vec3);

#[derive(Component, Debug)]
struct Player {
    name: PlayerName,
    pos: Vec3,
}

#[derive(Resource)]
struct GameRules {
    move_made: bool,
    player_turn: PlayerName,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameRules {
            move_made: false,
                player_turn: PlayerName::MainPlayer,
            })
            .add_system(setup.in_schedule(OnEnter(GameState::InGame)))
            .add_system(menu.in_set(OnUpdate(GameState::InGame)))
            // TODO it doesnt remove entities without transform
            .add_system(despawn_screen::<Transform, Camera>.in_schedule(OnExit(GameState::InGame)))
            .add_system(test);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let center = Vec3::ZERO;
    let angle: f32 = 360.0 / LOBBY_PLAYERS as f32;
    let font = asset_server.load("FiraSans-Bold.ttf");
    let tex_back = asset_server.load("Back.png");

    //let card_variants = mem::variant_count::<Rank>();
    // Create a deck and shuffle cards in it
    let mut new_deck: Vec<Card> = Vec::new();
    for color in 0..4 {
        for rank in 0..13 {
            new_deck.push(Card {
                rank: Rank::from_u32(rank).unwrap(),
                suite: Suit::from_u32(color).unwrap(),
                pos: None,
                owner: None,
            });
        }
    }
    new_deck.shuffle(&mut thread_rng());

    // Crate Players and assign to them position + text + hand of cards
    for i in 0..LOBBY_PLAYERS {
        // Calculate X and Y position
        let theta = (-90. + i as f32 * angle).to_radians();
        let x = center.x + theta.cos() * PLAYERS_DISTANCE;
        let y = center.y + theta.sin() * PLAYERS_DISTANCE;

        // move cards from a deck to a player's hand
        let mut player_hand: Vec<Card> = new_deck.drain(..=HAND_SIZE).collect();

        // assign a position + owner for each card in hand
        player_hand.iter_mut().enumerate().for_each(|(j, card)| {
            card.owner = Some(PlayerName::from_u32(i).unwrap());
            // If holder is not a MainPlayer then use enemy card spacing
            card.pos = Some(Vec3::new(
                    x + (j as f32) * if card.owner != Some(PlayerName::MainPlayer) { CARD_ENEMY_SPACING } else { CARD_PLAYER_SPACING },
                    y, j as f32));
        });

        // Spawn a player and give him a name from enum of PlayerName
        commands.spawn(Player {
            name: PlayerName::from_u32(i).unwrap(),
            pos: Vec3::new(x, y, 0.0),
        });

        // Spawn and render every card in a hand
        // TODO combin this and position assignment into single iter
        player_hand.iter().for_each(|card| {
            commands.spawn(CardBundle {
                card: *card,
                sprite: SpriteBundle {
                    // If hand is not player's draw a card's back instead
                    texture: if card.owner != Some(PlayerName::MainPlayer) { tex_back.clone() } else {
                        let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());
                        asset_server.load(image_name)
                    },
                    transform: Transform {
                        translation: card.pos.unwrap(),
                        scale: if card.owner != Some(PlayerName::MainPlayer) {CARDS_ENEMY_SCALE} else {CARDS_PLAYER_SCALE},
                        ..default()
                    },
                    ..default()
                }
            });
        });
        // Text of Player's name on top of a hand
        commands.spawn(Text2dBundle {
            text: Text::from_section(format!("Player {}", i + 1), TextStyle { font: font.clone(), font_size: 50.0, color: Color::WHITE }),
            transform: Transform::from_xyz(x - NAME_TEXT_OFFSET_X, y - NAME_TEXT_OFFSET_Y, 0.0),
            ..default()
        });
    }

    // Put a card from a deck to a discard pile
    let mut pile_top_card = new_deck.pop().unwrap();
    pile_top_card.owner = Some(PlayerName::Void);
    let image_name = format!("{}_{}.png", pile_top_card.suite.to_string(), pile_top_card.rank.to_string());
    commands.spawn((
        DiscardPile(Vec3::new(DECK_DISCARD_DISTANCE, 0.0, 0.0)),
        CardBundle {
            card: pile_top_card,
            sprite: SpriteBundle {
                texture: asset_server.load(image_name),
                transform: Transform::from_xyz(DECK_DISCARD_DISTANCE, 0.0, 0.0).with_scale(CARDS_PLAYER_SCALE),
                ..default()
            }
        }
    ));

    //Rest of the cards stay inside a deck
    commands.spawn((
        Deck,
        SpriteBundle {
            texture: tex_back.clone(),
            transform: Transform::from_xyz(-DECK_DISCARD_DISTANCE, 0.0, 0.0).with_scale(CARDS_PLAYER_SCALE),
            ..default()
        }
    ));
    commands.spawn_batch(new_deck);

    info!("cards have been dealt");
}

fn menu(
    mut next_state: ResMut<NextState<GameState>>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Escape) {
        info!("Going to menu...");
        next_state.set(GameState::Menu);
    }
}

fn update_renderer(game_rules: Res<GameRules>) -> bool {
    game_rules.move_made
}

fn test(
    deck_query: Query<&Deck>,
    pile_q: Query<&DiscardPile>,
    players_q: Query<&Player>,
    transforms: Query<&Transform>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        let deck = deck_query.single();
        let pile = pile_q.single();
        // info!("# of cards in deck: {:?}", deck.cards.len());
        // info!("Cards in deck{:?}", deck.cards);
        // info!("Cards in pile{:?}", pile.cards);

        let mut counter: u32 = 0;
        for _ in &players_q {
            counter += 1;
        }
        let mut counter2: u32 = 0;
        for _ in &deck_query {
            counter += 1;
        }
        info!("players: {:?}", counter);
        info!("decks: {:?}", counter2);
    }

    if key.just_pressed(KeyCode::T) {
        for positions in transforms.iter() {
            info!("Pos: {:?}", positions.translation);
        }
    }

    if key.just_pressed(KeyCode::A) {
        for player in players_q.iter() {
            if player.name == PlayerName::MainPlayer {
                info!("Player: {:?}", player.name);
                // info!("Cards in hand: {:?}", player.cards);
            }
        }
    }

    if key.just_pressed(KeyCode::D) {
        for player in players_q.iter() {
            if player.name == PlayerName::Player2 {
                info!("Player: {:?}", player.name);
                // info!("Cards in hand: {:?}", player.cards);
            }
        }
    }
}

// Allows to format an enum into sting
impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
