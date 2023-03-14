use bevy::prelude::*;

const MAX_PLAYERS: u32 = 7;
const DECK_SIZE: u32 = 20;
const HAND_SIZE: u32 = 7;
const PLAYERS_DISTANCE: f32 = 250.;

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

#[derive(Resource)]
struct GameRules {
    start_hand_size: u32,
    deck_size: u32,
    max_players: u32,
}

#[derive(Component)]
struct Card {
    rank: Rank,
    suite: Suit,
    texture: Handle<Image>,
}

#[derive(Component)]
struct Deck;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainPlayer;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.3)))
        .insert_resource(GameRules {
            start_hand_size: HAND_SIZE,
            deck_size: DECK_SIZE,
            max_players: MAX_PLAYERS })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(populate_deck.after(setup))
        .add_system(test)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    //Camera
    commands.spawn(Camera2dBundle::default());

    let tex = asset_server.load("back.png");
    let center = Vec3::ZERO;
    let angle: f32 = 360.0 / MAX_PLAYERS as f32;

    for i in 0..MAX_PLAYERS {
        let theta = (-90. + i as f32 * angle).to_radians();
        let x = center.x + theta.cos() * PLAYERS_DISTANCE;
        let y = center.y + theta.sin() * PLAYERS_DISTANCE;

        info!("X: {:?}\tY: {:?}\tAngle: {:?}\tIter: {:?}", x, y, theta, i as f32);
        commands.spawn((SpriteBundle {
            transform: Transform::from_xyz(x, y, 0.).with_scale(Vec3::splat(0.3)),
            texture: tex.clone(),
            ..default()
        },
        Player,
        if i == 0 { MainPlayer; }
        ));
    }
}

fn populate_deck(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    rules: Res<GameRules>
) {
    let tex = asset_server.load("back.png");
    commands.spawn((SpriteBundle {
        transform: Transform::IDENTITY.with_scale(Vec3::splat(0.3)),
        texture: tex.clone(),
        ..default()
    }, Deck));

    let mut cards = vec![];
    for i in 0..rules.deck_size {
        cards.push(Card {
            rank: Rank::Zero,
            suite: Suit::Red,
            texture: tex.clone(),
        });
    }

    commands.spawn_batch(cards);
}

fn test(
    deck_query: Query<&Card>,
    key: Res<Input<KeyCode>>
) {
    if key.just_pressed(KeyCode::Space) {
        for card in &deck_query {
            info!("Card rank: {:?}\tCard suite: {:?}", card.rank, card.suite);
        }
    }
}
