use bevy::math::bounding::{Aabb3d, Bounded3d, RayCast3d};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy::input::mouse::MouseButtonInput;

use crate::camera::MainCamera;
use crate::{Contains, FieldEvent, GameSettings, GameState};

#[derive(Component)]
pub struct Block {
    /// Whether this block has been marked as a mine.
    marked: bool,
    /// Whether this block has been revealed, and thus should
    /// show its number of adjacent mines.
    revealed: Option<Contains>,
    /// Axis-aligned bounding box for this block
    bb: Aabb3d,
    /// Field index of this block
    index: [usize; 3],
}
impl Block {
    pub fn new(bb: Aabb3d, index: [usize; 3]) -> Self {
        Self {
            marked: false,
            revealed: None,
            bb,
            index,
        }
    }
    pub fn index(&self) -> [usize; 3] {
        self.index
    }
}

pub struct BlockPlugin;
impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Start), spawn)
            .add_systems(Update, (click_on_block, handle_block_events))
            .add_event::<BlockEvent>();

        #[cfg(feature = "debug-draw")]
        app.add_systems(Update, block_gizmos);
    }
}

#[derive(Component)]
struct BlockMeshes {
    hidden: Handle<Mesh>,
    revealed_1: Handle<Mesh>,
    revealed_2: Handle<Mesh>,
    revealed_3: Handle<Mesh>,
    revealed_4: Handle<Mesh>,
    revealed_5: Handle<Mesh>,
    mine: Handle<Mesh>,
}

#[derive(Component)]
struct BlockMaterials {
    hidden: Handle<StandardMaterial>,
    marked: Handle<StandardMaterial>,
    revealed_1: Handle<StandardMaterial>,
    revealed_2: Handle<StandardMaterial>,
    revealed_3: Handle<StandardMaterial>,
    revealed_4: Handle<StandardMaterial>,
    revealed_5: Handle<StandardMaterial>,
    mine: Handle<StandardMaterial>,
}

fn calculate_position(index: [usize; 3], dim: [usize; 3]) -> Vec3 {
    Vec3::new(
        (index[0] as isize - dim[0] as isize / 2) as f32,
        (index[1] as isize - dim[1] as isize / 2) as f32,
        (index[2] as isize - dim[2] as isize / 2) as f32,
    )
}

fn spawn(
    settings: Res<GameSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let block_materials = BlockMaterials {
        hidden: materials.add(StandardMaterial::default()),
        marked: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        revealed_1: materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..default()
        }),
        revealed_2: materials.add(StandardMaterial {
            base_color: Color::GREEN,
            ..default()
        }),
        revealed_3: materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        }),
        revealed_4: materials.add(StandardMaterial {
            base_color: Color::ORANGE,
            ..default()
        }),
        revealed_5: materials.add(StandardMaterial {
            base_color: Color::PURPLE,
            ..default()
        }),
        mine: materials.add(StandardMaterial {
            base_color: Color::DARK_GRAY,
            ..default()
        }),
    };

    let cube = Cuboid::new(1.0, 1.0, 1.0);

    let block_meshes = BlockMeshes {
        hidden: meshes.add(cube),
        revealed_1: meshes.add(Sphere::new(0.1)),
        revealed_2: meshes.add(Sphere::new(0.15)),
        revealed_3: meshes.add(Sphere::new(0.2)),
        revealed_4: meshes.add(Sphere::new(0.25)),
        revealed_5: meshes.add(Sphere::new(0.275)),
        mine: meshes.add(Sphere::new(0.5)),
    };

    let mut add_cube = |index, pos| {
        let transform = Transform::from_translation(pos);
        let bb = cube.aabb_3d(transform.translation, transform.rotation);
        commands
            .spawn((
                PbrBundle {
                    mesh: block_meshes.hidden.clone(),
                    material: block_materials.hidden.clone(),
                    transform,
                    ..default()
                },
                Block::new(bb, index),
            ))
            .id()
    };

    let field_size = settings.field_size;
    for i in 0..field_size[0] {
        for j in 0..field_size[1] {
            for k in 0..field_size[2] {
                let pos = calculate_position([i, j, k], field_size);
                add_cube([i, j, k], pos);
            }
        }
    }

    // Keep the different possible meshes and materials for each block on a hidden entity
    commands.spawn((block_meshes, block_materials, Visibility::Hidden));
}

