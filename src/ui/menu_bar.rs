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
    OpenRecent(PathBuf),
    Search,
    Replace,
    None,
}

const BTN_W: f32 = 72.0;
const BTN_H: f32 = 28.0;
const BTN_ROUNDING: f32 = 5.0;
const ICON_BTN_W: f32 = 34.0;

pub fn show_menu_bar(
    ctx: &egui::Context,
    _state: &mut MenuBarState,
    current_file: Option<&PathBuf>,
    is_running: bool,
    docs_open: bool,
    theme: &Theme,
    recent_files: &[PathBuf],
    search_bar_visible: bool,
) -> MenuAction {
    let mut action = MenuAction::None;

    // Only handle Ctrl+F / Ctrl+H here when the search bar is not already open.
    // When the search bar IS open, it consumes those keys itself.
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
        } else if ctrl && i.key_pressed(egui::Key::F) && !search_bar_visible {
            action = MenuAction::Search;
        } else if ctrl && i.key_pressed(egui::Key::H) && !search_bar_visible {
            action = MenuAction::Replace;
        }
    });

    let t = theme;

    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::none()
                .fill(t.tab_bar_bg)
                .inner_margin(egui::Margin {
                    left: 12.0,
                    right: 12.0,
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

            ui.set_min_height(44.0);
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(ic::APP_LOGO).size(17.0).color(t.accent));
                ui.add_space(10.0);

                let (div_rect, _) =
                    ui.allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(div_rect, egui::Rounding::ZERO, t.border);
                ui.add_space(10.0);

                // ── File menu ─────────────────────────────────────────────
                let file_id = ui.make_persistent_id("menu_file_popup");
                let (file_rect, _) =
                    ui.allocate_exact_size(egui::vec2(BTN_W, BTN_H), egui::Sense::hover());
                let file_clicked = ui
                    .interact(file_rect, file_id.with("btn"), egui::Sense::click())
                    .clicked();
                let file_hovered = ui.rect_contains_pointer(file_rect);
                let popup_open = ui.memory(|m| m.is_popup_open(file_id));
                let file_active = file_hovered || popup_open;

                if file_clicked {
                    ui.memory_mut(|m| m.toggle_popup(file_id));
                }

                paint_menu_button(ui, file_rect, "File", ic::FILE_OPEN, file_active, false, t);

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
                        ui.set_min_width(260.0);
                        ui.add_space(4.0);

                        if icon_menu_item(ui, ic::FILE_NEW, "New File", "Ctrl+N", t) {
                            action = MenuAction::New;
                            ui.memory_mut(|m| m.close_popup());
                        }
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

                        styled_separator(ui, t);

                        // ── Recent Files ── flat list under a header ───────
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new("  Recent Files")
                                .size(11.0)
                                .color(t.tab_inactive_fg),
                        );
                        ui.add_space(2.0);

                        if recent_files.is_empty() {
                            let (r, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 24.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().text(
                                egui::pos2(r.left() + 20.0, r.center().y),
                                egui::Align2::LEFT_CENTER,
                                "No recent files",
                                egui::FontId::proportional(12.5),
                                t.tab_inactive_fg,
                            );
                        } else {
                            for path in recent_files.iter() {
                                let name = path
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| path.to_string_lossy().to_string());

                                let full = path.to_string_lossy().to_string();
                                let short_path = if full.len() > 38 {
                                    format!("…{}", &full[full.len() - 37..])
                                } else {
                                    full.clone()
                                };

                                let (r, resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 28.0),
                                    egui::Sense::click(),
                                );
                                let hov = resp.hovered();
                                if hov {
                                    ui.painter().rect_filled(
                                        r,
                                        egui::Rounding::same(4.0),
                                        t.accent,
                                    );
                                }
                                let fg = if hov { t.tab_bar_bg } else { t.menu_fg };
                                let hint = if hov {
                                    egui::Color32::from_rgba_premultiplied(
                                        t.tab_bar_bg.r(),
                                        t.tab_bar_bg.g(),
                                        t.tab_bar_bg.b(),
                                        160,
                                    )
                                } else {
                                    egui::Color32::from_rgba_premultiplied(
                                        t.tab_inactive_fg.r(),
                                        t.tab_inactive_fg.g(),
                                        t.tab_inactive_fg.b(),
                                        130,
                                    )
                                };
                                // File name on the left
                                ui.painter().text(
                                    egui::pos2(r.left() + 20.0, r.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    &name,
                                    egui::FontId::proportional(13.0),
                                    fg,
                                );
                                // Full path on the right, dimmed
                                ui.painter().text(
                                    egui::pos2(r.right() - 10.0, r.center().y),
                                    egui::Align2::RIGHT_CENTER,
                                    short_path,
                                    egui::FontId::proportional(10.5),
                                    hint,
                                );

                                if resp.clicked() {
                                    action = MenuAction::OpenRecent(path.clone());
                                    ui.memory_mut(|m| m.close_popup());
                                }
                            }
                        }

                        styled_separator(ui, t);

                        // ── Find / Replace ─────────────────────────────────
                        if icon_menu_item(ui, ic::DOCS, "Find…", "Ctrl+F", t) {
                            action = MenuAction::Search;
                            ui.memory_mut(|m| m.close_popup());
                        }
                        if icon_menu_item(ui, ic::DOCS, "Replace…", "Ctrl+H", t) {
                            action = MenuAction::Replace;
                            ui.memory_mut(|m| m.close_popup());
                        }

                        styled_separator(ui, t);

                        let docs_label = if docs_open { "Close Docs" } else { "Open Docs" };
                        if icon_menu_item(ui, ic::DOCS, docs_label, "Ctrl+D", t) {
                            action = MenuAction::ToggleDocs;
                            ui.memory_mut(|m| m.close_popup());
                        }
                        ui.add_space(4.0);
                    },
                );

                ui.add_space(4.0);

                // ── Run button ────────────────────────────────────────────
                let run_label = if is_running { "Running…" } else { "Run" };
                let run_icon = if is_running { ic::RUNNING } else { ic::RUN };
                let run_id = egui::Id::new("menu_run_btn");
                let (run_rect, _) =
                    ui.allocate_exact_size(egui::vec2(BTN_W, BTN_H), egui::Sense::hover());
                let run_hovered = ui.rect_contains_pointer(run_rect) && !is_running;

                paint_run_button(ui, run_rect, run_icon, run_label, is_running, run_hovered, t);

                let run_resp = ui.interact(run_rect, run_id, egui::Sense::click());
                if run_resp.clicked() && !is_running {
                    action = MenuAction::Run;
                }

                // ── Settings gear (right-aligned) ─────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let gear_id = egui::Id::new("menu_gear_btn");
                    let (gear_rect, _) =
                        ui.allocate_exact_size(egui::vec2(ICON_BTN_W, BTN_H), egui::Sense::hover());
                    let gear_hovered = ui.rect_contains_pointer(gear_rect);

                    paint_icon_button(ui, gear_rect, ic::SETTINGS, gear_hovered, t);

                    let gear_resp = ui.interact(gear_rect, gear_id, egui::Sense::click());
                    if gear_resp.clicked() {
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

// ── Paint helpers ─────────────────────────────────────────────────────────────

fn paint_menu_button(
    ui: &egui::Ui,
    rect: egui::Rect,
    label: &str,
    icon: &str,
    active: bool,
    _disabled: bool,
    t: &Theme,
) {
    if active {
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(BTN_ROUNDING), t.button_hover_bg);
        ui.painter().rect_stroke(
            rect,
            egui::Rounding::same(BTN_ROUNDING),
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(
                    t.border.r(),
                    t.border.g(),
                    t.border.b(),
                    180,
                ),
            ),
        );
    }
    let fg = if active { t.tab_active_fg } else { t.tab_inactive_fg };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{icon}  {label}"),
        egui::FontId::proportional(12.5),
        fg,
    );
}

