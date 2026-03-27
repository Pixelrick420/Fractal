use crate::ui::icons as ic;
use crate::ui::theme::{Theme, ThemeVariant};
use eframe::egui;

pub enum CloseConfirmAction {
    Save(usize),
    Discard(usize),
    Cancel,
    Pending,
}

pub struct CloseConfirmDialog {
    pub visible: bool,
    tab_index: usize,
    tab_name: String,
}

impl CloseConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            tab_index: 0,
            tab_name: String::new(),
        }
    }

    pub fn open(&mut self, tab_index: usize, tab_name: String) {
        self.tab_index = tab_index;
        self.tab_name = tab_name;
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context, theme: &Theme) -> CloseConfirmAction {
        if !self.visible {
            return CloseConfirmAction::Pending;
        }

        let mut action = CloseConfirmAction::Pending;
        let mut open = true;

        egui::Window::new("Unsaved Changes")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .default_size([320.0, 130.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
        egui::Frame::window(&ctx.style()).fill(
            if theme.variant == ThemeVariant::Light {
                egui::Color32::from_rgb(240, 242, 248) // ✅ light panel bg
            } else {
                theme.panel_bg // ✅ your existing dark bg (unchanged)
            }
        )
    )
            .show(ctx, |ui| {
                ui.add_space(6.0);

                // TEXT
                if theme.variant == ThemeVariant::Light {
                    ui.label(
                        egui::RichText::new(format!(
                            "\"{}\" has unsaved changes.",
                            self.tab_name
                        ))
                        .size(13.0)
                        .color(egui::Color32::from_rgb(20, 25, 40)),
                    );
                } else {
                    // DARK untouched
                    ui.label(
                        egui::RichText::new(format!(
                            "\"{}\" has unsaved changes.",
                            self.tab_name
                        ))
                        .size(13.0),
                    );
                }

                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    if theme.variant == ThemeVariant::Light {
                        // LIGHT BUTTONS
                        let btn = |label: String| {
                            egui::Button::new(
                                egui::RichText::new(label)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(20, 25, 40)),
                            )
                            .fill(egui::Color32::from_rgb(210, 214, 230))
                            .min_size(egui::vec2(84.0, 28.0))
                        };

                        if ui.add(btn(format!("{}  Save", ic::SAVE_ACTION))).clicked() {
                            action = CloseConfirmAction::Save(self.tab_index);
                        }

                        ui.add_space(4.0);

                        if ui.add(btn(format!("{}  Discard", ic::DISCARD))).clicked() {
                            action = CloseConfirmAction::Discard(self.tab_index);
                        }

                        ui.add_space(4.0);

                        if ui.add(btn(format!("{}  Cancel", ic::CANCEL))).clicked() {
                            action = CloseConfirmAction::Cancel;
                        }
                    } else {
                        // DARK ORIGINAL
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{}  Save",
                                        ic::SAVE_ACTION
                                    ))
                                    .size(13.0),
                                )
                                .min_size(egui::vec2(84.0, 28.0)),
                            )
                            .clicked()
                        {
                            action = CloseConfirmAction::Save(self.tab_index);
                        }

                        ui.add_space(4.0);

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{}  Discard",
                                        ic::DISCARD
                                    ))
                                    .size(13.0),
                                )
                                .min_size(egui::vec2(84.0, 28.0)),
                            )
                            .clicked()
                        {
                            action = CloseConfirmAction::Discard(self.tab_index);
                        }

                        ui.add_space(4.0);

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(format!(
                                        "{}  Cancel",
                                        ic::CANCEL
                                    ))
                                    .size(13.0),
                                )
                                .min_size(egui::vec2(76.0, 28.0)),
                            )
                            .clicked()
                        {
                            action = CloseConfirmAction::Cancel;
                        }
                    }
                });

                ui.add_space(4.0);
            });

        if !open {
            action = CloseConfirmAction::Cancel;
        }

        if !matches!(action, CloseConfirmAction::Pending) {
            self.visible = false;
        }

        action
    }
}

