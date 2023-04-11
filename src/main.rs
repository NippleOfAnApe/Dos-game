/****************************************************
*                                                   *
*    Терпи, терпи — терпець тебе шліфує,            *
*    Сталить твій дух — тож і терпи, терпи.         *
*    Ніхто тебе з недолі не врятує,                 *
*    Ніхто не зіб'є з власної тропи.                *
*                                                   *
*                        Василь Стус                *
*                                                   *
****************************************************/

use bevy::prelude::*;
use dos_game::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.4, 0.3)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dos".into(),
                fit_canvas_to_parent: true,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(MainPlugin)
        .run();
}

