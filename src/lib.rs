#![feature(variant_count)]
mod menu;
mod game;

use crate::menu::MenuPlugin;
use crate::game::GamePlugin;

use bevy::prelude::*;
use bevy::app::App;

pub const LOBBY_PLAYERS: usize = 5;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Settings
{
    players: u8,
    stackable_cards: bool,
}

pub struct MainPlugin;

impl Plugin for MainPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_state::<GameState>()
            .insert_resource(Settings {
                players: 5,
                stackable_cards: false,
            })
            .add_startup_system(setup)
            .add_plugin(MenuPlugin)
            .add_plugin(GamePlugin);
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

