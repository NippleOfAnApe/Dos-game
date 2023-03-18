use bevy::prelude::*;
use rand::{seq::SliceRandom, thread_rng};
use num::FromPrimitive;         //for accessing enum values via integer
use num_derive::FromPrimitive;  //to derive a trait on enum to access it with integer

const LOBBY_PLAYERS: u32 = 5;
const DECK_SIZE: u32 = 70;
const HAND_SIZE: u32 = 7;
const PLAYERS_DISTANCE: f32 = 350.;
const CARDS_SCALE: f32 = 0.3;
const CARD_PLAYER_SPACING: f32 = 40.0;
const CARD_ENEMY_SPACING: f32 = 20.0;

#[derive(Component, Debug, FromPrimitive)]
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

#[derive(Component, Debug, FromPrimitive)]
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

#[derive(Component, Debug)]
struct Card {
    rank: Rank,
    suite: Suit,
    //owner: Option<PlayerName>,
}

#[derive(Component)]
struct Deck {
    cards: Vec<Card>,
    discard_pile: Vec<Card>,
}

#[derive(Component)]
struct Player {
    name: PlayerName,
    cards: Vec<Card>,
}

#[derive(Bundle)]
struct PlayerCardBundle {
    card: Card,

    #[bundle]
    tex: SpriteBundle,
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    pos: Transform,
}

#[derive(Bundle)]
struct DeckBundle {
    deck: Deck,
    #[bundle]
    sprite: SpriteBundle,
}

#[derive(Resource)]
struct GameRules {
    start_hand_size: u32,
    deck_size: u32,
    max_players: u32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.3)))
        .insert_resource(GameRules {
            start_hand_size: HAND_SIZE,
            deck_size: DECK_SIZE,
            max_players: LOBBY_PLAYERS })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dos".into(),
                // resolution: (500., 300.).into(),
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(setup)
        .add_startup_system(create_deck.in_base_set(StartupSet::PreStartup))
        .add_startup_system(populate_deck.in_base_set(StartupSet::Startup))
        .add_startup_system(assign_cards_to_players.in_base_set(StartupSet::PostStartup))
        .add_system(render_cards.run_if(start_new_game))
        .add_system(test)
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
    let font = asset_server.load("FiraSans-Bold.ttf");

    for i in 0..LOBBY_PLAYERS {
        let theta = (-90. + i as f32 * angle).to_radians();
        let x = center.x + theta.cos() * PLAYERS_DISTANCE;
        let y = center.y + theta.sin() * PLAYERS_DISTANCE;
        // info!("X: {:?}\tY: {:?}\tAngle: {:?}\tIter: {:?}", x, y, theta, i as f32);
        // if i == 0 {
        //     commands.spawn((Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(CARDS_SCALE)), MainPlayer, Player)); }
        // else {
        //     commands.spawn((Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(CARDS_SCALE)), Player)); }

        commands.spawn(PlayerBundle {
            pos: Transform::from_xyz(x, y, 0.0),
            // pos: Transform::from_xyz(x, y, 0.0).with_scale(Vec3::splat(CARDS_SCALE)),
            player: Player {
                name: PlayerName::from_u32(i).unwrap(),
                cards: Vec::new(),
            }
        });
        commands.spawn(Text2dBundle {
            text: Text::from_section(format!("Player {}", i + 1), TextStyle { font: font.clone(), font_size: 50.0, color: Color::WHITE }),
            transform: Transform::from_xyz(x, y - 130.0, 0.0),
            ..default()
        });
    }
}

fn create_deck(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let tex = asset_server.load("back.png");

    commands.spawn(DeckBundle {
        deck: Deck {
            cards: Vec::new(),
            discard_pile: Vec::new()
        },
        sprite: SpriteBundle {
            texture: tex.clone(),
            transform: Transform::IDENTITY.with_scale(Vec3::splat(CARDS_SCALE)),
            ..default()
        }
    });
}

