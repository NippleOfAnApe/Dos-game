use bevy::prelude::*;
use num::FromPrimitive;         //for accessing enum values via integer
use num_derive::FromPrimitive;  //to derive a trait on enum to access it with integer

const LOBBY_PLAYERS: u32 = 5;
const DECK_SIZE: u32 = 20;
const HAND_SIZE: u32 = 7;
const PLAYERS_DISTANCE: f32 = 250.;
const CARDS_SCALE: f32 = 0.3;

#[derive(Component, Debug)]
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

#[derive(Component, Debug)]
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
}

#[derive(Resource)]
struct GameRules {
    start_hand_size: u32,
    deck_size: u32,
    max_players: u32,
}

#[derive(Component)]
struct Card {
    rank: Rank,
    suite: Suit
}

#[derive(Bundle)]
struct PlayerCard {
    card: Card,

    #[bundle]
    tex: SpriteBundle,
}

#[derive(Component)]
struct Deck;

#[derive(Component, Debug)]
struct Player(PlayerName);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.3)))
        .insert_resource(GameRules {
            start_hand_size: HAND_SIZE,
            deck_size: DECK_SIZE,
            max_players: LOBBY_PLAYERS })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(populate_deck)
        .add_system(test)
        .add_system(give_starting_hand.run_if(start_new_game))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    //Camera
    commands.spawn(Camera2dBundle::default());

    let center = Vec3::ZERO;
    let angle: f32 = 360.0 / LOBBY_PLAYERS as f32;

    for i in 0..LOBBY_PLAYERS {
        let theta = (-90. + i as f32 * angle).to_radians();
        let x = center.x + theta.cos() * PLAYERS_DISTANCE;
        let y = center.y + theta.sin() * PLAYERS_DISTANCE;
        // info!("X: {:?}\tY: {:?}\tAngle: {:?}\tIter: {:?}", x, y, theta, i as f32);
        // if i == 0 {
        //     commands.spawn((Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(CARDS_SCALE)), MainPlayer, Player)); }
        // else {
        //     commands.spawn((Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(CARDS_SCALE)), Player)); }

        commands.spawn((Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(CARDS_SCALE)), Player(PlayerName::from_u32(i).unwrap())));
        
    }
}

fn populate_deck(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rules: Res<GameRules>,
) {
    let tex = asset_server.load("back.png");
    commands.spawn((SpriteBundle {
        transform: Transform::IDENTITY.with_scale(Vec3::splat(CARDS_SCALE)),
        texture: tex.clone(),
        ..default()
    }, Deck));

    let mut cards = vec![];
    for i in 0..rules.deck_size {
        cards.push(Card {
            rank: Rank::Zero,
            suite: Suit::Red
        });
    }

    commands.spawn_batch(cards);
}

fn give_starting_hand(
    asset_server: Res<AssetServer>,
    mut cards_q: Query<&mut Card>,
    players_q: Query<(&Player, &Transform)>,
    mut commands: Commands,
) {
    let tex_back = asset_server.load("back.png");
    let tex_front = asset_server.load("4_red.png");

    for (player, position) in players_q.iter() {
        if player.0 == PlayerName::MainPlayer {
            commands.spawn(PlayerCard {
                tex: SpriteBundle {
                    texture: tex_front.clone(),
                    transform: *position,
                    ..default()
                },
                card: Card {
                    rank: Rank::Zero,
                    suite: Suit::Red,
                }
            });
        } else {
            commands.spawn(PlayerCard {
                tex: SpriteBundle {
                    texture: tex_back.clone(),
                    transform: *position,
                    ..default()
                },
                card: Card {
                    rank: Rank::Zero,
                    suite: Suit::Red,
                }
            });
        }
    }
}

fn start_new_game(
    keyboard_input: Res<Input<KeyCode>>,
) -> bool {
    keyboard_input.just_pressed(KeyCode::Return)
}

fn test(
    deck_query: Query<&Card>,
    players_q: Query<&Player>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        for card in &deck_query {
            info!("Card rank: {:?}\tCard suite: {:?}", card.rank, card.suite);
        }

        let mut counter: u32 = 0;
        for _ in &players_q {
            counter += 1;
        }
        info!("players: {:?}, Player name: {:?}", counter, PlayerName::from_u8(2).unwrap());
    }
}
