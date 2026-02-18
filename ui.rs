use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

mod lexer;

mod ui {
    pub mod editor;
    pub mod file_dialog;
    pub mod highlighter;
    pub mod menu_bar;
    pub mod terminal;
    pub mod theme;
}

use ui::editor::CodeEditor;
use ui::file_dialog::{FileDialog, FileDialogMode};
use ui::menu_bar::{show_menu_bar, MenuAction, MenuBarState};
use ui::terminal::Terminal;
use ui::theme::Theme;

struct FractalEditor {
    code: String,
    current_file: Option<PathBuf>,
    theme: Theme,
    menu_state: MenuBarState,
    editor: CodeEditor,
    terminal: Terminal,
    file_dialog: FileDialog,
    is_running: bool,
    // Channel to receive compiler output on the main thread
    output_rx: Option<Arc<Mutex<Vec<String>>>>,
    error_message: Option<String>,
    success_message: Option<String>,
}

impl Default for FractalEditor {
    fn default() -> Self {
        let theme = Theme::default();
        Self {
            code: String::from("!start\n# Welcome to Fractal Editor\n:int x = 42;\n!end\n"),
            current_file: None,
            editor: CodeEditor::new(theme),
            terminal: Terminal::new(theme),
            file_dialog: FileDialog::new(),
            menu_state: MenuBarState::default(),
            theme,
            is_running: false,
            output_rx: None,
            error_message: None,
            success_message: None,
        }
    }
}

impl FractalEditor {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn open_file(&mut self, path: &PathBuf) {
        match fs::read_to_string(path) {
            Ok(content) => {
                self.code = content;
                self.current_file = Some(path.clone());
                self.success_message = Some(format!("Opened: {}", path.display()));
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open: {}", e));
                self.success_message = None;
            }
        }
    }

    fn save_file(&mut self, path: &PathBuf) {
        match fs::write(path, &self.code) {
            Ok(_) => {
                self.current_file = Some(path.clone());
                self.success_message = Some(format!("Saved: {}", path.display()));
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to save: {}", e));
                self.success_message = None;
            }
        }
    }

    fn run_code(&mut self, ctx: &egui::Context) {
        // Save to a temp file first (or use current file if saved)
        let source_path = if let Some(ref p) = self.current_file {
            // Auto-save before running
            let _ = fs::write(p, &self.code);
            p.clone()
        } else {
            // Write to a temp file
            let tmp = std::env::temp_dir().join("fractal_temp_run.frac");
            let _ = fs::write(&tmp, &self.code);
            tmp
        };

        self.terminal.clear();
        self.terminal.append("▶ Running fractal-compiler…\n\n");
        self.is_running = true;

        // Shared output buffer
        let buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        self.output_rx = Some(buf.clone());

        let ctx_clone = ctx.clone();
        let path_str = source_path.to_string_lossy().to_string();

        thread::spawn(move || {
            // Try to find the compiler binary next to the current exe
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."));

            let compiler = exe_dir.join("fractal-compiler");
            let compiler_path = if compiler.exists() {
                compiler
            } else {
                PathBuf::from("fractal-compiler") // rely on PATH
            };

            match Command::new(&compiler_path)
                .arg(&path_str)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
            {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    let mut lines = buf.lock().unwrap();
                    if !stdout.is_empty() {
                        lines.push(stdout);
                    }
                    if !stderr.is_empty() {
                        lines.push(stderr);
                    }
                    if out.status.success() {
                        lines.push("\n✓ Exited successfully.\n".to_string());
                    } else {
                        let code = out
                            .status
                            .code()
                            .map(|c| c.to_string())
                            .unwrap_or("?".into());
                        lines.push(format!("\n✗ Exited with code {}.\n", code));
                    }
                }
                Err(e) => {
                    let mut lines = buf.lock().unwrap();
                    lines.push(format!("error: failed to launch compiler: {}\n", e));
                    lines.push(
                        "       Is 'fractal-compiler' in your PATH or next to this binary?\n"
                            .to_string(),
                    );
                }
            }

            // Signal done with a sentinel
            buf.lock().unwrap().push("\x00DONE\x00".to_string());
            ctx_clone.request_repaint();
        });
    }

    fn poll_compiler_output(&mut self) {
        let mut finished = false;
        if let Some(ref rx) = self.output_rx {
            let mut lines = rx.lock().unwrap();
            for line in lines.drain(..) {
                if line == "\x00DONE\x00" {
                    finished = true;
                } else {
                    self.terminal.append(&line);
                }
            }
        }
        if finished {
            self.is_running = false;
            self.output_rx = None;
        }
    }
}

impl eframe::App for FractalEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background compiler thread
        self.poll_compiler_output();
        if self.is_running {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Theme
        ctx.set_visuals(egui::Visuals {
            window_fill: self.theme.background,
            panel_fill: self.theme.background,
            ..egui::Visuals::dark()
        });

        // ── Menu bar ──────────────────────────────────────────────────────
        let action = show_menu_bar(
            ctx,
            &mut self.menu_state,
            self.current_file.as_ref(),
            self.is_running,
        );

        match action {
            MenuAction::OpenDialog => {
                self.file_dialog.open_for_open();
            }
            MenuAction::SaveDialog => {
                let suggested = self
                    .current_file
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "untitled.frac".to_string());
                self.file_dialog.open_for_save(&suggested);
            }
            MenuAction::SaveCurrent => {
                if let Some(p) = self.current_file.clone() {
                    self.save_file(&p);
                }
            }
            MenuAction::New => {
                self.code = String::from("!start\n\n!end\n");
                self.current_file = None;
            }
            MenuAction::Run => {
                self.run_code(ctx);
            }
            MenuAction::None => {}
        }

        // ── File dialog ───────────────────────────────────────────────────
        self.file_dialog.show(ctx);
        if let Some(result) = self.file_dialog.result.take() {
            match result.mode {
                FileDialogMode::Open => self.open_file(&result.path),
                FileDialogMode::Save => self.save_file(&result.path),
            }
        }

        // ── Status bars ───────────────────────────────────────────────────
        if let Some(msg) = self.error_message.clone() {
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "❌");
                    ui.label(&msg);
                    if ui.button("✖").clicked() {
                        self.error_message = None;
                    }
                });
            });
        }

        if let Some(msg) = self.success_message.clone() {
            egui::TopBottomPanel::bottom("success_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "✓");
                    ui.label(&msg);
                    if ui.button("✖").clicked() {
                        self.success_message = None;
                    }
                });
            });
        }

        // ── Terminal ──────────────────────────────────────────────────────
        self.terminal.show(ctx);

        // ── Main editor ───────────────────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor.show(ui, &mut self.code);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Fractal Code Editor",
        options,
        Box::new(|cc| Ok(Box::new(FractalEditor::new(cc)))),
    )
}
