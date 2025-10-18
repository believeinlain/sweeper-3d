// Disable console window in Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPlugin,
    egui::{self, Color32},
};
use egui_extras::install_image_loaders;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // Show the window only once internal startup has finished and we're running systems
                    visible: false,
                    title: "Splash".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            EguiPlugin,
        ))
        .add_systems(Update, update)
        .run();
}

fn update(mut contexts: EguiContexts, mut window: Query<&mut Window>) {
    let ctx = contexts.ctx_mut();
    install_image_loaders(ctx);
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(Color32::BLACK))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(egui::include_image!(
                    "../embed/splash.svg"
                )));
            });
        });
    window.single_mut().visible = true;
}
