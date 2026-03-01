use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use fractal::ui::close_confirm::{
    CloseConfirmAction, CloseConfirmDialog,
    QuitConfirmAction, QuitConfirmDialog,
};
use fractal::ui::docs::DocsPanel;
use fractal::ui::editor::CodeEditor;
use fractal::ui::file_dialog::{FileDialog, FileDialogMode};
use fractal::ui::formatter::format_code;
use fractal::ui::menu_bar::{show_menu_bar, MenuAction, MenuBarState};
use fractal::ui::terminal::Terminal;
use fractal::ui::theme::Theme;

const AUTOSAVE_INTERVAL_SECS: u64 = 120;

// ── Tab ───────────────────────────────────────────────────────────────────────

struct Tab {
    code: String,
    last_saved_code: String,
    current_file: Option<PathBuf>,
    editor: CodeEditor,
    output_rx: Option<Arc<Mutex<Vec<String>>>>,
    is_running: bool,
}

impl Tab {
    fn new(theme: Theme) -> Self {
        let code = String::from("!start\n# code here\n!end\n");
        Self {
            last_saved_code: code.clone(),
            code,
            current_file: None,
            editor: CodeEditor::new(theme),
            output_rx: None,
            is_running: false,
        }
    }

    fn from_file(path: PathBuf, content: String, theme: Theme) -> Self {
        Self {
            last_saved_code: content.clone(),
            code: content,
            current_file: Some(path),
            editor: CodeEditor::new(theme),
            output_rx: None,
            is_running: false,
        }
    }

    fn is_dirty(&self) -> bool {
        self.code != self.last_saved_code
    }

    fn display_name(&self) -> String {
        self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn is_pristine_new(&self) -> bool {
        self.current_file.is_none() && !self.is_dirty()
    }
}

// ── FractalEditor ─────────────────────────────────────────────────────────────

struct FractalEditor {
    tabs: Vec<Tab>,
    active_tab: usize,
    theme: Theme,
    menu_state: MenuBarState,
    terminal: Terminal,
    file_dialog: FileDialog,
    docs_panel: DocsPanel,
    docs_open: bool,
    /// Dialog for closing a single dirty tab.
    close_confirm: CloseConfirmDialog,
    /// Dialog shown when the user tries to quit with dirty tabs.
    quit_confirm: QuitConfirmDialog,
    /// When set, close this tab index after a successful save.
    pending_close_after_save: Option<usize>,
    /// Set to true when user confirms quit — lets the OS close proceed.
    allow_quit: bool,
    /// When true, quit the app after all dirty tabs are handled.
    error_message: Option<String>,
    success_message: Option<String>,
    last_autosave: Instant,
}

impl Default for FractalEditor {
    fn default() -> Self {
        let theme = Theme::default();
        Self {
            tabs: vec![Tab::new(theme)],
            active_tab: 0,
            terminal: Terminal::new(theme),
            file_dialog: FileDialog::new(),
            docs_panel: DocsPanel::new(theme),
            docs_open: false,
            menu_state: MenuBarState::default(),
            close_confirm: CloseConfirmDialog::new(),
            quit_confirm: QuitConfirmDialog::new(),
            pending_close_after_save: None,
            allow_quit: false,
            theme,
            error_message: None,
            success_message: None,
            last_autosave: Instant::now(),
        }
    }
}

// ── File / tab operations ─────────────────────────────────────────────────────

impl FractalEditor {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn open_file(&mut self, path: &PathBuf) {
        if let Some(i) = self.tabs.iter().position(|t| {
            t.current_file.as_deref() == Some(path.as_path())
        }) {
            self.active_tab = i;
            return;
        }
        match fs::read_to_string(path) {
            Ok(content) => {
                if self.tabs.len() == 1 && self.tabs[0].is_pristine_new() {
                    self.tabs[0] = Tab::from_file(path.clone(), content, self.theme);
                } else {
                    self.tabs.push(Tab::from_file(path.clone(), content, self.theme));
                    self.active_tab = self.tabs.len() - 1;
                }
                self.success_message = Some(format!("Opened: {}", path.display()));
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open: {e}"));
            }
        }
    }

    fn save_file(&mut self, path: &PathBuf) {
        let tab = &mut self.tabs[self.active_tab];
        tab.code = format_code(&tab.code);
        match fs::write(path, &tab.code) {
            Ok(_) => {
                tab.last_saved_code = tab.code.clone();
                tab.current_file = Some(path.clone());
                self.last_autosave = Instant::now();
                self.success_message = Some(format!("Saved & formatted: {}", path.display()));
                self.error_message = None;
                if let Some(idx) = self.pending_close_after_save.take() {
                    self.close_tab(idx);
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to save: {e}"));
                self.pending_close_after_save = None;
            }
        }
    }

    fn autosave(&mut self) {
        if let Some(path) = self.tabs[self.active_tab].current_file.clone() {
            let tab = &mut self.tabs[self.active_tab];
            tab.code = format_code(&tab.code);
            if fs::write(&path, &tab.code).is_ok() {
                tab.last_saved_code = tab.code.clone();
            }
        }
        self.last_autosave = Instant::now();
    }

    fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            self.tabs[0] = Tab::new(self.theme);
            self.active_tab = 0;
            return;
        }
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    fn request_close_tab(&mut self, index: usize) {
        if self.tabs[index].is_dirty() {
            self.close_confirm.open(index, self.tabs[index].display_name());
        } else {
            self.close_tab(index);
        }
    }

