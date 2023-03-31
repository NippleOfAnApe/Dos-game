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
const CARDS_ENEMY_SCALE: Vec3 = Vec3::new(0.6, 0.6, 0.0);
const CARD_ENEMY_SPACING: f32 = 12.0;
const CARDS_PLAYER_SCALE: Vec3 = Vec3::new(0.8, 0.8, 0.0);
const CARD_PLAYER_SPACING: f32 = 50.0;
const NAME_TEXT_OFFSET_X: f32 = -50.0;
const NAME_TEXT_OFFSET_Y: f32 = 150.0;
const DECK_DISCARD_DISTANCE: f32 = 100.0;
const FALLBACK_DECK_COLLIDER: Vec2 = Vec2::new(100.0, 150.0);

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
    // TODO might be redundant in the future
    owner: Option<PlayerName>,
}

#[derive(Bundle)]
struct CardBundle {
    card: Card,
    #[bundle]
    sprite: SpriteBundle,
}

#[derive(Component, Debug)]
struct Deck {
    cards: Vec<Card>,
}

#[derive(Component, Debug)]
struct DiscardPile(Vec3);

#[derive(Component, Debug)]
struct Player {
    name: PlayerName,
    pos: Vec3,
    cards: Vec<Card>,
}

#[derive(Resource)]
struct GameRules {
    move_made: bool,
    player_turn: PlayerName,
}

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
            .add_system(setup.in_schedule(OnEnter(GameState::InGame)))
            .add_system(menu.in_set(OnUpdate(GameState::InGame)))
            .add_system(check_deck_bounds.run_if(mouse_pressed).in_set(OnUpdate(GameState::InGame)))
            // Update after drawing so that EventWriter goes before EventReader
            .add_system(draw_card.after(check_deck_bounds).in_set(OnUpdate(GameState::InGame)))
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
        let mut player_hand: Vec<Card> = new_deck.drain(..HAND_SIZE).collect();

        // assign a position + owner for each card in hand
        player_hand.iter_mut().enumerate().for_each(|(j, card)| {
            card.owner = Some(PlayerName::from_u32(i).unwrap());
            // If holder is not a MainPlayer then use enemy card spacing
            card.pos = Some(Vec3::new(
                    x + (j as f32) * if card.owner != Some(PlayerName::MainPlayer) { CARD_ENEMY_SPACING } else { CARD_PLAYER_SPACING },
                    y, j as f32));
        });

        // Spawn and render every card in a hand
        // TODO combin this and position assignment into single iter
        player_hand.iter().for_each(|card| {
            commands.spawn(CardBundle {
                sprite: SpriteBundle {
                    // If hand is not a player's - draw a card's back image instead
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
                },
                card: *card,
            });
        });

        // Text of Player's name on top of a hand
        commands.spawn(Text2dBundle {
            text: Text::from_section(format!("Player {}", i + 1), TextStyle { font: font.clone(), font_size: 50.0, color: Color::WHITE }),
            transform: Transform::from_xyz(x - NAME_TEXT_OFFSET_X, y - NAME_TEXT_OFFSET_Y, 0.0),
            ..default()
        });

        // Spawn a player and give him a name from enum of PlayerName
        commands.spawn(Player {
            name: PlayerName::from_u32(i).unwrap(),
            pos: Vec3::new(x, y, 0.0),
            cards: player_hand,
        });
    }

    // Put a card from a deck to a discard pile
    let mut pile_top_card = new_deck.pop().unwrap();
    pile_top_card.owner = Some(PlayerName::Void);
    pile_top_card.pos = Some(Vec3::new(DECK_DISCARD_DISTANCE, 0.0, 0.0));
    let image_name = format!("{}_{}.png", pile_top_card.suite.to_string(), pile_top_card.rank.to_string());
    commands.spawn((
        DiscardPile(pile_top_card.pos.unwrap()),
        CardBundle {
            sprite: SpriteBundle {
                texture: asset_server.load(image_name),
                transform: Transform::from_translation(pile_top_card.pos.unwrap()).with_scale(CARDS_PLAYER_SCALE),
                ..default()
            },
            card: pile_top_card
        }
    ));

    // Create a vector with references to spawned cards
    commands.spawn((
        Deck { cards: new_deck },
        SpriteBundle {
            texture: tex_back.clone(),
            transform: Transform::from_xyz(-DECK_DISCARD_DISTANCE, 0.0, 0.0).with_scale(CARDS_PLAYER_SCALE),
            ..default()
        }
    ));

    info!("cards have been dealt");
}

fn mouse_pressed(mouse_button_input: Res<Input<MouseButton>>) -> bool {
    mouse_button_input.just_pressed(MouseButton::Left)
}

fn check_deck_bounds(
    camera_q: Query<(&Camera, &GlobalTransform)>,
    window_q: Query<&Window>,
    deck_q: Query<(&Handle<Image>, &Transform), With<Deck>>,
    all_images: Res<Assets<Image>>,
    mut card_event: EventWriter<DrawCard>,
) {
    let window = window_q.single();
    let (camera, camera_pos) = camera_q.single();

    // First make sure that click was inside a window.
    // If yes - transform cursor's position from global position to 2D world position
    if let Some(cursor_pos) = window.cursor_position().and_then(|cursor| camera.viewport_to_world_2d(camera_pos, cursor)) {
        let (deck_image, deck_pos) = deck_q.single();

        // If deck's image was properly loaded - find it using a deck's image handle
        // from all images in a world. Otherwise use user defined size
        let image_size: Vec2 = if let Some(image) = all_images.get(&deck_image) { image.size() } else { FALLBACK_DECK_COLLIDER };
        // Bounds for deck image
        let x_offset = image_size.x * CARDS_PLAYER_SCALE.x / 2.0;
        let y_offset = image_size.y * CARDS_PLAYER_SCALE.y / 2.0;

        //          +y_offset
        //           ___ 
        //          |   |
        //-x_offset | · | +x_offset
        //          |___|
        //          -y_offset
        if cursor_pos.x > deck_pos.translation.x - x_offset &&
            cursor_pos.x < deck_pos.translation.x + x_offset &&
            cursor_pos.y > deck_pos.translation.y - y_offset &&
            cursor_pos.y < deck_pos.translation.y + y_offset {
                card_event.send_default();
        }
    }
}

fn draw_card(
    mut commands: Commands,
    mut player_q: Query<&mut Player>,
    mut deck_q: Query<&mut Deck>,
    mut event: EventReader<DrawCard>,
    asset_server: Res<AssetServer>,
) {
    // Continue only on receiving an event
    for _ in event.iter() {
        let mut deck = deck_q.single_mut();

        // Check whether a deck has cards
        if let Some(mut card) = deck.cards.pop() {
            card.owner = Some(PlayerName::MainPlayer);
            for mut player in player_q.iter_mut() {
                if let PlayerName::MainPlayer = player.name {
                    // Calculate position of a new card depending on how many cards are in a hand
                    let counter = player.cards.len();
                    card.pos = Some(Vec3::new(player.pos.x + CARD_PLAYER_SPACING * counter as f32, player.pos.y, counter as f32));
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
                        scale: CARDS_PLAYER_SCALE,
                        ..default()
                    },
                    ..default()
                },
            });
        }
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

    if key.just_pressed(KeyCode::Space) {
        for player in &player_q {
            if let PlayerName::MainPlayer = player.name {
                player.cards.iter().for_each(|card| { info!("{}", card.pos.unwrap().z)});
            }
        }
    }

    if key.just_pressed(KeyCode::R) {
        game_rules.move_made = !game_rules.move_made;
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