pub enum QuitConfirmAction {
    Discard,
    Keep,
    Pending,
}

pub struct QuitConfirmDialog {
    pub visible: bool,
    dirty_names: Vec<String>,
}

impl QuitConfirmDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            dirty_names: Vec::new(),
        }
    }

    pub fn open(&mut self, dirty_names: Vec<String>) {
        self.dirty_names = dirty_names;
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context, theme: &Theme) -> QuitConfirmAction {
        if !self.visible {
            return QuitConfirmAction::Pending;
        }

        let mut action = QuitConfirmAction::Pending;
        let mut open = true;

        egui::Window::new("Unsaved Changes")
    .open(&mut open)
    .collapsible(false)
    .resizable(false)
    .movable(true)
    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
    .frame(
        egui::Frame::window(&ctx.style())
            .fill(theme.panel_bg) // ✅ outer window bg (auto for light/dark)
    )
    .show(ctx, |ui| {

        egui::Frame::new()
            .fill(
                if theme.variant == ThemeVariant::Light {
                    egui::Color32::from_rgb(240, 242, 248) // ✅ light inner bg
                } else {
                    egui::Color32::TRANSPARENT // ✅ dark untouched
                }
            )
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {

                ui.add_space(6.0);

                // ─── TEXT ─────────────────────────────
                if self.dirty_names.len() == 1 {
                    let text =
                        format!("\"{}\" has unsaved changes.", self.dirty_names[0]);

                    if theme.variant == ThemeVariant::Light {
                        ui.label(
                            egui::RichText::new(text)
                                .color(egui::Color32::from_rgb(20, 25, 40)),
                        );
                    } else {
                        ui.label(text);
                    }
                } else {
                    if theme.variant == ThemeVariant::Light {
                        ui.label(
                            egui::RichText::new(
                                "The following files have unsaved changes:",
                            )
                            .color(egui::Color32::from_rgb(20, 25, 40)),
                        );
                    } else {
                        ui.label("The following files have unsaved changes:");
                    }

                    for name in &self.dirty_names {
                        if theme.variant == ThemeVariant::Light {
                            ui.label(
                                egui::RichText::new(format!("  {name}"))
                                    .color(egui::Color32::from_rgb(20, 25, 40)),
                            );
                        } else {
                            ui.label(format!("  {name}"));
                        }
                    }
                }

                ui.add_space(14.0);

                // ─── BUTTONS ──────────────────────────
                ui.horizontal(|ui| {
                    if theme.variant == ThemeVariant::Light {
                        let btn = |label: String| {
                            egui::Button::new(
                                egui::RichText::new(label)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(20, 25, 40)),
                            )
                            .fill(egui::Color32::from_rgb(210, 214, 230))
                            .min_size(egui::vec2(130.0, 28.0))
                        };

                        if ui.add(btn("  Continue editing  ".to_string())).clicked() {
                            action = QuitConfirmAction::Keep;
                        }

                        ui.add_space(4.0);

                        if ui
                            .add(btn(format!("{}  Discard & quit", ic::DISCARD)))
                            .clicked()
                        {
                            action = QuitConfirmAction::Discard;
                        }
                    } else {
                        // ✅ DARK unchanged
                        if ui
                            .add(
                                egui::Button::new("  Continue editing  ")
                                    .min_size(egui::vec2(130.0, 28.0)),
                            )
                            .clicked()
                        {
                            action = QuitConfirmAction::Keep;
                        }

                        ui.add_space(4.0);

                        if ui
                            .add(
                                egui::Button::new(
                                    format!("{}  Discard & quit", ic::DISCARD),
                                )
                                .min_size(egui::vec2(130.0, 28.0)),
                            )
                            .clicked()
                        {
                            action = QuitConfirmAction::Discard;
                        }
                    }
                });

                ui.add_space(4.0);
            });
    });

if !open {
    action = QuitConfirmAction::Keep;
}

if !matches!(action, QuitConfirmAction::Pending) {
    self.visible = false;
}

action
    }}