    fn run_code(&mut self, ctx: &egui::Context) {
        let tab = &mut self.tabs[self.active_tab];
        let source_path = match &tab.current_file {
            Some(p) => { let _ = fs::write(p, &tab.code); p.clone() }
            None => {
                let tmp = std::env::temp_dir().join("fractal_temp_run.fr");
                let _ = fs::write(&tmp, &tab.code);
                tmp
            }
        };
        self.terminal.clear();
        self.terminal.append("▶ Running fractal-compiler…\n\n");
        tab.is_running = true;

        let buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        tab.output_rx = Some(buf.clone());
        let ctx2 = ctx.clone();
        let path_str = source_path.to_string_lossy().to_string();

        thread::spawn(move || {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));
            let compiler = exe_dir.join("fractal-compiler");
            let compiler_path = if compiler.exists() {
                compiler
            } else {
                PathBuf::from("fractal-compiler")
            };

            match Command::new(&compiler_path)
                .arg(&path_str)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
            {
                Ok(out) => {
                    let mut lines = buf.lock().unwrap();
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    if !stdout.is_empty() { lines.push(stdout); }
                    if !stderr.is_empty() { lines.push(stderr); }
                    if out.status.success() {
                        lines.push("\n✓ Exited successfully.\n".to_string());
                    } else {
                        let code = out.status.code()
                            .map(|c| c.to_string()).unwrap_or("?".into());
                        lines.push(format!("\n✗ Exited with code {code}.\n"));
                    }
                }
                Err(e) => {
                    let mut lines = buf.lock().unwrap();
                    lines.push(format!("error: failed to launch compiler: {e}\n"));
                    lines.push(
                        "  Is 'fractal-compiler' in your PATH or next to this binary?\n"
                            .to_string(),
                    );
                }
            }
            buf.lock().unwrap().push("\x00DONE\x00".to_string());
            ctx2.request_repaint();
        });
    }

    fn poll_compiler_output(&mut self) {
        let mut done = false;
        if let Some(rx) = &self.tabs[self.active_tab].output_rx {
            for line in rx.lock().unwrap().drain(..) {
                if line == "\x00DONE\x00" { done = true; }
                else { self.terminal.append(&line); }
            }
        }
        if done {
            self.tabs[self.active_tab].is_running = false;
            self.tabs[self.active_tab].output_rx = None;
        }
    }
}

// ── UI helpers ────────────────────────────────────────────────────────────────

impl FractalEditor {
    fn show_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut close_req: Option<usize> = None;

                for i in 0..self.tabs.len() {
                    let is_active = i == self.active_tab;
                    let name = self.tabs[i].display_name();
                    let dirty = self.tabs[i].is_dirty();

                    let label_text = if dirty {
                        format!("● {name}")
                    } else {
                        name
                    };

                    let bg = if is_active {
                        egui::Color32::from_rgb(90, 90, 100)
                    } else {
                        egui::Color32::from_rgb(38, 38, 48)
                    };

                    let text_color = if is_active {
                        egui::Color32::from_rgb(230, 230, 230)
                    } else {
                        egui::Color32::from_rgb(160, 160, 160)
                    };

                    egui::Frame::none()
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&label_text).color(text_color)
                                    )
                                    .sense(egui::Sense::click()),
                                ).clicked() {
                                    self.active_tab = i;
                                }

                                let close_btn = egui::Button::new(
                                    egui::RichText::new("×")
                                        .size(14.0)
                                        .color(egui::Color32::from_rgb(160, 160, 160)),
                                )
                                .frame(false)
                                .min_size(egui::vec2(16.0, 16.0));

                                if ui.add(close_btn).on_hover_text("Close tab").clicked() {
                                    close_req = Some(i);
                                }
                            });
                        });

                    ui.add_space(1.0);
                }

                if ui.small_button(" + ").on_hover_text("New tab").clicked() {
                    self.tabs.push(Tab::new(self.theme));
                    self.active_tab = self.tabs.len() - 1;
                }

                if let Some(idx) = close_req {
                    self.request_close_tab(idx);
                }
            });
        });
    }

    fn handle_close_confirm(&mut self, ctx: &egui::Context) {
        match self.close_confirm.show(ctx) {
            CloseConfirmAction::Cancel => {}
            CloseConfirmAction::Discard(idx) => self.close_tab(idx),
            CloseConfirmAction::Save(idx) => {
                let prev = self.active_tab;
                self.active_tab = idx;
                if let Some(path) = self.tabs[idx].current_file.clone() {
                    self.pending_close_after_save = Some(idx);
                    self.save_file(&path);
                    if self.active_tab == idx {
                        self.active_tab = prev.min(self.tabs.len().saturating_sub(1));
                    }
                } else {
                    self.pending_close_after_save = Some(idx);
                    self.file_dialog.open_for_save("untitled.fr");
                }
            }
            CloseConfirmAction::Pending => {}
        }
    }

    fn handle_quit_confirm(&mut self, ctx: &egui::Context) {
        match self.quit_confirm.show(ctx) {
            QuitConfirmAction::Keep => {
                // User chose to continue working — dialog already closed itself.
            }
            QuitConfirmAction::Discard => {
                self.allow_quit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            QuitConfirmAction::Pending => {}
        }
    }

    fn show_status_bar(&mut self, ctx: &egui::Context) {
        if let Some(msg) = self.error_message.clone() {
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "❌");
                    ui.label(&msg);
                    if ui.button("✖").clicked() { self.error_message = None; }
                });
            });
        } else if let Some(msg) = self.success_message.clone() {
            egui::TopBottomPanel::bottom("success_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "✓");
                    ui.label(&msg);
                    if ui.button("✖").clicked() { self.success_message = None; }
                });
            });
        }
    }
}

