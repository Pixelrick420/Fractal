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
    ToggleDocs,
    None,
}

pub fn show_menu_bar(
    ctx: &egui::Context,
    _state: &mut MenuBarState,
    current_file: Option<&PathBuf>,
    is_running: bool,
    docs_open: bool,
    is_dirty: bool,
    is_new: bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    ctx.input_mut(|i| {
        let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
        let shift = i.modifiers.shift;
        if ctrl && shift && i.key_pressed(egui::Key::S) {
            action = MenuAction::SaveDialog;
        } else if ctrl && i.key_pressed(egui::Key::S) {
            if current_file.is_some() {
                action = MenuAction::SaveCurrent;
            } else {
                action = MenuAction::SaveDialog;
            }
        }
    });

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📂 Open").clicked() {
                    action = MenuAction::OpenDialog;
                    ui.close_menu();
                }
                if ui.button("💾 Save        Ctrl+S").clicked() {
                    if current_file.is_some() {
                        action = MenuAction::SaveCurrent;
                    } else {
                        action = MenuAction::SaveDialog;
                    }
                    ui.close_menu();
                }
                if ui.button("💾 Save As  Ctrl+Shift+S").clicked() {
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

            let docs_label = "Docs";
            let docs_btn = egui::Button::new(egui::RichText::new(docs_label).color(if docs_open {
                egui::Color32::from_rgb(100, 200, 255)
            } else {
                egui::Color32::from_rgb(180, 180, 180)
            }));
            if ui.add(docs_btn).clicked() {
                action = MenuAction::ToggleDocs;
            }

            ui.separator();

            let file_label = match current_file {
                Some(p) => p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string()),
                None => "Untitled".to_string(),
            };

            let show_dot = is_new || is_dirty;

            let indicator_tooltip = if is_new {
                "New file — not saved yet"
            } else if is_dirty {
                "Unsaved changes"
            } else {
                "All changes saved"
            };

            if show_dot {
                let dot_radius = 5.0_f32;
                let dot_size = egui::Vec2::splat(dot_radius * 2.0 + 4.0);
                let (dot_rect, dot_resp) = ui.allocate_exact_size(dot_size, egui::Sense::hover());
                ui.painter().circle_filled(
                    dot_rect.center(),
                    dot_radius,
                    egui::Color32::from_rgb(120, 120, 120),
                );
                dot_resp.on_hover_text(indicator_tooltip);
            }

            let label_resp = ui.label(format!("{}", file_label));
            label_resp.on_hover_text(indicator_tooltip);
        });
    });

    action
}
