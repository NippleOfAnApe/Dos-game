/************************************************
*                                               *
*   Ви, байстрюки катів осатанілих,             *
*   Не забувайте, виродки, ніде:                *
*   Народ мій є! В його гарячих жилах           *
*   Козацька кров пульсує і гуде!               *
*                                               *
*                       Василь Симоненко        *
*                                               *
************************************************/

use std::fmt::{self, Debug};   // Conver enum variants into string
use bevy::prelude::*;
use crate::{despawn_screen, GameState, LOBBY_PLAYERS};
use rand::{seq::SliceRandom, thread_rng};
use num_derive::FromPrimitive;  //derive a trait on enum to access it with integer
use num::FromPrimitive;         //access enum values via integer


//----------------------------------------------------------------------------------
// Game configurations
//----------------------------------------------------------------------------------

const HAND_SIZE: usize = 7;
const PLAYERS_DISTANCE: f32 = 380.0;

const ENEMY_CARD_SCALE: Vec3 = Vec3::new(0.6, 0.6, 0.0);
const PLAYER_CARD_SCALE: Vec3 = Vec3::new(0.8, 0.8, 0.0);
const DECK_CARD_SCALE: Vec3 = Vec3::new(0.8, 0.8, 0.0);
const DISCARD_CARD_SCALE: Vec3 = Vec3::new(1.0, 1.0, 0.0);
const PLAYER_CARDS_SPACING: f32 = 50.0;
const ENEMY_CARDS_SPACING: f32 = 12.0;

const DECK_DISCARD_DISTANCE: f32 = 100.0;
const FALLBACK_DECK_COLLIDER: Vec2 = Vec2::new(100.0, 150.0);

const NAME_TEXT_OFFSET_X: f32 = -50.0;
const NAME_TEXT_OFFSET_Y: f32 = 150.0;


#[derive(Clone, Copy, Debug, FromPrimitive)]
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

#[derive(Clone, Copy, Debug, FromPrimitive)]
enum Suit {
    Red,
    Blue,
    Yellow,
    Green,
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
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

#[derive(Component, Clone, Copy, Debug)]
struct Card {
    rank: Rank,
    suite: Suit,
    pos: Option<Vec3>,
}

#[derive(Bundle)]
struct CardBundle {
    card: Card,
    owner: PlayerName,
    #[bundle]
    sprite: SpriteBundle,
}

#[derive(Component, Debug)]
struct Deck {
    cards: Vec<Card>,
}

#[derive(Component, Debug)]
struct DiscardPile {
    cards: Vec<Card>,
}

#[derive(Component, Debug)]
struct Player {
    pos: Vec3,
    cards: Vec<Card>,
}

#[derive(Resource)]
struct GameRules {
    move_made: bool,
    player_turn: PlayerName,
}

struct PlayCard(usize);

#[derive(Default)]
struct DrawCard;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameRules {
            move_made: false,
                player_turn: PlayerName::MainPlayer,
            })
            .add_event::<DrawCard>()
            .add_event::<PlayCard>()
            .add_system(setup.in_schedule(OnEnter(GameState::InGame)))
            .add_system(menu.in_set(OnUpdate(GameState::InGame)))
            .add_system(check_deck_bounds.run_if(mouse_pressed).in_set(OnUpdate(GameState::InGame)))
            // EventWriter goes before EventReader
            .add_system(draw_card.after(check_deck_bounds).in_set(OnUpdate(GameState::InGame)))
            .add_system(play_card.after(check_deck_bounds).in_set(OnUpdate(GameState::InGame)))
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
        let mut player_hand: Vec<Card> = new_deck.drain(..HAND_SIZE).collect();

        // Spawn images of each card in hand.
        // Assign to it a corresponding card, player name and position
        player_hand.iter_mut().enumerate().for_each(|(j, card)| {
            // If not a MainPlayer
            if i != 0 {
                card.pos = Some(Vec3::new(x + (j as f32) * ENEMY_CARDS_SPACING, y, j as f32));

                commands.spawn(CardBundle {
                    owner: PlayerName::from_usize(i).unwrap(),
                    sprite: SpriteBundle {
                        // If hand is not player's - load a card's back image
                        texture: tex_back.clone(),
                        transform: Transform {
                            translation: card.pos.unwrap(),
                            scale: ENEMY_CARD_SCALE,
                            ..default()
                        },
                        ..default()
                    },
                    card: *card,
                });
            } else {
                card.pos = Some(Vec3::new(x + (j as f32) * PLAYER_CARDS_SPACING, y, j as f32));
                // If MainPlayer's hand - load a front image instead instead of a back image
                let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());

                commands.spawn(CardBundle {
                    owner: PlayerName::from_usize(i).unwrap(),
                    sprite: SpriteBundle {
                        texture: asset_server.load(image_name),
                        transform: Transform {
                            translation: card.pos.unwrap(),
                            scale: PLAYER_CARD_SCALE,
                            ..default()
                        },
                        ..default()
                    },
                    card: *card,
                });
            }
        });

        // Text of Player's name on top of a hand
        commands.spawn(Text2dBundle {
            text: Text::from_section(format!("Player {}", i + 1), TextStyle { font: font.clone(), font_size: 50.0, color: Color::WHITE }),
            transform: Transform::from_xyz(x - NAME_TEXT_OFFSET_X, y - NAME_TEXT_OFFSET_Y, 0.0),
            ..default()
        });

        // Spawn a player and give him a name from enum of PlayerName
        commands.spawn((PlayerName::from_usize(i).unwrap(),
            Player { pos: Vec3::new(x, y, 0.0), cards: player_hand },
        ));
    }

    // Put a card from a deck to a discard pile
    // TODO check if no cards left in a deck
    let mut pile_top_card = new_deck.pop().unwrap();
    pile_top_card.pos = Some(Vec3::new(DECK_DISCARD_DISTANCE, 0.0, 0.0));
    let image_name = format!("{}_{}.png", pile_top_card.suite.to_string(), pile_top_card.rank.to_string());
    let discard_pile = vec![pile_top_card];
    commands.spawn((
        DiscardPile { cards: discard_pile },
        SpriteBundle {
            texture: asset_server.load(image_name),
            transform: Transform::from_translation(pile_top_card.pos.unwrap()).with_scale(DISCARD_CARD_SCALE),
            ..default()
        },
        pile_top_card,
    ));

    // Spawn a deck and put unused cards there
    commands.spawn((
        Deck { cards: new_deck },
        SpriteBundle {
            texture: tex_back.clone(),
            transform: Transform::from_xyz(-DECK_DISCARD_DISTANCE, 0.0, 0.0).with_scale(DECK_CARD_SCALE),
            ..default()
        }
    ));
}

