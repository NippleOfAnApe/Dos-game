#![feature(variant_count)]
mod menu;
mod game;

use crate::menu::MenuPlugin;
use crate::game::GamePlugin;

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
    Low,
    Medium,
    High,
}

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct StackableCards(bool);

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct NumberPlayers(usize);

#[derive(Resource, Debug, Component, PartialEq, Eq, Clone, Copy)]
struct Rules
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
            .insert_resource(Rules {
                players: 5,
                stackable_cards: false,
            })
            .insert_resource(DisplayQuality::Medium)
            .insert_resource(StackableCards(false))
            .insert_resource(NumberPlayers(5))
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

