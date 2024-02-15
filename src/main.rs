use bevy::prelude::*;

mod block;
mod camera;

use block::BlockPlugin;
use camera::MainCameraPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_plugins((BlockPlugin, MainCameraPlugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(16.0, 8.0, 8.0),
        ..default()
    });
}