fn mouse_pressed(mouse_button_input: Res<Input<MouseButton>>) -> bool {
    mouse_button_input.just_pressed(MouseButton::Left)
}

fn check_deck_bounds(
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    deck_q: Query<&Handle<Image>, With<Deck>>,
    card_q: Query<(&Handle<Image>, &PlayerName)>,
    player_q: Query<(&Player, &PlayerName)>,
    all_images: Res<Assets<Image>>,
    mut deck_event: EventWriter<DrawCard>,
    mut card_event: EventWriter<PlayCard>,
) {
    let window = window_q.single();
    let (camera, camera_pos) = camera_q.single();

    // First make sure that click was inside a window.
    // If yes - transform cursor's position from global position to 2D world position
    if let Some(cursor_pos) = window.cursor_position().and_then(|cursor| camera.viewport_to_world_2d(camera_pos, cursor)) {

        /*********Check if clicked on your hand*********/

        // Get the size of a card to determine a collider
        let mut card_image_size: Vec2 = Vec2::ZERO;
        for (card_image, name) in card_q.iter() {
            if *name == PlayerName::MainPlayer {
                // TODO prevent loading game dureing setup if image couldn't be loaded
                card_image_size = if let Some(image) = all_images.get(&card_image) { image.size() } else { FALLBACK_DECK_COLLIDER };
                break;
            }
        }
        // Bounds for deck image
        let card_x_offset: f32 = card_image_size.x * PLAYER_CARD_SCALE.x / 2.0;
        let card_y_offset: f32 = card_image_size.y * PLAYER_CARD_SCALE.y / 2.0;

        // Detrmin how many cards are in a hand and where is player located
        let mut counter: usize = 0;
        let mut player_pos: Vec3 = Vec3::ZERO;
        for (player, name) in player_q.iter() {
            if *name == PlayerName::MainPlayer {
                counter = player.cards.len();
                player_pos = player.pos;
                break;
            }
        }

        let left_edge = player_pos.x - card_x_offset;
        //          +y_offset
        //           ___ 
        //          |   |
        //-x_offset | · | +x_offset
        //          |___|
        //          -y_offset
        if cursor_pos.x > left_edge &&
            cursor_pos.x < player_pos.x + card_x_offset + PLAYER_CARDS_SPACING * (counter - 1) as f32 &&
            cursor_pos.y > player_pos.y - card_y_offset &&
            cursor_pos.y < player_pos.y + card_y_offset {
                let num_card = get_card_index(player_pos.x - card_x_offset, cursor_pos.x, counter);
                card_event.send(PlayCard(num_card));
                return;
        }

        /*********Check if clicked on a deck*********/

        let deck_image = deck_q.single();
        // If deck's image was properly loaded - find it using a deck's image handle
        // from all images in a world. Otherwise use user defined size
        let deck_image_size: Vec2 = if let Some(image) = all_images.get(&deck_image) { image.size() } else { FALLBACK_DECK_COLLIDER };
        // Bounds for deck image
        let x_offset = deck_image_size.x * DECK_CARD_SCALE.x / 2.0;
        let y_offset = deck_image_size.y * DECK_CARD_SCALE.y / 2.0;

        if cursor_pos.x > -DECK_DISCARD_DISTANCE - x_offset &&
            cursor_pos.x < -DECK_DISCARD_DISTANCE + x_offset &&
            cursor_pos.y > -y_offset &&
            cursor_pos.y < y_offset {
                deck_event.send_default();
        }
    }
}

