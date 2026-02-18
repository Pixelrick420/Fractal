use eframe::egui;
use std::path::PathBuf;

#[derive(Default)]
pub struct MenuBarState {}

pub enum MenuAction {
    OpenDialog,
    SaveDialog,
    SaveCurrent,
    New,
    Run,
    None,
}

pub fn show_menu_bar(
    ctx: &egui::Context,
    _state: &mut MenuBarState,
    current_file: Option<&PathBuf>,
    is_running: bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📂 Open").clicked() {
                    action = MenuAction::OpenDialog;
                    ui.close_menu();
                }
                if ui.button("💾 Save").clicked() {
                    if current_file.is_some() {
                        action = MenuAction::SaveCurrent;
                    } else {
                        action = MenuAction::SaveDialog;
                    }
                    ui.close_menu();
                }
                if ui.button("💾 Save As").clicked() {
                    action = MenuAction::SaveDialog;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🆕 New").clicked() {
                    action = MenuAction::New;
                    ui.close_menu();
                }
            });

            ui.separator();

            let run_label = if is_running {
                "⏳ Running…"
            } else {
                "▶  Run"
            };
            let run_btn = egui::Button::new(egui::RichText::new(run_label).color(if is_running {
                egui::Color32::from_rgb(180, 180, 60)
            } else {
                egui::Color32::from_rgb(100, 220, 100)
            }));
            if ui.add_enabled(!is_running, run_btn).clicked() {
                action = MenuAction::Run;
            }

            ui.separator();

            match current_file {
                Some(p) => {
                    ui.label(format!("📄 {}", p.display()));
                }
                None => {
                    ui.label("📄 Untitled");
                }
            }
        });
    });

    action
}
