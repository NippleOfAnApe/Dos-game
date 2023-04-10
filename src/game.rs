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
use std::mem;               // Conver variants of enum into integer
use bevy::prelude::*;
use crate::{despawn_screen, GameState, Rules};
use rand::{seq::SliceRandom, thread_rng};
use num_derive::FromPrimitive;  //derive a trait on enum to access it with integer
use num::FromPrimitive;         //access enum values via integer

//----------------------------------------------------------------------------------
//  Game configurations
//----------------------------------------------------------------------------------

const NAME_TEXT_COLOR: Color = Color::rgb(0.6, 0.8, 0.9);
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


#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive)]
enum Rank
{
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
    // Wild,
    // WildDraw4
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive)]
enum Suit
{
    Red,
    Blue,
    Yellow,
    Green,
}

#[derive(Debug)]
struct Card
{
    rank: Rank,
    suite: Suit,
    id: usize,
}

//----------------------------------------------------------------------------------
//  Components and Bundles
//----------------------------------------------------------------------------------

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
enum PlayerName
{
    MainPlayer,
    Player2,
    Player3,
    Player4,
    Player5,
    Player6,
    Player7,
    Player8,
    Player9,
    Player10,
    Player11,
    Player12,
    Void,
}

#[derive(Component, PartialEq, Eq, Copy, Clone, Debug)]
struct Id(usize);

#[derive(Component, Debug)]
struct Deck
{
    cards: Vec<Card>,
}

#[derive(Component, Debug)]
struct DiscardPile
{
    cards: Vec<Card>,
}

#[derive(Component, Debug)]
struct Player
{
    pos: Vec3,
    cards: Vec<Card>,
}

#[derive(Component)]
struct MainPlayer;

#[derive(Bundle)]
struct CardBundle
{
    id: Id,
    #[bundle]
    sprite: SpriteBundle,
}

// Tag component used to tag entities added in game
#[derive(Component)]
struct GameItem;

//----------------------------------------------------------------------------------
//  Resources and Events
//----------------------------------------------------------------------------------

#[derive(Resource)]
struct GameplayState
{
    player_turn: PlayerName,
}

struct PlayCard(usize);

#[derive(Default)]
struct DrawCard;

#[derive(Resource)]
struct BotWaiting {
    event_timer: Timer,
}

//----------------------------------------------------------------------------------
//  Plugin
//----------------------------------------------------------------------------------

pub struct GamePlugin;

impl Plugin for GamePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_event::<DrawCard>()
            .add_event::<PlayCard>()
            .init_resource::<BotWaiting>()
            .add_system(setup.in_schedule(OnEnter(GameState::Game)))
            .add_systems((menu, bot_play, test).in_set(OnUpdate(GameState::Game)))
            .add_system(check_deck_bounds.run_if(mouse_pressed).in_set(OnUpdate(GameState::Game)))
            // EventWriter goes before EventReader
            .add_system(draw_card.after(check_deck_bounds).in_set(OnUpdate(GameState::Game)))
            .add_system(play_card.after(check_deck_bounds).in_set(OnUpdate(GameState::Game)))
            // TODO it doesnt remove entities without transform
            .add_system(despawn_screen::<GameItem>.in_schedule(OnExit(GameState::Game)));
    }
}

