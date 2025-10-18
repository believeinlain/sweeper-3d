use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use egui_extras::install_image_loaders;

use crate::GameState;

pub struct LoaderPlugin;
impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameAssets>()
            .add_systems(Startup, load_assets)
            .add_systems(
                EguiPrimaryContextPass,
                show_splash.run_if(in_state(GameState::Loading)),
            )
            .add_systems(Update, update.run_if(in_state(GameState::Loading)));
    }
}

#[derive(Resource, Default)]
pub struct GameAssets {
    pub sweeper_objects: Loadable<SweeperObjects, Gltf>,
    pub pop1: Handle<AudioSource>,
    pub pop2: Handle<AudioSource>,
    pub pop3: Handle<AudioSource>,
    pub concrete_02_albedo: Handle<Image>,
    pub concrete_02_normal: Handle<Image>,
    pub concrete_02_orm: Handle<Image>,
}
impl GameAssets {
    fn all_loaded(&self) -> bool {
        self.sweeper_objects.is_loaded()
    }
}

trait IsLoaded {
    fn is_loaded(&self) -> bool;
}

/// Assets to be loaded from "sweeper_objects.gltf"
#[derive(Resource)]
pub struct SweeperObjects {
    pub block_merged: Handle<Mesh>,
    pub mine_merged: Handle<Mesh>,
    pub ring: Handle<Mesh>,
    pub single1: Handle<Mesh>,
    pub single2: Handle<Mesh>,
    pub single3: Handle<Mesh>,
    pub single4: Handle<Mesh>,
    pub orbit1: Handle<Mesh>,
    pub orbit2: Handle<Mesh>,
    pub orbit3: Handle<Mesh>,
    pub orbit4: Handle<Mesh>,
}

/// Resource loadable from base type A.
#[derive(Resource, Default)]
pub enum Loadable<T, A>
where
    A: Asset,
{
    #[default]
    Init,
    Loading(Handle<A>),
    Loaded(T),
}
impl<T, A> Loadable<T, A>
where
    A: Asset,
{
    /// # Panics
    /// Panics if the resource is not fully loaded.
    pub fn unwrap(&self) -> &T {
        match self {
            Loadable::Init => panic!("Resource not loaded: Asset not initialized"),
            Loadable::Loading(asset) => {
                panic!("Resource not loaded: Asset {asset:?} still loading")
            }
            Loadable::Loaded(res) => res,
        }
    }
}
impl<T, A> IsLoaded for Loadable<T, A>
where
    A: Asset,
{
    fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }
}
impl<T, A> From<Handle<A>> for Loadable<T, A>
where
    A: Asset,
{
    fn from(value: Handle<A>) -> Self {
        Loadable::Loading(value)
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    log::info!("Loading sweeper_objects");
    let sweeper_objects = Loadable::from(asset_server.load("sweeper_objects.gltf"));
    log::info!("Loading sound effects");
    let pop1 = asset_server.load("pop1.ogg");
    let pop2 = asset_server.load("pop2.ogg");
    let pop3 = asset_server.load("pop3.ogg");
    log::info!("Loading textures");
    let concrete_02_albedo = asset_server.load("concrete_02_albedo.png");
    let concrete_02_normal = asset_server.load("concrete_02_normal.png");
    let concrete_02_orm = asset_server.load("concrete_02_orm.png");
    commands.insert_resource(GameAssets {
        sweeper_objects,
        pop1,
        pop2,
        pop3,
        concrete_02_albedo,
        concrete_02_normal,
        concrete_02_orm,
    });
}

fn update(
    asset_server: Res<AssetServer>,
    mut game_assets: ResMut<GameAssets>,
    mut next_state: ResMut<NextState<GameState>>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    time: Res<Time>,
) {
    if game_assets.all_loaded() {
        // Hold so we can see the splash screen
        if time.elapsed_secs() >= 1.0 {
            log::info!("All assets loaded");
            next_state.set(GameState::MenuMain);
        }
    } else if let Loadable::Loading(sweeper_objects) = &game_assets.sweeper_objects {
        if asset_server.is_loaded_with_dependencies(sweeper_objects) {
            log::info!("Sweeper objects loaded");
            let gltf = gltf_assets.get(sweeper_objects).unwrap();
            let get_mesh = |name| {
                gltf_meshes
                    .get(&gltf.named_meshes[name])
                    .unwrap()
                    .primitives[0]
                    .mesh
                    .clone()
            };
            let block_merged = get_mesh("BlockMerged");
            let mine_merged = get_mesh("MineMerged");
            let ring = get_mesh("Ring");
            let single1 = get_mesh("Single1");
            let single2 = get_mesh("Single2");
            let single3 = get_mesh("Single3");
            let single4 = get_mesh("Single4");
            let orbit1 = get_mesh("Orbit1");
            let orbit2 = get_mesh("Orbit2");
            let orbit3 = get_mesh("Orbit3");
            let orbit4 = get_mesh("Orbit4");
            game_assets.sweeper_objects = Loadable::Loaded(SweeperObjects {
                block_merged,
                mine_merged,
                ring,
                single1,
                single2,
                single3,
                single4,
                orbit1,
                orbit2,
                orbit3,
                orbit4,
            })
        }
    }
}

fn show_splash(mut contexts: EguiContexts, mut window: Query<&mut Window>) {
    let ctx = contexts.ctx_mut().expect("unable to get gui context");
    install_image_loaders(ctx);
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::BLACK))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(egui::include_image!(
                    "../embed/splash.svg"
                )));
            });
        });
    window.single_mut().unwrap().visible = true;
}
