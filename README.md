# Dos-game
Blazingly fast Uno game with a customizable ruleset

## Setup

### To run locally in a new window
This project uses a [mold](https://github.com/rui314/mold) linker, so you will need to download it. Otherwise just remove a linker from .cargo/config.toml.

1. Clone repo
2. `cargo run'

### To run in a web browser
``` rustup target install wasm32-unknown-unknown
    cargo install basic-http-server     # or any other tool to start a local server
    basic-http-server out
```

Or to make a web build yourself
``` rustup target install wasm32-unknown-unknown
    cargo install wasm-bindgen-cli
    wasm-bindgen --out-name dos_example --out-dir ./out --target web ./target/wasm32-unknown-unknown/release/my_game.wasm
```
You will have a build that you can host on a website

## Entities

Player Bundle
∟PlayerName
∟Player

Text

Card Bundle
∟Card
∟PlayerName
∟SpriteBundle
 ∟...

Discarded Cards
∟DiscardPile
∟CardBundle
 ∟Card
 ∟PlayerName
 ∟SpriteBundle
  ∟...

Deck Bundle
∟Deck
∟SpriteBundle
 ∟...

Unless card is inside a deck, it has a position

## Components

```
#[derive(Component, ...]
struct Card {
    rank: Rank,
    suite: Suit,
    pos: Option<Vec3>,
}

#[derive(Component, ...]
struct Deck {
    cards: Vec<Card>,
}

#[derive(Component, ...]
struct DiscardPile {
    cards: Vec<Card>,
}

#[derive(Component, ...]
struct Player {
    pos: Vec3,
    cards: Vec<Card>,
}

#[derive(Component, ...]
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

```