fn paint_run_button(
    ui: &egui::Ui,
    rect: egui::Rect,
    icon: &str,
    label: &str,
    is_running: bool,
    hovered: bool,
    t: &Theme,
) {
    let rounding = egui::Rounding::same(BTN_ROUNDING);
    if is_running {
        ui.painter().rect_filled(
            rect,
            rounding,
            egui::Color32::from_rgba_premultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 30),
        );
        ui.painter().rect_stroke(
            rect,
            rounding,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(
                    t.accent.r(),
                    t.accent.g(),
                    t.accent.b(),
                    60,
                ),
            ),
        );
    } else if hovered {
        ui.painter().rect_filled(rect, rounding, t.accent);
    }
    let fg = if is_running {
        egui::Color32::from_rgba_premultiplied(
            t.tab_inactive_fg.r(),
            t.tab_inactive_fg.g(),
            t.tab_inactive_fg.b(),
            110,
        )
    } else if hovered {
        t.tab_bar_bg
    } else {
        t.accent
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{icon}  {label}"),
        egui::FontId::proportional(12.5),
        fg,
    );
}

fn paint_icon_button(ui: &egui::Ui, rect: egui::Rect, icon: &str, active: bool, t: &Theme) {
    let rounding = egui::Rounding::same(BTN_ROUNDING);
    if active {
        ui.painter().rect_filled(rect, rounding, t.button_hover_bg);
        ui.painter().rect_stroke(
            rect,
            rounding,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(
                    t.border.r(),
                    t.border.g(),
                    t.border.b(),
                    180,
                ),
            ),
        );
    }
    let fg = if active { t.tab_active_fg } else { t.tab_inactive_fg };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(14.0),
        fg,
    );
}

fn styled_separator(ui: &mut egui::Ui, t: &Theme) {
    ui.add_space(2.0);
    let (sep_rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(sep_rect, egui::Rounding::ZERO, t.border);
    ui.add_space(2.0);
}

fn icon_menu_item(ui: &mut egui::Ui, icon: &str, label: &str, shortcut: &str, t: &Theme) -> bool {
    let desired_size = egui::vec2(ui.available_width(), 28.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(4.0), t.accent);
    }
    let text_fg = if hovered { t.tab_bar_bg } else { t.menu_fg };
    ui.painter().text(
        egui::pos2(rect.left() + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{icon}   {label}"),
        egui::FontId::proportional(13.5),
        text_fg,
    );
    if !shortcut.is_empty() {
        let shortcut_fg = if hovered {
            egui::Color32::from_rgba_premultiplied(
                t.tab_bar_bg.r(),
                t.tab_bar_bg.g(),
                t.tab_bar_bg.b(),
                180,
            )
        } else {
            egui::Color32::from_rgba_premultiplied(
                t.tab_inactive_fg.r(),
                t.tab_inactive_fg.g(),
                t.tab_inactive_fg.b(),
                160,
            )
        };
        ui.painter().text(
            egui::pos2(rect.right() - 10.0, rect.center().y),
            egui::Align2::RIGHT_CENTER,
            shortcut,
            egui::FontId::proportional(11.5),
            shortcut_fg,
        );
    }
    resp.clicked()
}