#[derive(Event)]
pub enum BlockEvent {
    /// Uncover a block, detonating any contained mines.
    Reveal(Entity, Contains),
    /// Mark a block (or unmark if already marked) as containing a mine.
    Mark(Entity),
}
impl BlockEvent {
    pub fn block_id(&self) -> Entity {
        match self {
            Self::Reveal(e, _) | Self::Mark(e) => *e,
        }
    }
}

fn click_on_block(
    mut mouse_input: EventReader<MouseButtonInput>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    main_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    blocks: Query<(Entity, &Block)>,
    mut block_events: EventWriter<BlockEvent>,
    mut field_events: EventWriter<FieldEvent>,
) {
    let Some(cursor_pos) = primary_window.single().cursor_position() else {
        return;
    };
    let (camera, camera_trans) = main_camera.single();
    for mouse_event in mouse_input.read() {
        if mouse_event.state.is_pressed() {
            debug!("Click at {cursor_pos:?}");
            let Some(ray) = super::camera::get_cursor_ray(camera, camera_trans, cursor_pos) else {
                continue;
            };
            debug!("Cursor ray at {ray:?}");
            let cast = RayCast3d::from_ray(ray, 100.0);

            let mut hits: Vec<_> = blocks
                .iter()
                .filter(|(_, block)| block.revealed.is_none())
                .filter_map(|(entity, block)| {
                    cast.aabb_intersection_at(&block.bb)
                        .map(|dist| (dist, entity, block))
                })
                .collect();
            // Consider any unresolved comparisons to be equal (i.e. NaN == NaN)
            hits.sort_unstable_by(|(a, _, _), (b, _, _)| {
                a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
            });

            let Some((dist, hit, block)) = hits.first() else {
                continue;
            };
            let index = block.index;
            debug!("Block {hit:?} {index:?} hit at {dist}");
            match mouse_event.button {
                MouseButton::Left => {
                    field_events.send(FieldEvent::Reveal(*hit, index));
                }
                MouseButton::Right => {
                    block_events.send(BlockEvent::Mark(*hit));
                }
                _ => {}
            };
        }
    }
}

fn handle_block_events(
    mut commands: Commands,
    mut block_events: EventReader<BlockEvent>,
    mut blocks: Query<&mut Block>,
    block_meshes: Query<&BlockMeshes>,
    block_materials: Query<&BlockMaterials>,
) {
    let block_meshes = block_meshes.single();
    let block_materials = block_materials.single();
    for event in block_events.read() {
        let id = event.block_id();
        let mut block = match blocks.get_mut(id) {
            Ok(block) => block,
            Err(err) => {
                error!("Unable to retrieve Block {id:?}: {err}");
                continue;
            }
        };
        match event {
            BlockEvent::Reveal(entity, contains) => {
                info!("Revealed block {entity:?}");
                block.revealed = Some(*contains);
                commands.entity(*entity).remove::<Handle<Mesh>>();
                commands
                    .entity(*entity)
                    .remove::<Handle<StandardMaterial>>();
                match contains {
                    Contains::Mine => {
                        commands
                            .entity(*entity)
                            .insert(block_meshes.mine.clone())
                            .insert(block_materials.mine.clone());
                    }
                    Contains::Empty { adjacent_mines } => match adjacent_mines {
                        0 => {}
                        1 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_1.clone())
                                .insert(block_materials.revealed_1.clone());
                        }
                        2 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_2.clone())
                                .insert(block_materials.revealed_2.clone());
                        }
                        3 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_3.clone())
                                .insert(block_materials.revealed_3.clone());
                        }
                        4 => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_4.clone())
                                .insert(block_materials.revealed_4.clone());
                        }
                        _ => {
                            commands
                                .entity(*entity)
                                .insert(block_meshes.revealed_5.clone())
                                .insert(block_materials.revealed_5.clone());
                        }
                    },
                }
            }
            BlockEvent::Mark(entity) => {
                debug!("Mark block {entity:?}");
                commands
                    .entity(*entity)
                    .remove::<Handle<StandardMaterial>>();
                match block.marked {
                    true => {
                        debug!("Unmark block {entity:?} as mine");
                        block.marked = false;
                        commands
                            .entity(*entity)
                            .insert(block_materials.hidden.clone());
                    }
                    false => {
                        debug!("Mark block {entity:?} as mine");
                        block.marked = true;
                        commands
                            .entity(*entity)
                            .insert(block_materials.marked.clone());
                    }
                }
            }
        }
    }
}

#[cfg(feature = "debug-draw")]
fn block_gizmos(mut gizmos: Gizmos, blocks: Query<&Transform, With<Block>>) {
    for tf in blocks.iter() {
        gizmos.cuboid(*tf, Color::RED);
    }
}
