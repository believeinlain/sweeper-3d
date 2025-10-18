use std::fmt::Display;

use bevy::prelude::*;
use ndarray::prelude::*;
use rand::prelude::*;

use super::{
    GamePiece, GameResult, GameState,
    block::{Block, BlockEvent},
};
use crate::{FieldSettings, GameSettings, Safety};

pub struct FieldPlugin;
impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        // Add Minefield systems
        app.add_systems(OnEnter(GameState::GameStart), spawn.after(super::cleanup));
        app.add_systems(
            Update,
            handle_field_events
                .after(super::block::handle_ray_events)
                .run_if(GameState::playable()),
        );
        app.add_systems(OnEnter(GameState::GameOver), reveal_all);
        app.add_event::<FieldEvent>();
    }
}

#[derive(Message)]
pub enum FieldEvent {
    SpawnBlock(Entity, [usize; 3]),
    ClearBlock([usize; 3]),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
    contains: Contains,
    revealed: bool,
    block: Option<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub enum Contains {
    Mine,
    Empty { adjacent_mines: u8 },
}
impl Default for Contains {
    fn default() -> Self {
        Self::Empty { adjacent_mines: 0 }
    }
}

#[derive(Debug, Deref, Clone, Copy, PartialEq, Eq)]
pub struct FieldIndex((usize, usize, usize));
impl From<&(usize, usize, usize)> for FieldIndex {
    #[inline]
    fn from((i, j, k): &(usize, usize, usize)) -> Self {
        Self((*i, *j, *k))
    }
}
impl From<(usize, usize, usize)> for FieldIndex {
    #[inline]
    fn from(value: (usize, usize, usize)) -> Self {
        Self(value)
    }
}
impl From<[usize; 3]> for FieldIndex {
    #[inline]
    fn from(value: [usize; 3]) -> Self {
        Self((value[0], value[1], value[2]))
    }
}
impl From<&[usize; 3]> for FieldIndex {
    #[inline]
    fn from(value: &[usize; 3]) -> Self {
        Self((value[0], value[1], value[2]))
    }
}
impl Display for FieldIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FieldIndex((i, j, k)) = self;
        write!(f, "({i}, {j}, {k})")
    }
}

#[derive(Component)]
pub struct Minefield {
    cells: Array3<Cell>,
    density: f64,
    safety: Safety,
}
impl Minefield {
    /// Initialize the [Minefield], placing mines randomly according to [Minefield::density].
    fn initialize(&mut self, blocks: &Query<(Entity, &Block)>, click_location: FieldIndex) {
        // Save Block ids
        for (entity, block) in blocks {
            self.cells[block.index()].block = Some(entity)
        }
        log::info!("Creating minefield");
        let mut rng = rand::thread_rng();
        let num_blocks = self.cells.iter().count();
        let num_mines = (num_blocks as f64 * self.density) as usize;
        log::debug!(
            "Density {} => num_mines = {}/{}",
            self.density,
            num_mines,
            num_blocks
        );
        // Determine safe cells based on safety and click location
        let safe_cells = match self.safety {
            Safety::Random => vec![],
            Safety::Safe => vec![click_location],
            Safety::Clear => {
                let mut safe = vec![click_location];
                self.foreach_adjacent(click_location, |adj_index| {
                    safe.push(adj_index);
                });
                safe
            }
        };
        // Sort remaining potential mine locations in random order
        let mut random_cells: Vec<_> = self
            .cells
            .indexed_iter()
            .map(|(i, _)| i.into())
            .filter(|i| {
                let safe = safe_cells.contains(i);
                if safe {
                    log::debug!("Ignoring safe cell {i}");
                }
                !safe
            })
            .collect();
        random_cells.shuffle(&mut rng);
        // Place mines
        let mut mines_to_place = num_mines;
        let num_cells = random_cells.len();
        // Place mines randomly until there is exactly as many cells left as mines,
        // then just fill the rest.
        // Because we're iterating in random order, this will be fine.
        // Not guaranteed to place exactly num_mines mines; we prioritize the safety setting.
        for (n, index) in random_cells.into_iter().enumerate() {
            if mines_to_place == 0 {
                break;
            }
            let cells_remaining = num_cells - n;
            if cells_remaining <= mines_to_place || rng.gen_bool(self.density) {
                self.cells[*index].contains = Contains::Mine;
                mines_to_place -= 1;
            }
        }
        // Determine adjacent value in each cell
        let mines: Vec<_> = self
            .cells
            .indexed_iter()
            .filter_map(|(i, c)| match c.contains {
                Contains::Mine => Some(i),
                _ => None,
            })
            .collect();
        for index in mines {
            log::debug!("Placed mine at {index:?}");
            let (i, j, k) = index;
            let mut increment_adjacent = |i_off, j_off, k_off| {
                let adj_index = (
                    i.wrapping_add_signed(i_off),
                    j.wrapping_add_signed(j_off),
                    k.wrapping_add_signed(k_off),
                );
                if let Some(adj) = self.cells.get_mut(adj_index) {
                    if let Contains::Empty {
                        ref mut adjacent_mines,
                    } = adj.contains
                    {
                        log::debug!("Increment adjacent at {adj_index:?}");
                        *adjacent_mines += 1;
                    }
                }
            };
            for i_off in -1..=1 {
                for j_off in -1..=1 {
                    for k_off in -1..=1 {
                        // The block at index is not adjacent to itself
                        if i_off == 0 && j_off == 0 && k_off == 0 {
                            continue;
                        }
                        increment_adjacent(i_off, j_off, k_off);
                    }
                }
            }
        }
    }
    fn foreach_adjacent<F>(&self, index: impl Into<FieldIndex>, mut f: F)
    where
        F: FnMut(FieldIndex),
    {
        let (i, j, k) = *index.into();
        for i_off in -1..=1 {
            for j_off in -1..=1 {
                for k_off in -1..=1 {
                    // The block at index is not adjacent to itself
                    if i_off == 0 && j_off == 0 && k_off == 0 {
                        continue;
                    }
                    // Get a block adjacent to index
                    let adj_index = (
                        i.wrapping_add_signed(i_off),
                        j.wrapping_add_signed(j_off),
                        k.wrapping_add_signed(k_off),
                    );
                    // Make sure we have a valid adj_index
                    if self.cells.get(adj_index).is_none() {
                        continue;
                    };
                    f(adj_index.into());
                }
            }
        }
    }
    fn reveal_adjacent(
        &mut self,
        index: (usize, usize, usize),
        block_events: &mut MessageWriter<BlockEvent>,
    ) {
        let (i, j, k) = index;
        for i_off in -1..=1 {
            for j_off in -1..=1 {
                for k_off in -1..=1 {
                    // The block at index is not adjacent to itself
                    if i_off == 0 && j_off == 0 && k_off == 0 {
                        continue;
                    }
                    // Get a block adjacent to index
                    let adj_index = (
                        i.wrapping_add_signed(i_off),
                        j.wrapping_add_signed(j_off),
                        k.wrapping_add_signed(k_off),
                    );
                    // Make sure we have a valid adj_index
                    let Some(adj) = self.cells.get_mut(adj_index) else {
                        continue;
                    };
                    // If the adjacent block is already revealed, don't bother
                    // TODO: maybe not good idea?
                    if adj.revealed {
                        continue;
                    }
                    let contains = adj.contains;
                    // Don't reveal mines
                    let Contains::Empty { adjacent_mines } = contains else {
                        continue;
                    };
                    // Get the entity to send with the message
                    let Some(adj_id) = adj.block else {
                        continue;
                    };
                    adj.revealed = true;
                    // Send a message to reveal this block
                    let event = BlockEvent::Clear(adj_id, contains);
                    log::debug!("Send {event:?}");
                    block_events.write(event);
                    // Recurse only if this block was not adjacent to any mines
                    if adjacent_mines == 0 {
                        self.reveal_adjacent(adj_index, block_events);
                    }
                }
            }
        }
    }
    /// Return true iff the Minefield has been fully revealed (victory condition)
    fn fully_revealed(&self) -> bool {
        for cell in &self.cells {
            if !cell.revealed && !matches!(cell.contains, Contains::Mine) {
                return false;
            }
        }
        true
    }
}

