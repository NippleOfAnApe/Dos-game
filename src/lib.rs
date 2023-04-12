#![feature(variant_count)]
mod menu;
mod game;
mod fullscreen;
mod game_ui;

use crate::menu::MenuPlugin;
use crate::game::GamePlugin;
use crate::game_ui::GameUIPlugin;
use crate::fullscreen::FullViewportPlugin;

use bevy::prelude::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
}

// One of the two settings that can be set through the menu. It will be a resource in the app
#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
enum DisplayQuality {
    Light,
    Dark,
    Avocado,
}

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
struct Rules
{
    num_players: usize,
    stackable_cards: bool,
    turbo: bool,
    clockwise: bool,
    no_skip: bool,
}

pub struct MainPlugin;

impl Plugin for MainPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_state::<GameState>()
            .insert_resource(Rules {
                num_players: 5,
                stackable_cards: false,
                turbo: false,
                clockwise: false,
                no_skip: false,
            })
            .insert_resource(DisplayQuality::Light)
            .add_startup_system(setup)
            .add_plugin(MenuPlugin)
            .add_plugin(GamePlugin)
            .add_plugin(GameUIPlugin);

            #[cfg(target_family = "wasm")]
            app.add_plugin(FullViewportPlugin);
    }
}

fn setup(mut commands: Commands)
{
    commands.spawn(Camera2dBundle::default());
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

