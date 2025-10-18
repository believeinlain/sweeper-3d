use bevy::{app::AppExit, prelude::*};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::{FieldSettings, GameSettings, GameState, Safety, game::GameResult};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default()).add_systems(
            EguiPrimaryContextPass,
            (
                display_main_menu.run_if(in_state(GameState::MenuMain)),
                display_custom_menu.run_if(in_state(GameState::MenuCustom)),
                display_settings_menu.run_if(in_state(GameState::MenuSettings)),
                display_game_over.run_if(in_state(GameState::GameOver)),
            ),
        );
    }
}

fn global_settings(ctx: &mut egui::Context) {
    ctx.style_mut(|style| {
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(32.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );
        style.spacing.item_spacing = egui::Vec2::new(5.0, 5.0);
    });
    let mut visuals = egui::Visuals::dark();
    visuals.window_shadow = egui::epaint::Shadow::NONE;
}

fn create_menu_window<'a>(title: impl Into<egui::WidgetText>) -> egui::Window<'a> {
    egui::Window::new(title)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .movable(false)
        .resizable(false)
}

fn display_main_menu(
    mut contexts: EguiContexts,
    mut field_settings: ResMut<FieldSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit_events: MessageWriter<AppExit>,
) {
    let ctx = contexts.ctx_mut().expect("unable to get gui context");
    global_settings(ctx);
    create_menu_window("Sweeper 3D").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Small")).clicked() {
                        field_settings.set_if_neq(FieldSettings::small());
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Medium")).clicked() {
                        field_settings.set_if_neq(FieldSettings::medium());
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Large")).clicked() {
                        field_settings.set_if_neq(FieldSettings::large());
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Custom")).clicked() {
                        field_settings.set_if_neq(FieldSettings::default());
                        next_state.set(GameState::MenuCustom);
                    }
                });
                if ui.add(egui::Button::new("Settings")).clicked() {
                    next_state.set(GameState::MenuSettings);
                }
                if ui.add(egui::Button::new("Quit")).clicked() {
                    exit_events.write(AppExit::Success);
                }
            });
        });
    });
}

fn display_custom_menu(
    mut contexts: EguiContexts,
    mut field_settings: ResMut<FieldSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (field_size, mine_density) = field_settings.fields_mut();
    let ctx = contexts.ctx_mut().expect("unable to get gui context");
    global_settings(ctx);
    create_menu_window("Custom Game").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.add(egui::Label::new("Size:"));
                    ui.add(egui::DragValue::new(&mut field_size[0]).range(1..=20));
                    ui.add(egui::DragValue::new(&mut field_size[1]).range(1..=20));
                    ui.add(egui::DragValue::new(&mut field_size[2]).range(1..=20));
                });
                ui.horizontal_centered(|ui| {
                    ui.add(egui::Label::new("Mine Density:"));
                    ui.add(
                        egui::Slider::new(mine_density, 0.01..=1.0)
                            .min_decimals(2)
                            .max_decimals(2),
                    );
                });
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Start")).clicked() {
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Back")).clicked() {
                        next_state.set(GameState::MenuMain);
                    }
                });
            });
        });
    });
}

fn display_settings_menu(
    mut contexts: EguiContexts,
    mut game_settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let safety = &mut game_settings.safety;
    let ctx = contexts.ctx_mut().expect("unable to get gui context");
    global_settings(ctx);
    create_menu_window("Settings").show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    ui.label("First Block Safety:");
                    ui.radio_value(safety, Safety::Clear, "Clear")
                        .on_hover_text(
                            "The first block cleared is guaranteed to reveal more than one space.",
                        );
                    ui.radio_value(safety, Safety::Safe, "Safe")
                        .on_hover_text(concat!(
                            "The first block cleared is guaranteed to be safe, ",
                            "but may only reveal one space."
                        ));
                    ui.radio_value(safety, Safety::Random, "Random")
                        .on_hover_text(
                            "No safety guarantees - the first block cleared might contain a mine.",
                        );
                });
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Back")).clicked() {
                        next_state.set(GameState::MenuMain);
                    }
                });
            });
        });
    });
}

fn display_game_over(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit_events: MessageWriter<AppExit>,
    game_result: Res<GameResult>,
) {
    let ctx = contexts.ctx_mut().expect("unable to get gui context");
    global_settings(ctx);
    egui::Window::new(match *game_result {
        GameResult::Unfinished => {
            log::error!("Should not be displaying game over menu when GameResult::Unfinished");
            "Game Over"
        }
        GameResult::Failure => "Game Over",
        GameResult::Victory => "Victory!",
    })
    .anchor(egui::Align2::CENTER_BOTTOM, [0.0, 0.0])
    .collapsible(false)
    .movable(false)
    .resizable(false)
    .show(ctx, |ui| {
        ui.allocate_ui(egui::Vec2::new(0.0, 0.0), |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal_centered(|ui| {
                    if ui.add(egui::Button::new("Restart")).clicked() {
                        next_state.set(GameState::GameStart);
                    }
                    if ui.add(egui::Button::new("Main Menu")).clicked() {
                        next_state.set(GameState::MenuMain);
                    }
                    if ui.add(egui::Button::new("Quit")).clicked() {
                        exit_events.write(AppExit::Success);
                    }
                });
            });
        });
    });
}