//----------------------------------------------------------------------------------
//  Systems
//----------------------------------------------------------------------------------

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rules: Res<Rules>,
) {
    /********* Initialization *********/

    let center = Vec3::ZERO;
    let angle: f32 = 360.0 / rules.num_players as f32;
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let tex_back = asset_server.load("Back.png");

    // Automatically calculate the amount of possible combinations of card X color
    let card_variants = mem::variant_count::<Rank>();
    let card_colors = mem::variant_count::<Suit>();
    // Every card has a unique ID to help access SpriteBundle
    let mut ids: Vec<usize> = (0..card_colors * card_variants * 4).collect();
    ids.shuffle(&mut thread_rng());
    // Create a deck and shuffle cards in it
    let mut new_deck: Vec<Card> = Vec::new();
    for color in 0..card_colors
    {
        for rank in 0..card_variants
        {
            for _ in 0..2
            {
                new_deck.push(Card {
                    rank: Rank::from_usize(rank).unwrap(),
                    suite: Suit::from_usize(color).unwrap(),
                    id: ids.pop().unwrap(),
                });
            }
        }
    }
    new_deck.shuffle(&mut thread_rng());
    commands.insert_resource(GameplayState { player_turn: PlayerName::MainPlayer });

    /********* Create players *********/

    for i in 0..rules.num_players
    {
        // Calculate X and Y position
        let theta = (-90. + i as f32 * angle).to_radians();
        let x = center.x + theta.cos() * PLAYERS_DISTANCE;
        let y = center.y + theta.sin() * PLAYERS_DISTANCE;

        // move cards from a deck to a player's hand
        let mut player_hand: Vec<Card> = new_deck.drain(..HAND_SIZE).collect();

        // If not a MainPlayer
        if i != 0
        {
            for (j, card) in player_hand.iter_mut().enumerate()
            {
                commands.spawn((
                    CardBundle {
                        sprite: SpriteBundle {
                            texture: tex_back.clone(),
                            transform: Transform::from_xyz(x + (j as f32) * ENEMY_CARDS_SPACING, y, j as f32).with_scale(ENEMY_CARD_SCALE),
                            ..default()
                        },
                        id: Id(card.id),
                    },
                    GameItem
                ));
            }
            // Spawn a player and give him a name from enum of PlayerName
            commands.spawn((PlayerName::from_usize(i).unwrap(),
                Player { pos: Vec3::new(x, y, 0.0), cards: player_hand },
                GameItem,
            ));
        }
        else
        {
            for (j, card) in player_hand.iter_mut().enumerate()
            {
                // If MainPlayer's hand - load a front image instead of a back image
                let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());

                commands.spawn((
                    CardBundle {
                        sprite: SpriteBundle {
                            texture: asset_server.load(image_name),
                            transform: Transform::from_xyz(x + (j as f32) * PLAYER_CARDS_SPACING, y, j as f32).with_scale(PLAYER_CARD_SCALE),
                            ..default()
                        },
                        id: Id(card.id),
                    },
                    GameItem
                ));

            }
            // Spawn a player and give him a MainPlayer component to access him directly without
            // quering every player in a game and filtering a MainPlayer
            commands.spawn((PlayerName::MainPlayer,
                Player { pos: Vec3::new(x, y, 0.0), cards: player_hand },
                MainPlayer,
                GameItem,
            ));
        }

        // Text of a Player's name on top of a hand
        commands.spawn((
            Text2dBundle {
                text: Text::from_section(format!("Player {}", i + 1), TextStyle { font: font.clone(), font_size: 50.0, color: NAME_TEXT_COLOR }),
                transform: Transform::from_xyz(x - NAME_TEXT_OFFSET_X, y - NAME_TEXT_OFFSET_Y, 0.0),
                ..default()
            },
            GameItem,
        ));

    }

    /************ Create discard pile *************/

    // TODO threow back to menu if not enough cards in deck to deal for everyone
    // Put a card from a deck to a discard pile
    let pile_top_card = new_deck.pop().unwrap();
    let image_name = format!("{}_{}.png", pile_top_card.suite.to_string(), pile_top_card.rank.to_string());
    let discard_pile = vec![pile_top_card];
    commands.spawn((
        DiscardPile { cards: discard_pile },
        SpriteBundle {
            texture: asset_server.load(image_name),
            transform: Transform::from_xyz(DECK_DISCARD_DISTANCE, 0.0, 0.0).with_scale(DISCARD_CARD_SCALE),
            ..default()
        },
        GameItem,
    ));

    /************ Create a deck *************/

    // Spawn a deck and put unused cards there
    commands.spawn((
        Deck { cards: new_deck },
        SpriteBundle {
            texture: tex_back,
            transform: Transform::from_xyz(-DECK_DISCARD_DISTANCE, 0.0, 0.0).with_scale(DECK_CARD_SCALE),
            ..default()
        },
        GameItem,
    ));
}

