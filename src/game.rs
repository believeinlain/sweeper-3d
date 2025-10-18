use bevy::prelude::*;

mod block;
mod camera;
mod minefield;

use block::BlockPlugin;
use camera::CameraPlugin;
use minefield::FieldPlugin;

use crate::GameState;

/// Marker component indicating an entity to be removed when the game is reset.
#[derive(Component)]
pub struct GamePiece;

/// [camera::camera_controls] consumes [crate::InputEvent] and produces [RayEvent].  
/// [block::handle_ray_events] consumes [RayEvent] and produces [FieldEvent] and/or [BlockEvent].  
/// [minefield::handle_field_events] consumes [FieldEvent] and produces [BlockEvent].  
/// [block::handle_block_events] consumes [BlockEvent] and potentially changes [GameState].  
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameResult>();
        app.add_systems(OnEnter(GameState::GameStart), cleanup);
        app.add_plugins((BlockPlugin, CameraPlugin, FieldPlugin));
    }
}

pub fn cleanup(
    to_despawn: Query<Entity, With<GamePiece>>,
    mut commands: Commands,
    mut result: ResMut<GameResult>,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
    *result = GameResult::default();
}

/// When the game ends, what was the result?
#[derive(Default, Resource)]
pub enum GameResult {
    #[default]
    Unfinished,
    Victory,
    Failure,
}

// TAB - step away ______ -> asfgrdsgg
