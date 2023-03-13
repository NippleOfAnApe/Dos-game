use bevy::prelude::*;

#[derive(Resource)]
struct GameRules {
    start_hand_size: usize,
    max_players: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(GameRules {
        start_hand_size: 7,
        max_players: 4,
    });
    commands.spawn(Camera2dBundle::default());
    let tex = asset_server.load("back.png");
    commands.spawn(SpriteBundle {
        transform: Transform::from_xyz(50., -100., 0.).with_scale(Vec3::splat(0.3)),
        texture: tex.clone(),
        ..default()
    });
}
