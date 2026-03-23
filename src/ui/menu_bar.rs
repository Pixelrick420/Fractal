use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;
use egui::{LayerId, Order, Popup, PopupAnchor, PopupCloseBehavior};
use std::path::PathBuf;

#[derive(Default)]
pub struct MenuBarState {}

pub enum MenuAction {
    OpenDialog,
    SaveDialog,
    SaveCurrent,
    New,
    Run,

    StepRun,

    StepStop,
    ToggleDocs,
    ToggleTreeView,
    ToggleVarView,
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

const FLYOUT_GAP: f32 = 0.0;

pub fn show_menu_bar(
    ctx: &egui::Context,
    _state: &mut MenuBarState,
    current_file: Option<&PathBuf>,
    is_running: bool,
    is_debugging: bool,
    docs_open: bool,
    tree_view_open: bool,
    var_view_open: bool,
    theme: &Theme,
    recent_files: &[PathBuf],
    search_bar_visible: bool,
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
        } else if ctrl && i.key_pressed(egui::Key::F) && !search_bar_visible {
            action = MenuAction::Search;
        } else if ctrl && i.key_pressed(egui::Key::H) {
            action = MenuAction::Replace;
        } else if i.key_pressed(egui::Key::F5) {
            action = MenuAction::StepRun;
        } else if i.key_pressed(egui::Key::F6) {
            action = MenuAction::StepStop;
        }
    });

    let t = theme;

    let flyout_open_id = egui::Id::new("recent_flyout_open");
    let flyout_row_rect_id = egui::Id::new("recent_row_rect");
    let recent_clicked_id = egui::Id::new("recent_clicked_path");

    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::new()
                .fill(t.tab_bar_bg)
                .inner_margin(egui::Margin {
                    left: 12,
                    right: 12,
                    top: 0,
                    bottom: 0,
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
                    .rect_filled(div_rect, egui::CornerRadius::ZERO, t.border);
                ui.add_space(10.0);

                let file_id = ui.make_persistent_id("menu_file_popup");
                let (file_rect, _) =
                    ui.allocate_exact_size(egui::vec2(BTN_W, BTN_H), egui::Sense::hover());

                let popup_open = Popup::is_id_open(ctx, file_id);
                let file_clicked = ui
                    .interact(file_rect, file_id.with("btn"), egui::Sense::click())
                    .clicked();
                if file_clicked {
                    Popup::toggle_id(ctx, file_id);
                }

                let file_hovered = ui.rect_contains_pointer(file_rect);
                let file_active = file_hovered || popup_open;

                paint_menu_button(ui, file_rect, "File", ic::FILE_OPEN, file_active, false, t);

                let anchor = file_rect;

                if popup_open {
                    Popup::new(
                        file_id,
                        ctx.clone(),
                        PopupAnchor::ParentRect(anchor),
                        LayerId::new(Order::Foreground, file_id),
                    )
                    .open_memory(None)
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
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
                            Popup::close_id(ctx, file_id);
                        }
                        if icon_menu_item(ui, ic::FILE_OPEN, "Open…", "Ctrl+O", t) {
                            action = MenuAction::OpenDialog;
                            Popup::close_id(ctx, file_id);
                        }
                        if icon_menu_item(ui, ic::FILE_SAVE, "Save", "Ctrl+S", t) {
                            action = if current_file.is_some() {
                                MenuAction::SaveCurrent
                            } else {
                                MenuAction::SaveDialog
                            };
                            Popup::close_id(ctx, file_id);
                        }
                        if icon_menu_item(ui, ic::FILE_SAVE_AS, "Save As…", "Ctrl+Shift+S", t) {
                            action = MenuAction::SaveDialog;
                            Popup::close_id(ctx, file_id);
                        }

                        styled_separator(ui, t);

                        let flyout_was_hovered = ui
                            .ctx()
                            .data(|d| d.get_temp::<bool>(flyout_open_id).unwrap_or(false));

                        let (r_row, r_resp) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 28.0),
                            egui::Sense::hover(),
                        );
                        let row_hovered = r_resp.hovered();

                        let show_flyout = row_hovered || flyout_was_hovered;

                        if show_flyout {
                            ui.painter()
                                .rect_filled(r_row, egui::CornerRadius::same(4), t.accent);
                        }
                        let row_fg = if show_flyout { t.tab_bar_bg } else { t.menu_fg };
                        ui.painter().text(
                            egui::pos2(r_row.left() + 10.0, r_row.center().y),
                            egui::Align2::LEFT_CENTER,
                            format!("{}   Recent", ic::FILE_OPEN),
                            egui::FontId::proportional(13.5),
                            row_fg,
                        );
                        ui.painter().text(
                            egui::pos2(r_row.right() - 10.0, r_row.center().y),
                            egui::Align2::RIGHT_CENTER,
                            ic::CARET_RIGHT,
                            egui::FontId::proportional(13.0),
                            row_fg,
                        );

                        ui.ctx()
                            .data_mut(|d| d.insert_temp(flyout_row_rect_id, r_row));

                        if row_hovered {
                            ui.ctx().data_mut(|d| d.insert_temp(flyout_open_id, true));
                        }

                        styled_separator(ui, t);

                        if icon_menu_item(ui, ic::MAGNIFY, "Find…", "Ctrl+F", t) {
                            action = MenuAction::Search;
                            Popup::close_id(ctx, file_id);
                        }
                        if icon_menu_item(ui, ic::ARROWS_CLOCKWISE, "Replace…", "Ctrl+H", t) {
                            action = MenuAction::Replace;
                            Popup::close_id(ctx, file_id);
                        }

                        styled_separator(ui, t);

                        let docs_label = if docs_open { "Close Docs" } else { "Open Docs" };
                        if icon_menu_item(ui, ic::DOCS, docs_label, "Ctrl+D", t) {
                            action = MenuAction::ToggleDocs;
                            Popup::close_id(ctx, file_id);
                        }
                        ui.add_space(4.0);
                    });
                }

                let popup_is_open = Popup::is_id_open(ctx, file_id);
                let flyout_active =
                    ctx.data(|d| d.get_temp::<bool>(flyout_open_id).unwrap_or(false));
                let row_rect: Option<egui::Rect> = ctx.data(|d| d.get_temp(flyout_row_rect_id));

                if popup_is_open && flyout_active {
                    if let Some(r_row) = row_rect {
                        let flyout_pos = egui::pos2(r_row.right() + FLYOUT_GAP, r_row.top());
                        let recent_area_id = egui::Id::new("recent_flyout_area");

                        let area_resp = egui::Area::new(recent_area_id)
                            .order(egui::Order::Foreground)
                            .fixed_pos(flyout_pos)
                            .show(ctx, |ui| {
                                egui::Frame::new()
                                    .fill(t.menu_bg)
                                    .stroke(egui::Stroke::new(1.0, t.border))
                                    .corner_radius(egui::CornerRadius::same(6))
                                    .shadow(egui::Shadow {
                                        offset: [0, 4],
                                        blur: 12,
                                        spread: 0,
                                        color: egui::Color32::from_black_alpha(80),
                                    })
                                    .inner_margin(egui::Margin::same(4))
                                    .show(ui, |ui| {
                                        ui.set_min_width(260.0);
                                        ui.add_space(2.0);

                                        if recent_files.is_empty() {
                                            let (er, _) = ui.allocate_exact_size(
                                                egui::vec2(260.0, 28.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().text(
                                                egui::pos2(er.left() + 12.0, er.center().y),
                                                egui::Align2::LEFT_CENTER,
                                                "No recent files",
                                                egui::FontId::proportional(12.5),
                                                t.tab_inactive_fg,
                                            );
                                        } else {
                                            for path in recent_files.iter().take(10) {
                                                let name = path
                                                    .file_name()
                                                    .map(|n| n.to_string_lossy().to_string())
                                                    .unwrap_or_else(|| {
                                                        path.to_string_lossy().to_string()
                                                    });
                                                let full = path.to_string_lossy().to_string();
                                                let short = if full.len() > 36 {
                                                    format!("…{}", &full[full.len() - 35..])
                                                } else {
                                                    full.clone()
                                                };

                                                let (ir, ir_resp) = ui.allocate_exact_size(
                                                    egui::vec2(260.0, 28.0),
                                                    egui::Sense::click(),
                                                );
                                                let ih = ir_resp.hovered();
                                                if ih {
                                                    ui.painter().rect_filled(
                                                        ir,
                                                        egui::CornerRadius::same(4),
                                                        t.accent,
                                                    );
                                                }
                                                let ifg = if ih { t.tab_bar_bg } else { t.menu_fg };
                                                let ihint = if ih {
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
                                                ui.painter().text(
                                                    egui::pos2(ir.left() + 12.0, ir.center().y),
                                                    egui::Align2::LEFT_CENTER,
                                                    &name,
                                                    egui::FontId::proportional(13.0),
                                                    ifg,
                                                );
                                                ui.painter().text(
                                                    egui::pos2(ir.right() - 10.0, ir.center().y),
                                                    egui::Align2::RIGHT_CENTER,
                                                    &short,
                                                    egui::FontId::proportional(10.5),
                                                    ihint,
                                                );

                                                if ir_resp.clicked() {
                                                    ctx.data_mut(|d| {
                                                        d.insert_temp(
                                                            recent_clicked_id,
                                                            path.clone(),
                                                        )
                                                    });
                                                    ctx.data_mut(|d| {
                                                        d.insert_temp(flyout_open_id, false)
                                                    });
                                                    Popup::close_id(ctx, file_id);
                                                }
                                            }
                                        }
                                        ui.add_space(2.0);
                                    });
                            });

                        let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
                        let flyout_window_rect = area_resp.response.rect;

                        let bridge = egui::Rect::from_min_max(
                            egui::pos2(r_row.right(), r_row.top()),
                            egui::pos2(flyout_window_rect.left(), r_row.bottom()),
                        );

                        let still_active = pointer_pos.map_or(false, |p| {
                            flyout_window_rect.contains(p)
                                || bridge.contains(p)
                                || r_row.contains(p)
                        });

                        ctx.data_mut(|d| d.insert_temp(flyout_open_id, still_active));
                    }
                } else if !popup_is_open {
                    ctx.data_mut(|d| d.insert_temp(flyout_open_id, false));
                }

                if matches!(action, MenuAction::None) {
                    let deferred: Option<PathBuf> =
                        ctx.data_mut(|d| d.remove_temp(recent_clicked_id));
                    if let Some(path) = deferred {
                        action = MenuAction::OpenRecent(path);
                    }
                }

                ui.add_space(4.0);

                let run_label = if is_running { "Running…" } else { "Run" };
                let run_icon = if is_running { ic::RUNNING } else { ic::RUN };
                let run_id = egui::Id::new("menu_run_btn");
                let (run_rect, _) =
                    ui.allocate_exact_size(egui::vec2(BTN_W, BTN_H), egui::Sense::hover());
                let run_hovered = ui.rect_contains_pointer(run_rect) && !is_running;

                paint_run_button(
                    ui,
                    run_rect,
                    run_icon,
                    run_label,
                    is_running,
                    run_hovered,
                    t,
                );

                let run_resp = ui.interact(run_rect, run_id, egui::Sense::click());
                if run_resp.clicked() && !is_running {
                    action = MenuAction::Run;
                }

                ui.add_space(6.0);

                let step_label = if is_debugging {
                    "Step  F5"
                } else {
                    "Debug  F5"
                };
                let step_id = egui::Id::new("menu_step_btn");
                let (step_rect, _) =
                    ui.allocate_exact_size(egui::vec2(BTN_W + 14.0, BTN_H), egui::Sense::hover());
                let step_hovered = ui.rect_contains_pointer(step_rect);

                paint_step_button(ui, step_rect, step_label, is_debugging, step_hovered, t);

                let step_resp = ui.interact(step_rect, step_id, egui::Sense::click());
                if step_resp.clicked() {
                    action = MenuAction::StepRun;
                }

                if is_debugging {
                    ui.add_space(4.0);
                    let stop_id = egui::Id::new("menu_stop_btn");
                    let (stop_rect, _) =
                        ui.allocate_exact_size(egui::vec2(BTN_W, BTN_H), egui::Sense::hover());
                    let stop_hovered = ui.rect_contains_pointer(stop_rect);

                    if stop_hovered {
                        ui.painter().rect_filled(
                            stop_rect,
                            egui::CornerRadius::same(BTN_ROUNDING as u8),
                            t.terminal_error,
                        );
                    }
                    let stop_fg = if stop_hovered {
                        t.tab_bar_bg
                    } else {
                        t.terminal_error
                    };
                    ui.painter().text(
                        stop_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "■  Stop",
                        egui::FontId::proportional(12.5),
                        stop_fg,
                    );
                    if ui
                        .interact(stop_rect, stop_id, egui::Sense::click())
                        .clicked()
                    {
                        action = MenuAction::StepStop;
                    }
                }

                ui.add_space(6.0);

                let (div_rect2, _) =
                    ui.allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(div_rect2, egui::CornerRadius::ZERO, t.border);
                ui.add_space(6.0);

                let view_id = ui.make_persistent_id("menu_view_popup");
                let (view_rect, _) =
                    ui.allocate_exact_size(egui::vec2(BTN_W, BTN_H), egui::Sense::hover());

                let view_popup_open = Popup::is_id_open(ctx, view_id);
                let view_clicked = ui
                    .interact(view_rect, view_id.with("btn"), egui::Sense::click())
                    .clicked();
                if view_clicked {
                    Popup::toggle_id(ctx, view_id);
                }

                let view_hovered = ui.rect_contains_pointer(view_rect);
                let view_active = view_hovered || view_popup_open;

                paint_menu_button(ui, view_rect, "View", ic::SETTINGS, view_active, false, t);

                if view_popup_open {
                    Popup::new(
                        view_id,
                        ctx.clone(),
                        PopupAnchor::ParentRect(view_rect),
                        LayerId::new(Order::Foreground, view_id),
                    )
                    .open_memory(None)
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .show(|ui| {
                        let s = ui.style_mut();
                        s.visuals.window_fill = t.menu_bg;
                        s.visuals.override_text_color = Some(t.menu_fg);
                        s.visuals.widgets.hovered.bg_fill = t.menu_hover_bg;
                        ui.set_min_width(220.0);
                        ui.add_space(4.0);

                        let tree_label = if tree_view_open {
                            "✓  Abstract Syntax Tree"
                        } else {
                            "   Abstract Syntax Tree"
                        };
                        if icon_menu_item(ui, "", tree_label, "", t) {
                            action = MenuAction::ToggleTreeView;
                            Popup::close_id(ctx, view_id);
                        }
                        let var_label = if var_view_open {
                            "✓  Variable State"
                        } else {
                            "   Variable State"
                        };
                        if icon_menu_item(ui, "", var_label, "", t) {
                            action = MenuAction::ToggleVarView;
                            Popup::close_id(ctx, view_id);
                        }
                        ui.add_space(4.0);
                    });
                }

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
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(BTN_ROUNDING as u8),
            t.button_hover_bg,
        );
        ui.painter().rect_stroke(
            rect,
            egui::CornerRadius::same(BTN_ROUNDING as u8),
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(
                    t.border.r(),
                    t.border.g(),
                    t.border.b(),
                    180,
                ),
            ),
            egui::StrokeKind::Outside,
        );
    }
    let fg = if active {
        t.tab_active_fg
    } else {
        t.tab_inactive_fg
    };
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
    let rounding = egui::CornerRadius::same(BTN_ROUNDING as u8);
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
            egui::StrokeKind::Outside,
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
    let rounding = egui::CornerRadius::same(BTN_ROUNDING as u8);
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
            egui::StrokeKind::Outside,
        );
    }
    let fg = if active {
        t.tab_active_fg
    } else {
        t.tab_inactive_fg
    };
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
        .rect_filled(sep_rect, egui::CornerRadius::ZERO, t.border);
    ui.add_space(2.0);
}