// TODO very time we check the bounds of cards in a player's hand, we query all the images and
// filter them by id of a card in player's hand. Need to store a size as a resource after first
// check, so that subsequent click will not need to calculate it again, but instad read image size
fn check_deck_bounds(
    window_q: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    deck_q: Query<&Handle<Image>, With<Deck>>,
    card_q: Query<(&Handle<Image>, &Id)>,
    player_q: Query<&Player, With<MainPlayer>>,
    all_images: Res<Assets<Image>>,
    mut deck_event: EventWriter<DrawCard>,
    mut card_event: EventWriter<PlayCard>,
    gameplay_rules: Res<GameplayState>,
) {
    if gameplay_rules.player_turn != PlayerName::MainPlayer { return; }

    let window = window_q.single();
    let (camera, camera_pos) = camera_q.single();

    // First make sure that click was inside a window.
    // If yes - transform cursor's position from global position to 2D world position
    if let Some(cursor_pos) = window.cursor_position().and_then(|cursor| camera.viewport_to_world_2d(camera_pos, cursor))
    {
        /********* Check if clicked on a deck *********/

        let deck_image = deck_q.single();
        // If deck's image was properly loaded - find it using a deck's image handle
        // from all images in a world. Otherwise use user defined size
        let deck_image_size: Vec2 = if let Some(image) = all_images.get(&deck_image) { image.size() } else { FALLBACK_DECK_COLLIDER };
        // Bounds for deck image
        let x_offset = deck_image_size.x * DECK_CARD_SCALE.x / 2.0;
        let y_offset = deck_image_size.y * DECK_CARD_SCALE.y / 2.0;

        //          +y_offset
        //           ___ 
        //          |   |
        //-x_offset | · | +x_offset
        //          |___|
        //          -y_offset
        if cursor_pos.x > -DECK_DISCARD_DISTANCE - x_offset &&
            cursor_pos.x < -DECK_DISCARD_DISTANCE + x_offset &&
            cursor_pos.y > -y_offset &&
            cursor_pos.y < y_offset
        {
                deck_event.send_default();
                return;
        }

        /*********Check if clicked on your hand*********/

        let player = player_q.single();
        // Detrmin how many cards are in a hand to get rightmost bound
        let counter: usize = player.cards.len();

        // Get the size of a card to determine a collider
        let mut card_image_size: Vec2 = Vec2::ZERO;
        for (card_image, id) in card_q.iter()
        {
            if id.0 == player.cards[0].id
            {
                // TODO prevent loading game dureing setup if image couldn't be loaded
                card_image_size = if let Some(image) = all_images.get(&card_image) { image.size() } else { FALLBACK_DECK_COLLIDER };
                break;
            }
        }
        // Bounds for deck image
        let card_x_offset: f32 = card_image_size.x * PLAYER_CARD_SCALE.x / 2.0;
        let card_y_offset: f32 = card_image_size.y * PLAYER_CARD_SCALE.y / 2.0;
        let left_edge = player.pos.x - card_x_offset;

        if cursor_pos.x > left_edge &&
            cursor_pos.x < player.pos.x + card_x_offset + PLAYER_CARDS_SPACING * (counter - 1) as f32 &&
            cursor_pos.y > player.pos.y - card_y_offset &&
            cursor_pos.y < player.pos.y + card_y_offset
        {
                // Send an index of acard that was clicked on
                let num_card = get_card_index(player.pos.x - card_x_offset, cursor_pos.x, counter);
                card_event.send(PlayCard(num_card));
        }
    }
}

fn draw_card(
    mut commands: Commands,
    mut player_q: Query<&mut Player, With<MainPlayer>>,
    mut deck_q: Query<&mut Deck>,
    asset_server: Res<AssetServer>,
    mut event: EventReader<DrawCard>,
) {
    // Continue only on receiving an event
    for _ in event.iter()
    {
        let mut deck = deck_q.single_mut();

        // Check whether there are anymore cards in deck
        if let Some(card) = deck.cards.pop()
        {
            let mut player = player_q.single_mut();
            // X = position of last card + spacing
            let num_cards = player.cards.len();
            // Load and spawn an image with a new new card
            let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());

            commands.spawn((
                CardBundle {
                    sprite: SpriteBundle {
                        texture: asset_server.load(image_name),
                        transform: Transform::from_xyz(player.pos.x + PLAYER_CARDS_SPACING * num_cards as f32, player.pos.y, num_cards as f32).with_scale(PLAYER_CARD_SCALE),
                        ..default()
                    },
                    id: Id(card.id),
                },
                GameItem,
            ));

            player.cards.push(card);
        }
        else
        {
            warn!("No cards lft inside a deck.");
        }
    }
}

