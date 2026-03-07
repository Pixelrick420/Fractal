use crate::ui::icons as ic;
use crate::ui::theme::Theme;
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
    OpenSettings,
    None,
}

pub fn show_menu_bar(
    ctx: &egui::Context,
    _state: &mut MenuBarState,
    current_file: Option<&PathBuf>,
    is_running: bool,
    docs_open: bool,
    theme: &Theme,
) -> MenuAction {
    let mut action = MenuAction::None;

    ctx.input_mut(|i| {
        let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
        if ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S) {
            action = MenuAction::SaveDialog;
        } else if ctrl && i.key_pressed(egui::Key::S) {
            action = if current_file.is_some() {
                MenuAction::SaveCurrent
            } else {
                MenuAction::SaveDialog
            };
        } else if ctrl && i.key_pressed(egui::Key::O) {
            action = MenuAction::OpenDialog;
        } else if ctrl && i.key_pressed(egui::Key::N) {
            action = MenuAction::New;
        } else if ctrl && i.key_pressed(egui::Key::D) {
            action = MenuAction::ToggleDocs;
        }
    });

    let t = theme;

    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::none()
                .fill(t.tab_bar_bg)
                .inner_margin(egui::Margin {
                    left: 10.0,
                    right: 10.0,
                    top: 0.0,
                    bottom: 0.0,
                }),
        )
        .show(ctx, |ui| {
            {
                let s = ui.style_mut();
                s.visuals.window_fill = t.menu_bg;
                s.visuals.panel_fill = t.menu_bg;
                s.visuals.override_text_color = Some(t.menu_fg);

                s.visuals.widgets.noninteractive.bg_fill = egui::Color32::TRANSPARENT;
                s.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
                s.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
                s.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                s.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                s.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
                s.visuals.widgets.hovered.bg_fill = egui::Color32::TRANSPARENT;
                s.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                s.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, t.tab_active_fg);
                s.visuals.widgets.active.bg_fill = egui::Color32::TRANSPARENT;
                s.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                s.visuals.widgets.open.bg_fill = egui::Color32::TRANSPARENT;
                s.visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
            }

            ui.set_min_height(36.0);
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(ic::APP_LOGO).size(17.0).color(t.accent));
                ui.add_space(8.0);

                let file_id = ui.make_persistent_id("menu_file_popup");
                let file_size = egui::vec2(40.0, 26.0);
                let (file_rect, _) = ui.allocate_exact_size(file_size, egui::Sense::hover());
                let file_clicked = ui
                    .interact(file_rect, file_id.with("btn"), egui::Sense::click())
                    .clicked();
                let file_hovered = ui.rect_contains_pointer(file_rect);
                let popup_open = ui.memory(|m| m.is_popup_open(file_id));

                if file_clicked {
                    ui.memory_mut(|m| m.toggle_popup(file_id));
                }
                if file_hovered || popup_open {
                    ui.painter().rect_filled(
                        file_rect,
                        egui::Rounding::same(4.0),
                        t.button_hover_bg,
                    );
                }
                ui.painter().text(
                    file_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "File",
                    egui::FontId::proportional(12.5),
                    if file_hovered || popup_open {
                        t.tab_active_fg
                    } else {
                        t.tab_inactive_fg
                    },
                );

                egui::popup::popup_below_widget(
                    ui,
                    file_id,
                    &ui.interact(file_rect, file_id.with("anchor"), egui::Sense::hover()),
                    egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                    |ui| {
                        let s = ui.style_mut();
                        s.visuals.window_fill = t.menu_bg;
                        s.visuals.panel_fill = t.menu_bg;
                        s.visuals.override_text_color = Some(t.menu_fg);
                        s.visuals.widgets.noninteractive.bg_fill = t.menu_bg;
                        s.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                        s.visuals.widgets.hovered.bg_fill = t.menu_hover_bg;
                        ui.set_min_width(220.0);

                        if icon_menu_item(ui, ic::FILE_OPEN, "Open…", "Ctrl+O", t) {
                            action = MenuAction::OpenDialog;
                            ui.memory_mut(|m| m.close_popup());
                        }
                        if icon_menu_item(ui, ic::FILE_SAVE, "Save", "Ctrl+S", t) {
                            action = if current_file.is_some() {
                                MenuAction::SaveCurrent
                            } else {
                                MenuAction::SaveDialog
                            };
                            ui.memory_mut(|m| m.close_popup());
                        }
                        if icon_menu_item(ui, ic::FILE_SAVE_AS, "Save As…", "Ctrl+⇧+S", t) {
                            action = MenuAction::SaveDialog;
                            ui.memory_mut(|m| m.close_popup());
                        }
                        ui.separator();
                        if icon_menu_item(ui, ic::FILE_NEW, "New File", "Ctrl+N", t) {
                            action = MenuAction::New;
                            ui.memory_mut(|m| m.close_popup());
                        }
                        ui.separator();
                        let docs_label = if docs_open { "Close Docs" } else { "Open Docs" };
                        if icon_menu_item(ui, ic::DOCS, docs_label, "Ctrl+D", t) {
                            action = MenuAction::ToggleDocs;
                            ui.memory_mut(|m| m.close_popup());
                        }
                    },
                );
                ui.add_space(4.0);

                let run_label = if is_running { "Running…" } else { "Run" };
                let run_icon = if is_running { ic::RUNNING } else { ic::RUN };

                let run_id = egui::Id::new("menu_run_btn");
                let run_size = egui::vec2(80.0, 26.0);
                let (run_rect, _) = ui.allocate_exact_size(run_size, egui::Sense::hover());
                let run_hovered = ui.rect_contains_pointer(run_rect) && !is_running;

                if run_hovered {
                    ui.painter().rect_filled(
                        run_rect,
                        egui::Rounding::same(4.0),
                        t.button_hover_bg,
                    );
                }
                let run_fg = if is_running {
                    egui::Color32::from_rgba_premultiplied(
                        t.tab_inactive_fg.r(),
                        t.tab_inactive_fg.g(),
                        t.tab_inactive_fg.b(),
                        120,
                    )
                } else if run_hovered {
                    t.tab_active_fg
                } else {
                    t.tab_inactive_fg
                };
                ui.painter().text(
                    run_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{run_icon}  {run_label}"),
                    egui::FontId::proportional(12.5),
                    run_fg,
                );
                let run_resp = ui.interact(run_rect, run_id, egui::Sense::click());
                if run_resp.clicked() && !is_running {
                    action = MenuAction::Run;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let gear_id = egui::Id::new("menu_gear_btn");
                    let gear_size = egui::vec2(28.0, 26.0);
                    let (gear_rect, _) = ui.allocate_exact_size(gear_size, egui::Sense::hover());
                    let gear_hovered = ui.rect_contains_pointer(gear_rect);

                    if gear_hovered {
                        ui.painter().rect_filled(
                            gear_rect,
                            egui::Rounding::same(4.0),
                            t.button_hover_bg,
                        );
                    }
                    ui.painter().text(
                        gear_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        ic::SETTINGS,
                        egui::FontId::proportional(15.0),
                        if gear_hovered {
                            t.tab_active_fg
                        } else {
                            t.tab_inactive_fg
                        },
                    );
                    let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
                    if gear_resp.on_hover_text("Settings").clicked() {
                        action = MenuAction::OpenSettings;
                    }
                });
            });

            let r = ui.min_rect();
            ui.painter().line_segment(
                [r.left_bottom(), r.right_bottom()],
                egui::Stroke::new(1.0, t.border),
            );
        });

    action
}

fn icon_menu_item(ui: &mut egui::Ui, icon: &str, label: &str, shortcut: &str, t: &Theme) -> bool {
    let mut clicked = false;
    let resp = ui.add(
        egui::Button::new(
            egui::RichText::new(format!("{icon}   {label}"))
                .size(13.0)
                .color(t.menu_fg),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::NONE)
        .min_size(egui::vec2(180.0, 26.0)),
    );

    if !shortcut.is_empty() {
        let r = resp.rect;
        ui.painter().text(
            egui::pos2(r.right() - 6.0, r.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(11.0),
            t.tab_inactive_fg,
        );
    }

    if resp.hovered() {
        ui.painter()
            .rect_filled(resp.rect, egui::Rounding::same(4.0), t.menu_hover_bg);
    }

    if resp.clicked() {
        clicked = true;
        ui.close_menu();
    }
    clicked
}
