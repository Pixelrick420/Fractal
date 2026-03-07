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

        ctx.style_mut(|s| {
            s.visuals.window_fill = t.panel_bg;
            s.visuals.panel_fill = t.panel_bg;
            s.visuals.override_text_color = Some(t.menu_fg);
            s.visuals.window_stroke = egui::Stroke::new(1.0, t.border);

            s.visuals.widgets.noninteractive.bg_fill = t.panel_bg;
            s.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
            s.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, t.border);

            s.visuals.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            s.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, t.menu_fg);
            s.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, t.border);

            s.visuals.widgets.hovered.bg_fill = t.button_hover_bg;
            s.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, t.tab_active_fg);
            s.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, t.accent);

            s.visuals.widgets.active.bg_fill = t.accent;
            s.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
            s.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, t.accent);

            s.visuals.widgets.open.bg_fill = t.button_hover_bg;
            s.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, t.accent);

            s.visuals.extreme_bg_color = t.button_bg;
            s.visuals.faint_bg_color = t.panel_bg;
            s.visuals.selection.bg_fill = t.selection;
            s.visuals.selection.stroke = egui::Stroke::new(1.0, t.accent);
        });

        let title = match self.mode {
            FileDialogMode::Open => "Open File",
            FileDialogMode::Save => "Save File",
        };

        let mut open = true;
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([600.0, 420.0])
            .min_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("⬆  Up").size(12.5).color(t.menu_fg),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, t.border)),
                        )
                        .clicked()
                    {
                        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
                            self.navigate_to(parent);
                        }
                    }
                    ui.separator();

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
                    for part in &parts {
                        let name = part
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| part.to_string_lossy().to_string());
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(&name).size(12.5).color(t.menu_fg),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            )
                            .clicked()
                        {
                            self.navigate_to(part.clone());
                        }
                        ui.label(egui::RichText::new("/").size(12.5).color(t.tab_inactive_fg));
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let entries = self.entries.clone();
                        for entry in &entries {
                            let icon = if entry.is_dir { "📁" } else { "📄" };
                            let label = format!("{}  {}", icon, entry.name);
                            let selected = self.selected.as_ref() == Some(&entry.path);

                            let resp = ui.selectable_label(selected, &label);

                            if resp.double_clicked() {
                                if entry.is_dir {
                                    self.navigate_to(entry.path.clone());
                                } else if self.mode == FileDialogMode::Open {
                                    self.result = Some(FileDialogResult {
                                        path: entry.path.clone(),
                                        mode: FileDialogMode::Open,
                                    });
                                    self.visible = false;
                                }
                            } else if resp.clicked() {
                                self.selected = Some(entry.path.clone());
                                if !entry.is_dir {
                                    self.filename_input = entry.name.clone();
                                }
                            }
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("File name:")
                            .size(13.0)
                            .color(t.menu_fg),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filename_input).text_color(t.menu_fg),
                    );
                });

                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    let confirm_label = match self.mode {
                        FileDialogMode::Open => "Open",
                        FileDialogMode::Save => "Save",
                    };

                    let can_confirm = match self.mode {
                        FileDialogMode::Open => {
                            self.selected.as_ref().map(|p| p.is_file()).unwrap_or(false)
                                || (!self.filename_input.is_empty()
                                    && self.current_dir.join(&self.filename_input).is_file())
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
                            .stroke(egui::Stroke::NONE),
                        )
                        .clicked()
                    {
                        let path = if self.mode == FileDialogMode::Open {
                            self.selected
                                .clone()
                                .unwrap_or_else(|| self.current_dir.join(&self.filename_input))
                        } else {
                            self.current_dir.join(&self.filename_input)
                        };
                        self.result = Some(FileDialogResult {
                            path,
                            mode: self.mode,
                        });
                        self.visible = false;
                    }

                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("Cancel").size(13.0).color(t.menu_fg),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, t.border)),
                        )
                        .clicked()
                    {
                        self.visible = false;
                    }
                });
            });

        if !open {
            self.visible = false;
        }
    }
}
