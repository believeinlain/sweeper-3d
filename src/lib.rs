use bevy::prelude::*;

mod game;
mod input;
mod loader;
mod menu;
mod settings;

pub use input::InputEvent;
pub use loader::GameAssets;
pub use settings::{FieldSettings, GameSettings, Safety};

pub use game::GamePlugin;
pub use input::InputPlugin;
pub use loader::LoaderPlugin;
pub use menu::MenuPlugin;
pub use settings::SettingsPlugin;

#[derive(Debug, Component)]
pub struct MaterialHandle(Handle<StandardMaterial>);

#[derive(Debug, Component)]
pub struct MeshHandle(Handle<Mesh>);

#[derive(Debug, Component)]
pub struct AudioHandle(Handle<AudioSource>);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    /// Loading
    #[default]
    Loading,
    /// Main menu
    MenuMain,
    /// Custom game menu
    MenuCustom,
    /// Settings menu
    MenuSettings,
    /// Game has started, but no cell has been clicked yet.
    GameStart,
    /// Game transitions to this state once the first cell is clicked.
    /// This is when the field actually initializes and determines mine placement.
    /// At this point the position of all mines is known.
    GamePlaying,
    /// Game has ended, either by clicking on a mine or by clearing all non-mines.
    GameOver,
}
impl GameState {
    /// Any in-game state. [`GameState::GameStart`] || [`GameState::GamePlaying`] || [`GameState::GameOver`].
    pub fn in_game() -> impl SystemCondition<()> {
        in_state(Self::GameStart)
            .or(in_state(Self::GamePlaying))
            .or(in_state(Self::GameOver))
    }
    /// Whether moves are allowed. [`GameState::GameStart`] || [`GameState::GamePlaying`].
    pub fn playable() -> impl SystemCondition<()> {
        in_state(Self::GameStart).or(in_state(Self::GamePlaying))
    }
}
