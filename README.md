# Dos-game
Blazingly fast Uno game with a customizable ruleset

## Setup

### To run locally

Clone a repo and `cargo run`. 
This project uses a [mold](https://github.com/rui314/mold) linker. If you don't have it installed just remove a linker flag from .cargo/config.toml.
To build with all the optimization run `cargo build --release` and executable will be at /target folder.

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

```
Player Bundle
∟PlayerName
∟Player
∟MainPlayer (optional)

Text    // Text with player names

Card Bundle
∟Id
∟SpriteBundle
 ∟Handle<Image>
 ∟Transform
 ∟...

Discarded Cards
∟DiscardPile
∟SpriteBundle
 ∟Handle<Image>
 ∟Transform
 ∟...

Deck Bundle
∟Deck
∟SpriteBundle
 ∟Handle<Image>
 ∟Transform
 ∟...

```
_Menu font used inside a menu is [Vividly](https://www.dafont.com/vividly.font?fpp=200) by Tata_