// ── eframe::App ───────────────────────────────────────────────────────────────

impl eframe::App for FractalEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_compiler_output();

        // Intercept the OS close button. eframe 0.29+ exposes close_requested
        // via ViewportInfo inside ctx.input().
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && !self.allow_quit {
            let dirty: Vec<String> = self.tabs.iter()
                .filter(|t| t.is_dirty())
                .map(|t| t.display_name())
                .collect();
            if !dirty.is_empty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if !self.quit_confirm.visible {
                    self.quit_confirm.open(dirty);
                }
            }
        }

        // Ctrl+` toggles terminal
        ctx.input_mut(|i| {
            if (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::Backtick) {
                self.terminal.toggle_minimized();
            }
        });

        // Autosave
        let needs_autosave = self.tabs[self.active_tab].current_file.is_some()
            && self.tabs[self.active_tab].is_dirty()
            && self.last_autosave.elapsed().as_secs() >= AUTOSAVE_INTERVAL_SECS;
        if needs_autosave {
            self.autosave();
        }

        // Repaint rate
        if self.tabs[self.active_tab].is_running {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(10));
        }

        ctx.set_visuals(egui::Visuals {
            window_fill: self.theme.background,
            panel_fill: self.theme.background,
            ..egui::Visuals::dark()
        });

        // ── Menu bar ──────────────────────────────────────────────────────
        let tab = &self.tabs[self.active_tab];
        let action = show_menu_bar(
            ctx,
            &mut self.menu_state,
            tab.current_file.as_ref(),
            tab.is_running,
            self.docs_open,
            tab.is_dirty(),
            tab.current_file.is_none(),
        );

        match action {
            MenuAction::OpenDialog => self.file_dialog.open_for_open(),
            MenuAction::SaveDialog => {
                let name = self.tabs[self.active_tab]
                    .current_file.as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "untitled.fr".to_string());
                self.file_dialog.open_for_save(&name);
            }
            MenuAction::SaveCurrent => {
                if let Some(p) = self.tabs[self.active_tab].current_file.clone() {
                    self.save_file(&p);
                }
            }
            MenuAction::New => {
                self.tabs.push(Tab::new(self.theme));
                self.active_tab = self.tabs.len() - 1;
                self.docs_open = false;
            }
            MenuAction::Run => self.run_code(ctx),
            MenuAction::ToggleDocs => self.docs_open = !self.docs_open,
            MenuAction::None => {}
        }

        // ── Panels ────────────────────────────────────────────────────────
        self.show_tab_bar(ctx);
        self.handle_close_confirm(ctx);
        self.handle_quit_confirm(ctx);

        self.file_dialog.show(ctx);
        if let Some(result) = self.file_dialog.result.take() {
            match result.mode {
                FileDialogMode::Open => self.open_file(&result.path),
                FileDialogMode::Save => self.save_file(&result.path),
            }
        }

        self.show_status_bar(ctx);
        self.terminal.show(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.docs_open {
                self.docs_panel.show(ui);
            } else {
                let tab = &mut self.tabs[self.active_tab];
                tab.editor.show(ui, &mut tab.code);
            }
        });
    }
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Fractal IDLE",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([600.0, 400.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(FractalEditor::new(cc)))),
    )
}