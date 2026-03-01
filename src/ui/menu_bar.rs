use eframe::egui;

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
    current_file: Option<&std::path::PathBuf>,
    is_running: bool,
    docs_open: bool,
    is_dirty: bool,
    _is_new: bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    // Keyboard shortcuts
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
            // ── File menu ─────────────────────────────────────────────────
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

            // ── Run button ────────────────────────────────────────────────
            let run_label = if is_running { "⏳ Running…" } else { "▶  Run" };
            let run_btn = egui::Button::new(
                egui::RichText::new(run_label).color(if is_running {
                    egui::Color32::from_rgb(180, 180, 60)
                } else {
                    egui::Color32::from_rgb(100, 220, 100)
                }),
            );
            if ui.add_enabled(!is_running, run_btn).clicked() {
                action = MenuAction::Run;
            }

            ui.separator();

            // ── Docs toggle ───────────────────────────────────────────────
            let docs_btn = egui::Button::new(
                egui::RichText::new("Docs").color(if docs_open {
                    egui::Color32::from_rgb(100, 200, 255)
                } else {
                    egui::Color32::from_rgb(180, 180, 180)
                }),
            );
            if ui.add(docs_btn).clicked() {
                action = MenuAction::ToggleDocs;
            }

            // ── Unsaved-changes dot (right side, no filename label) ───────
            if is_dirty {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dot_radius = 5.0_f32;
                    let dot_size = egui::Vec2::splat(dot_radius * 2.0 + 6.0);
                    let (rect, resp) =
                        ui.allocate_exact_size(dot_size, egui::Sense::hover());
                    ui.painter().circle_filled(
                        rect.center(),
                        dot_radius,
                        egui::Color32::from_rgb(220, 160, 60),
                    );
                    resp.on_hover_text("Unsaved changes");
                });
            }
        });
    });

    action
}