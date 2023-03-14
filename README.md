# Dos-game
Blazingly fast Uno game with a customizable ruleset

## Setup

### To run locally in a new window
This project uses a [mold](https://github.com/rui314/mold) linker, so you will need to download it. Otherwise just remove a linker from .cargo/config.toml.
It also uses a nightly toolchain. If you use a standart one, just delete a rust-toolchain.toml.

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