fn spawn(
    game_settings: Res<GameSettings>,
    field_settings: Res<FieldSettings>,
    mut commands: Commands,
) {
    let field = Minefield {
        cells: Array3::default(field_settings.field_size),
        density: field_settings.mine_density.into(),
        safety: game_settings.safety,
    };
    commands.spawn((field, GamePiece));
}

pub(super) fn handle_field_events(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_result: ResMut<GameResult>,
    blocks: Query<(Entity, &Block)>,
    mut field: Query<&mut Minefield>,
    mut field_events: MessageReader<FieldEvent>,
    mut block_events: MessageWriter<BlockEvent>,
) {
    for event in field_events.read() {
        match event {
            FieldEvent::SpawnBlock(entity, index) => {
                let mut field = field.single_mut().unwrap();
                let Some(cell) = field.cells.get_mut(*index) else {
                    continue;
                };
                cell.block = Some(*entity);
            }
            FieldEvent::ClearBlock(index) => {
                let mut field = field.single_mut().unwrap();
                let Some(cell) = field.cells.get_mut(*index) else {
                    continue;
                };
                cell.revealed = true;
                if matches!(game_state.get(), GameState::GameStart) {
                    log::debug!("Transition to GameState::Playing");
                    next_state.set(GameState::GamePlaying);
                    field.initialize(&blocks, index.into());
                }
                // Get the updated field
                let Some(cell) = field.cells.get_mut(*index) else {
                    continue;
                };
                let contains = cell.contains;
                let event = BlockEvent::Clear(cell.block.unwrap(), contains);
                log::debug!("Send {event:?}");
                block_events.write(event);
                if matches!(contains, Contains::Empty { adjacent_mines } if adjacent_mines == 0) {
                    field.reveal_adjacent((index[0], index[1], index[2]), &mut block_events);
                }
                if field.fully_revealed() {
                    log::info!("Victory!");
                    log::debug!("Transition to GameState::Ended");
                    *game_result = GameResult::Victory;
                    next_state.set(GameState::GameOver);
                }
            }
        }
    }
}

fn reveal_all(mut field: Query<&mut Minefield>, mut block_events: MessageWriter<BlockEvent>) {
    for cell in field.single_mut().unwrap().cells.iter_mut() {
        cell.revealed = true;
        block_events.write(BlockEvent::EndReveal(cell.block.unwrap(), cell.contains));
    }
}