fn draw_card(
    mut commands: Commands,
    mut player_q: Query<(&mut Player, &PlayerName)>,
    mut deck_q: Query<&mut Deck>,
    asset_server: Res<AssetServer>,
    mut event: EventReader<DrawCard>,
) {
    // Continue only on receiving an event
    for _ in event.iter() {
        let mut deck = deck_q.single_mut();

        // Check whether a deck has cards
        if let Some(mut card) = deck.cards.pop() {
            for (mut player, name) in player_q.iter_mut() {
                if *name == PlayerName::MainPlayer {
                    // Calculate position of a new card depending on how many cards are in a hand
                    let counter = player.cards.len();
                    card.pos = Some(Vec3::new(player.pos.x + PLAYER_CARDS_SPACING * counter as f32, player.pos.y, counter as f32));
                    player.cards.push(card);
                }
            }

            // Load and spawn an image with a new new card
            let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());
            commands.spawn(CardBundle {
                card,
                sprite: SpriteBundle {
                    texture: asset_server.load(image_name),
                    transform: Transform {
                        translation: card.pos.unwrap(),
                        scale: PLAYER_CARD_SCALE,
                        ..default()
                    },
                    ..default()
                },
                owner: PlayerName::MainPlayer,
            });
        }
    }
}

fn play_card(
    mut play_event: EventReader<PlayCard>,
    mut discard_q: Query<(&mut Handle<Image>, &mut Card, &mut DiscardPile)>,
    mut player_q: Query<(&mut Player, &PlayerName)>,
    asset_server: Res<AssetServer>,
) {
    for event in play_event.iter() {
        let (mut image, mut top_card, mut pile) = discard_q.single_mut();

        for (mut player, name) in player_q.iter_mut() {
            if *name == PlayerName::MainPlayer {
                let mut card = player.cards.remove(event.0);
                card.pos = Some(Vec3::new(DECK_DISCARD_DISTANCE, 0.0, 0.0));
                *top_card = card;
                pile.cards.push(card);

                let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());
                *image = asset_server.load(image_name);
            }
        }

        info!("Card #: {}", event.0);
    }
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

fn test(
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut game_rules: ResMut<GameRules>,
    player_q: Query<&Player>,
    card_q: Query<&Card>,
    transform_q: Query<(&Transform, &Card)>,
    key: Res<Input<KeyCode>>,
    window_q: Query<&Window>,
) {
    let window = window_q.single();

    if key.just_pressed(KeyCode::D) {
        let (camera, camera_transform) = camera_q.single();
        let cursor_pos = window.cursor_position().unwrap();
        let final_dest = camera.viewport_to_world(camera_transform, cursor_pos).map(|ray| ray.origin.truncate()).unwrap();
        info!("ray pos: {:?}", final_dest);
    }

    if key.just_pressed(KeyCode::T) {
        info!("Physical cursor position: {}", window.physical_cursor_position().unwrap());
        info!("Cursor position: {}", window.cursor_position().unwrap());
    }

    if key.just_pressed(KeyCode::R) {
        game_rules.move_made = !game_rules.move_made;
    }
}

fn get_card_index(left: f32, x: f32, counter: usize) -> usize {
    for i in 0..counter {
        if x < left + PLAYER_CARDS_SPACING * (i + 1) as f32 {
            return i;
        }
    }
    counter - 1
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