fn populate_deck(
    mut deck_q: Query<&mut Deck>,
) {
    //let card_variants = mem::variant_count::<Rank>();

    let mut deck = deck_q.single_mut();
    for color in 0..4 {
        for rank in 0..13 {
            deck.cards.push(Card {
                rank: Rank::from_u32(rank).unwrap(),
                suite: Suit::from_u32(color).unwrap(),
            });
        }
    }
    deck.cards.shuffle(&mut thread_rng());
}

fn assign_cards_to_players(
    mut deck_q: Query<&mut Deck>,
    mut players_q: Query<&mut Player>,
) {
    let mut deck = deck_q.single_mut();

    for mut player in players_q.iter_mut() {
        for i in 0..HAND_SIZE {
            let card = deck.cards.remove(i as usize);
            player.cards.push(card);
        }
    }

    let discarded_card = deck.cards.remove(0);
    deck.discard_pile.push(discarded_card);
}

fn render_cards(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    players_q: Query<(&Player, &Transform)>,
    deck_q: Query<&Deck>,
) {
    info!("Starting to render.");
    let tex_back = asset_server.load("back.png");
    let tex_front = asset_server.load("4_red.png");


    for (player, pos) in players_q.iter() {
        if player.name == PlayerName::MainPlayer {
            info!("Player: {:?}", player.cards.len());
            for i in 0..player.cards.len() {
                commands.spawn(SpriteBundle {
                    texture: tex_front.clone(),
                    transform: Transform {
                        translation: Vec3::new((*pos).translation.x + (i as f32) * CARD_PLAYER_SPACING, (*pos).translation.y, i as f32),
                        scale: Vec3::splat(CARDS_SCALE),
                        ..default()
                    },
                    ..default()
                });
            }
        } else {
            for i in 0..player.cards.len() {
                commands.spawn(SpriteBundle {
                    texture: tex_back.clone(),
                    transform: Transform {
                        translation: Vec3::new((*pos).translation.x + (i as f32) * CARD_ENEMY_SPACING, (*pos).translation.y, i as f32),
                        scale: Vec3::splat(CARDS_SCALE),
                        ..default()
                    },
                    ..default()
                });
            }
        }
    }

    commands.spawn(SpriteBundle {
        texture: tex_front.clone(),
        transform: Transform::from_xyz(150.0, 0.0, 0.0).with_scale(Vec3::splat(CARDS_SCALE)),
        ..default()
    });
}

fn start_new_game(
    keyboard_input: Res<Input<KeyCode>>,
) -> bool {
    keyboard_input.just_pressed(KeyCode::Return)
}

fn test(
    deck_query: Query<&Deck>,
    players_q: Query<&Player>,
    transforms: Query<&Transform>,
    key: Res<Input<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Space) {
        let deck = deck_query.single();
        info!("# of cards in deck: {:?}", deck.cards.len());
        info!("Cards in deck{:?}", deck.cards);
        info!("Cards in discar pile{:?}", deck.discard_pile);

        let mut counter: u32 = 0;
        for _ in &players_q {
            counter += 1;
        }
        info!("players: {:?}", counter);
    }

    if key.just_pressed(KeyCode::T) {
        for positions in transforms.iter() {
            info!("Pos: {:?}", positions.translation);
        }
        // info!("Cards in discar pile{:?}", deck.discard_pile);
    }

    if key.just_pressed(KeyCode::A) {
        for player in players_q.iter() {
            if player.name == PlayerName::MainPlayer {
                info!("Player: {:?}", player.name);
                info!("Cards in hand: {:?}", player.cards);
            }
        }
    }

    if key.just_pressed(KeyCode::D) {
        for player in players_q.iter() {
            if player.name == PlayerName::Player2 {
                info!("Player: {:?}", player.name);
                info!("Cards in hand: {:?}", player.cards);
            }
        }
    }
}
