use crate::ui::theme::Theme;
use eframe::egui;
use std::fs;
use std::path::PathBuf;

#[derive(PartialEq, Clone, Copy)]
pub enum FileDialogMode {
    Open,
    Save,
}

pub struct FileDialog {
    pub visible: bool,
    pub mode: FileDialogMode,
    current_dir: PathBuf,
    entries: Vec<DirEntry>,
    selected: Option<PathBuf>,
    filename_input: String,
    pub result: Option<FileDialogResult>,
    theme: Theme,
}

pub struct FileDialogResult {
    pub path: PathBuf,
    pub mode: FileDialogMode,
}

#[derive(Clone)]
struct DirEntry {
    path: PathBuf,
    name: String,
    is_dir: bool,
}

impl FileDialog {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let mut dialog = Self {
            visible: false,
            mode: FileDialogMode::Open,
            current_dir: cwd.clone(),
            entries: Vec::new(),
            selected: None,
            filename_input: String::new(),
            result: None,
            theme: Theme::default(),
        };
        dialog.refresh_entries();
        dialog
    }

    pub fn update_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    pub fn open_for_open(&mut self) {
        self.mode = FileDialogMode::Open;
        self.selected = None;
        self.filename_input.clear();
        self.refresh_entries();
        self.visible = true;
        self.result = None;
    }

    pub fn open_for_save(&mut self, suggested_name: &str) {
        self.mode = FileDialogMode::Save;
        self.selected = None;
        self.filename_input = suggested_name.to_string();
        self.refresh_entries();
        self.visible = true;
        self.result = None;
    }

    fn refresh_entries(&mut self) {
        self.entries.clear();
        if let Ok(rd) = fs::read_dir(&self.current_dir) {
            let mut entries: Vec<DirEntry> = rd
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    let name = e.file_name().to_string_lossy().to_string();
                    let is_dir = path.is_dir();
                    DirEntry { path, name, is_dir }
                })
                .collect();
            entries.sort_by(|a, b| {
                b.is_dir
                    .cmp(&a.is_dir)
                    .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            });
            self.entries = entries;
        }
    }

    fn navigate_to(&mut self, path: PathBuf) {
        self.current_dir = path;
        self.selected = None;
        self.refresh_entries();
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        let t = self.theme;

        let is_dark = {
            let c = t.panel_bg;
            let lum = 0.299 * c.r() as f32 + 0.587 * c.g() as f32 + 0.114 * c.b() as f32;
            lum < 128.0
        };

        let hover_bg = if is_dark {
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18)
        } else {
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 14)
        };
        let selected_bg = egui::Color32::from_rgba_unmultiplied(
            t.accent.r(),
            t.accent.g(),
            t.accent.b(),
            if is_dark { 55 } else { 35 },
        );

        let title = match self.mode {
            FileDialogMode::Open => "Open File",
            FileDialogMode::Save => "Save File",
        };

        let mut close_requested = false;

        let window_frame = egui::Frame {
            fill: t.panel_bg,
            stroke: egui::Stroke::new(1.0, t.border),
            rounding: egui::Rounding::same(10.0),
            inner_margin: egui::Margin::same(0.0),
            outer_margin: egui::Margin::same(0.0),
            shadow: egui::epaint::Shadow {
                offset: egui::vec2(0.0, 8.0),
                blur: 32.0,
                spread: 0.0,
                color: egui::Color32::from_black_alpha(80),
            },
        };

        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 440.0])
            .min_size([420.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(window_frame)
            .title_bar(false)
            .show(ctx, |ui| {
                {
                    let s = ui.style_mut();
                    s.visuals.override_text_color = Some(t.menu_fg);
                    s.visuals.widgets.noninteractive.bg_fill = t.panel_bg;
                    s.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
                    s.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, t.border);
                    s.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                    s.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
                    s.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    s.visuals.widgets.hovered.bg_fill = hover_bg;
                    s.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
                    s.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                    s.visuals.widgets.active.bg_fill = selected_bg;
                    s.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
                    s.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                    s.visuals.widgets.open.bg_fill = selected_bg;
                    s.visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
                    s.visuals.extreme_bg_color = t.button_bg;
                    s.visuals.faint_bg_color = t.panel_bg;
                    s.visuals.selection.bg_fill = selected_bg;
                    s.visuals.selection.stroke = egui::Stroke::new(1.0, t.accent);
                    s.spacing.item_spacing = egui::vec2(6.0, 2.0);
                }

                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: 16.0,
                        right: 10.0,
                        top: 13.0,
                        bottom: 11.0,
                    })
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(title)
                                    .size(14.0)
                                    .color(t.menu_fg)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let close_id = ui.make_persistent_id("dlg_close");
                                    let (cr, _) = ui.allocate_exact_size(
                                        egui::vec2(24.0, 24.0),
                                        egui::Sense::hover(),
                                    );
                                    let close_hovered = ui.rect_contains_pointer(cr);

                                    let close_hover_bg = egui::Color32::from_rgba_unmultiplied(
                                        220,
                                        60,
                                        60,
                                        if is_dark { 180 } else { 160 },
                                    );
                                    if close_hovered {
                                        ui.painter().rect_filled(
                                            cr,
                                            egui::Rounding::same(5.0),
                                            close_hover_bg,
                                        );
                                    }
                                    ui.painter().text(
                                        cr.center(),
                                        egui::Align2::CENTER_CENTER,
                                        egui_phosphor::regular::X,
                                        egui::FontId::proportional(14.0),
                                        if close_hovered {
                                            egui::Color32::WHITE
                                        } else {
                                            t.tab_inactive_fg
                                        },
                                    );
                                    if ui.interact(cr, close_id, egui::Sense::click()).clicked() {
                                        close_requested = true;
                                    }
                                },
                            );
                        });
                    });

                let (div, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(div, egui::Rounding::ZERO, t.border);

                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: 14.0,
                        right: 14.0,
                        top: 10.0,
                        bottom: 14.0,
                    })
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let up_id = ui.make_persistent_id("dlg_up");
                            let (up_rect, _) = ui
                                .allocate_exact_size(egui::vec2(26.0, 22.0), egui::Sense::hover());
                            let up_hovered = ui.rect_contains_pointer(up_rect);
                            if up_hovered {
                                ui.painter().rect_filled(
                                    up_rect,
                                    egui::Rounding::same(4.0),
                                    hover_bg,
                                );
                            }
                            ui.painter().text(
                                up_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                egui_phosphor::regular::ARROW_UP,
                                egui::FontId::proportional(15.0),
                                if up_hovered {
                                    t.menu_fg
                                } else {
                                    t.tab_inactive_fg
                                },
                            );
                            if ui.interact(up_rect, up_id, egui::Sense::click()).clicked() {
                                if let Some(parent) =
                                    self.current_dir.parent().map(|p| p.to_path_buf())
                                {
                                    self.navigate_to(parent);
                                }
                            }

                            let (vr, _) =
                                ui.allocate_exact_size(egui::vec2(1.0, 14.0), egui::Sense::hover());
                            ui.painter().rect_filled(vr, egui::Rounding::ZERO, t.border);
                            ui.add_space(2.0);

                            let parts: Vec<PathBuf> = {
                                let mut v = Vec::new();
                                let mut p = self.current_dir.clone();
                                loop {
                                    v.push(p.clone());
                                    if !p.pop() {
                                        break;
                                    }
                                }
                                v.reverse();
                                v
                            };
                            for (i, part) in parts.iter().enumerate() {
                                let name = part
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| part.to_string_lossy().to_string());
                                let is_last = i == parts.len() - 1;
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new(&name).size(12.0).color(
                                                if is_last {
                                                    t.menu_fg
                                                } else {
                                                    t.tab_inactive_fg
                                                },
                                            ),
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE),
                                    )
                                    .clicked()
                                {
                                    self.navigate_to(part.clone());
                                }
                                if !is_last {
                                    ui.label(
                                        egui::RichText::new("›")
                                            .size(13.0)
                                            .color(t.tab_inactive_fg),
                                    );
                                }
                            }
                        });

                        ui.add_space(8.0);

                        egui::Frame::none()
                            .fill(t.panel_bg)
                            .stroke(egui::Stroke::new(1.0, t.border))
                            .rounding(egui::Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(4.0))
                            .show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(250.0)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        let entries = self.entries.clone();
                                        for entry in &entries {
                                            let icon = if entry.is_dir { "📁" } else { "📄" };
                                            let label = format!("{}  {}", icon, entry.name);
                                            let is_selected =
                                                self.selected.as_ref() == Some(&entry.path);

                                            let (row_rect, row_resp) = ui.allocate_exact_size(
                                                egui::vec2(ui.available_width(), 26.0),
                                                egui::Sense::click(),
                                            );

                                            if is_selected {
                                                ui.painter().rect_filled(
                                                    row_rect,
                                                    egui::Rounding::same(4.0),
                                                    selected_bg,
                                                );
                                            } else if row_resp.hovered() {
                                                ui.painter().rect_filled(
                                                    row_rect,
                                                    egui::Rounding::same(4.0),
                                                    hover_bg,
                                                );
                                            }

                                            ui.painter().text(
                                                egui::pos2(
                                                    row_rect.left() + 6.0,
                                                    row_rect.center().y,
                                                ),
                                                egui::Align2::LEFT_CENTER,
                                                &label,
                                                egui::FontId::proportional(13.0),
                                                t.menu_fg,
                                            );

                                            if row_resp.double_clicked() {
                                                if entry.is_dir {
                                                    self.navigate_to(entry.path.clone());
                                                } else if self.mode == FileDialogMode::Open {
                                                    self.result = Some(FileDialogResult {
                                                        path: entry.path.clone(),
                                                        mode: FileDialogMode::Open,
                                                    });
                                                    self.visible = false;
                                                }
                                            } else if row_resp.clicked() {
                                                self.selected = Some(entry.path.clone());
                                                if !entry.is_dir {
                                                    self.filename_input = entry.name.clone();
                                                }
                                            }
                                        }
                                    });
                            });

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Name")
                                    .size(12.5)
                                    .color(t.tab_inactive_fg),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut self.filename_input)
                                    .font(egui::FontId::proportional(13.0))
                                    .text_color(t.menu_fg)
                                    .desired_width(f32::INFINITY),
                            );
                        });

                        ui.add_space(12.0);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Cancel")
                                            .size(13.0)
                                            .color(t.tab_inactive_fg),
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::new(1.0, t.border))
                                    .min_size(egui::vec2(80.0, 32.0)),
                                )
                                .clicked()
                            {
                                self.visible = false;
                            }

                            ui.add_space(6.0);

                            let confirm_label = match self.mode {
                                FileDialogMode::Open => "Open",
                                FileDialogMode::Save => "Save",
                            };
                            let can_confirm = match self.mode {
                                FileDialogMode::Open => {
                                    self.selected.as_ref().map(|p| p.is_file()).unwrap_or(false)
                                        || (!self.filename_input.is_empty()
                                            && self
                                                .current_dir
                                                .join(&self.filename_input)
                                                .is_file())
                                }
                                FileDialogMode::Save => !self.filename_input.is_empty(),
                            };
                            if ui
                                .add_enabled(
                                    can_confirm,
                                    egui::Button::new(
                                        egui::RichText::new(confirm_label)
                                            .size(13.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(t.accent)
                                    .stroke(egui::Stroke::NONE)
                                    .min_size(egui::vec2(80.0, 32.0)),
                                )
                                .clicked()
                            {
                                let path = if self.mode == FileDialogMode::Open {
                                    self.selected.clone().unwrap_or_else(|| {
                                        self.current_dir.join(&self.filename_input)
                                    })
                                } else {
                                    self.current_dir.join(&self.filename_input)
                                };
                                self.result = Some(FileDialogResult {
                                    path,
                                    mode: self.mode,
                                });
                                self.visible = false;
                            }
                        });
                    });
            });

        if close_requested {
            self.visible = false;
        }
    }
}
