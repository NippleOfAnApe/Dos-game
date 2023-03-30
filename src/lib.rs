mod menu;
mod game;

use crate::menu::MenuPlugin;
use crate::game::GamePlugin;

use bevy::prelude::*;
use bevy::app::App;

pub const LOBBY_PLAYERS: u32 = 5;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Menu,
    InGame,
}

pub struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_startup_system(setup)
            .add_plugin(MenuPlugin)
            .add_plugin(GamePlugin);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component, F: Component>(to_despawn: Query<Entity, (With<T>, Without<F>)>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

