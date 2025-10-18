// Disable console window in Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::window::WindowResolution;
use bevy::{prelude::*, state::app::StatesPlugin};
use sweeper_3d::{GamePlugin, GameState, InputPlugin, LoaderPlugin, MenuPlugin, SettingsPlugin};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Window settings
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Show the window only once internal startup has finished and we're running systems
                        visible: false,
                        resolution: WindowResolution::new(1024, 768),
                        title: "3D Sweeper".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            MenuPlugin,
            SettingsPlugin,
            GamePlugin,
            InputPlugin,
            LoaderPlugin,
        ))
        .add_systems(Startup, setup)
        .init_state::<GameState>()
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 1200.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.90),
            ..default()
        },
        Transform::from_xyz(-1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.insert_resource(AmbientLight {
        brightness: 100.0,
        color: Color::srgb(0.95, 0.95, 1.0),
        ..default()
    });
    commands.spawn(Camera2d);
}