fn play_card(
    mut commands: Commands,
    mut play_event: EventReader<PlayCard>,
    mut discard_q: Query<(&mut Handle<Image>, &mut DiscardPile), Without<Deck>>,
    mut player_q: Query<&mut Player, With<MainPlayer>>,
    mut cards_image_q: Query<(Entity, &Handle<Image>, &mut Transform, &Id), (Without<DiscardPile>, Without<Deck>)>,
    mut gameplay: ResMut<GameplayState>,
    rules: Res<Rules>,
) {
    for event in play_event.iter()
    {
        let (mut discard_image, mut pile) = discard_q.single_mut();
        let mut player = player_q.single_mut();

        if pile.cards.last().unwrap().rank != player.cards[event.0].rank &&
            pile.cards.last().unwrap().suite != player.cards[event.0].suite { return; }

        let card = player.cards.remove(event.0);
        let cards_len = player.cards.len();

        // Store Ids of cards that are to the right of a card that was played, so that we can find
        // sprites of those cards by Id and change their positions accordingly
        let mut ids = vec![];
        for i in event.0..cards_len
        {
            ids.push(player.cards[i].id);
        }
        
        for (entity, image, mut pos, id) in cards_image_q.iter_mut()
        {
            // Match Ids of all sprites in game. If it's the one of a card that was player -> despawn it
            // If it's a card that is to the right of a removed card -> move it to left and down by 1
            match id.0
            {
                id if id == card.id => {
                    *discard_image = image.clone();
                    commands.entity(entity).despawn();
                },
                _ if ids.contains(&id.0) => {
                    pos.translation.x -= PLAYER_CARDS_SPACING;
                    pos.translation.z -= 1.0;
                },
                _ => (),
            }
        }

        // Add a played card to discard pile
        pile.cards.push(card);
        
        // In case only 1 player in a lobby, don't pass a turn
        // TODO stackable cards logic
        if rules.num_players == 1 || rules.stackable_cards { return; }

        if !rules.clockwise
        {
            gameplay.player_turn = PlayerName::from_usize(1).unwrap();
        }
        else
        {
            gameplay.player_turn = PlayerName::from_usize(rules.num_players - 1).unwrap();
        }
    }
}

fn bot_play(
    mut commands: Commands,
    mut discard_q: Query<(&mut Handle<Image>, &mut DiscardPile)>,
    mut players_q: Query<(&mut Player, &PlayerName), Without<MainPlayer>>,
    cards_image_q: Query<(Entity, &Id), (Without<DiscardPile>, Without<Deck>)>,
    time: Res<Time>,
    rules: Res<Rules>,
    mut state: ResMut<BotWaiting>,
    mut game_rules: ResMut<GameplayState>,
    asset_server: Res<AssetServer>,
) {
    if game_rules.player_turn == PlayerName::MainPlayer { return; }

    if state.event_timer.tick(time.delta()).finished()
    {
        let (mut discard_image, mut pile) = discard_q.single_mut();
        'players: for (mut player, name) in players_q.iter_mut()
        {
            if game_rules.player_turn != *name { continue 'players; }

            let card = player.cards.pop().unwrap();
            let image_name = format!("{}_{}.png", card.suite.to_string(), card.rank.to_string());

            'cards: for (entity, id) in &cards_image_q
            {
                if card.id != id.0 { continue 'cards; }

                *discard_image = asset_server.load(image_name);
                commands.entity(entity).despawn();
                pile.cards.push(card);
                break 'players;
            }
        }

        let index = game_rules.player_turn as usize;
        let index = if rules.clockwise { index - 1 } else { (index + 1) % rules.num_players };
        game_rules.player_turn = PlayerName::from_usize(index).unwrap();
    }
}

fn menu(
    mut next_state: ResMut<NextState<GameState>>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Escape)
    {
        next_state.set(GameState::Menu);
    }
}

fn mouse_pressed(mouse_button_input: Res<Input<MouseButton>>) -> bool
{
    mouse_button_input.just_pressed(MouseButton::Left)
}

fn test(
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut game_rules: ResMut<GameplayState>,
    mut player_q: Query<&mut Player, With<MainPlayer>>,
    discard_pile_q: Query<&DiscardPile>,
    deck_q: Query<&Deck>,
    id_q: Query<&Id>,
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
        info!("{:?}", game_rules.player_turn);
    }

    if key.just_pressed(KeyCode::R) {
        let deck = deck_q.single();
        let num_cards = deck.cards.len();
        info!("Cards in deck left: {}", num_cards);
    }


    if key.just_pressed(KeyCode::Space) {
        let pile = discard_pile_q.single();
        if let Some(last_card) = pile.cards.last()
        {
            info!("{:?}", last_card);
        }
    }
}

//----------------------------------------------------------------------------------
//  Helper functions
//----------------------------------------------------------------------------------

fn get_card_index(left: f32, x: f32, counter: usize) -> usize {
    for i in 0..counter {
        if x < left + PLAYER_CARDS_SPACING * (i + 1) as f32 {
            return i;
        }
    }
    counter - 1
}

// Allows to format an enum into string
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

impl Default for BotWaiting {
    fn default() -> Self {
        BotWaiting {
            event_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        }
    }
}

