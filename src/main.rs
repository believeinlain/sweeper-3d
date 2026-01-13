// Disable console window in Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
    window::WindowResolution,
};
use sweeper_3d::{GamePlugin, GameState, InputPlugin, LoaderPlugin, MenuPlugin, SettingsPlugin};

fn main() {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings.backends = Some(Backends::VULKAN);
    App::new()
        .init_state::<GameState>()
        .add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: wgpu_settings.into(),
                    ..default()
                })
                // Window settings
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Show the window only once internal startup has finished and we're running systems
                        visible: false,
                        resolution: WindowResolution::new(1024.0, 768.0),
                        title: "3D Sweeper".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                // Log settings
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_plugins((
            MenuPlugin,
            SettingsPlugin,
            GamePlugin,
            InputPlugin,
            LoaderPlugin,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 1200.0,
            shadows_enabled: true,
            color: Color::rgb(1.0, 0.95, 0.90),
            ..default()
        },
        transform: Transform::from_xyz(-1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    commands.insert_resource(AmbientLight {
        brightness: 100.0,
        color: Color::rgb(0.95, 0.95, 1.0),
    });
}
