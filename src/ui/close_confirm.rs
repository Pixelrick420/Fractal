use eframe::egui;

// ── Action returned each frame ────────────────────────────────────────────────

pub enum CloseConfirmAction {
    /// Save this tab index, then close it.
    Save(usize),
    /// Close this tab without saving.
    Discard(usize),
    /// User cancelled — do nothing.
    Cancel,
    /// Dialog not visible or no decision yet.
    Pending,
}

// ── Single-tab close dialog ───────────────────────────────────────────────────

pub struct CloseConfirmDialog {
    pub visible: bool,
    tab_index: usize,
    tab_name: String,
}

impl CloseConfirmDialog {
    pub fn new() -> Self {
        Self { visible: false, tab_index: 0, tab_name: String::new() }
    }

    pub fn open(&mut self, tab_index: usize, tab_name: String) {
        self.tab_index = tab_index;
        self.tab_name = tab_name;
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> CloseConfirmAction {
        if !self.visible {
            return CloseConfirmAction::Pending;
        }

        let mut action = CloseConfirmAction::Pending;
        let mut window_open = true;

        egui::Window::new("Unsaved Changes")
            .open(&mut window_open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.label(format!("\"{}\" has unsaved changes.", self.tab_name));
                ui.label("What would you like to do?");
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button("  💾 Save  ").clicked() {
                        action = CloseConfirmAction::Save(self.tab_index);
                    }
                    ui.add_space(4.0);
                    if ui.button("  🗑 Discard  ").clicked() {
                        action = CloseConfirmAction::Discard(self.tab_index);
                    }
                    ui.add_space(4.0);
                    if ui.button("  Cancel  ").clicked() {
                        action = CloseConfirmAction::Cancel;
                    }
                });
                ui.add_space(4.0);
            });

        if !window_open {
            action = CloseConfirmAction::Cancel;
        }

        if !matches!(action, CloseConfirmAction::Pending) {
            self.visible = false;
        }

        action
    }
}

// ── Quit-with-dirty-tabs dialog ─────────────────────────────────────────────

pub enum QuitConfirmAction {
    /// Quit without saving — shut down immediately.
    Discard,
    /// User chose to keep working — close the popup.
    Keep,
    /// Dialog not visible or no decision yet.
    Pending,
}

pub struct QuitConfirmDialog {
    pub visible: bool,
    dirty_names: Vec<String>,
}

impl QuitConfirmDialog {
    pub fn new() -> Self {
        Self { visible: false, dirty_names: Vec::new() }
    }

    pub fn open(&mut self, dirty_names: Vec<String>) {
        self.dirty_names = dirty_names;
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> QuitConfirmAction {
        if !self.visible {
            return QuitConfirmAction::Pending;
        }

        let mut action = QuitConfirmAction::Pending;
        let mut window_open = true;

        egui::Window::new("Unsaved Changes")
            .open(&mut window_open)
            .collapsible(false)
            .resizable(false)
            .movable(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.add_space(6.0);

                if self.dirty_names.len() == 1 {
                    ui.label(format!(
                        "\"{}\" has unsaved changes.",
                        self.dirty_names[0]
                    ));
                } else {
                    ui.label("The following files have unsaved changes:");
                    ui.add_space(4.0);
                    for name in &self.dirty_names {
                        ui.label(format!("  • {name}"));
                    }
                }

                ui.add_space(14.0);

                ui.horizontal(|ui| {
                    if ui.button("  ✏ Continue Work  ").clicked() {
                        action = QuitConfirmAction::Keep;
                    }
                    ui.add_space(4.0);
                    if ui.button("  🗑 Discard Changes  ").clicked() {
                        action = QuitConfirmAction::Discard;
                    }
                    ui.add_space(4.0);
                    if ui.button("  Cancel  ").clicked() {
                        action = QuitConfirmAction::Keep;
                    }
                });
                ui.add_space(4.0);
            });

        // Title-bar X also keeps working.
        if !window_open {
            action = QuitConfirmAction::Keep;
        }

        if !matches!(action, QuitConfirmAction::Pending) {
            self.visible = false;
        }

        action
    }
}