fn icon_menu_item(ui: &mut egui::Ui, icon: &str, label: &str, shortcut: &str, t: &Theme) -> bool {
    let desired_size = egui::vec2(ui.available_width(), 28.0);
    let (rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(4), t.accent);
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

fn paint_step_button(
    ui: &egui::Ui,
    rect: egui::Rect,
    label: &str,
    is_active: bool,
    hovered: bool,
    t: &Theme,
) {
    let rounding = egui::CornerRadius::same(BTN_ROUNDING as u8);
    if is_active {
        ui.painter().rect_filled(
            rect,
            rounding,
            egui::Color32::from_rgba_premultiplied(210, 153, 34, 30),
        );
        ui.painter().rect_stroke(
            rect,
            rounding,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_premultiplied(210, 153, 34, 120),
            ),
            egui::StrokeKind::Outside,
        );
    } else if hovered {
        ui.painter().rect_filled(
            rect,
            rounding,
            egui::Color32::from_rgba_premultiplied(210, 153, 34, 220),
        );
    }
    let fg = if hovered && !is_active {
        t.tab_bar_bg
    } else if is_active {
        egui::Color32::from_rgb(210, 153, 34)
    } else {
        egui::Color32::from_rgb(210, 153, 34)
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(12.5),
        fg,
    );
}
