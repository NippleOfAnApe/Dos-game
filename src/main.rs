use bevy::prelude::*;
use dos_game::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.3)))
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
        .add_plugin(MainPlugin)
        .run();
}

