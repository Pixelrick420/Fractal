use eframe::egui;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use fractal::ui::docs::DocsPanel;
use fractal::ui::editor::CodeEditor;
use fractal::ui::file_dialog::{FileDialog, FileDialogMode};
use fractal::ui::formatter::format_code;
use fractal::ui::menu_bar::{show_menu_bar, MenuAction, MenuBarState};
use fractal::ui::terminal::Terminal;
use fractal::ui::theme::Theme;

const AUTOSAVE_INTERVAL_SECS: u64 = 120;

struct FractalEditor {
    code: String,

    last_saved_code: String,
    current_file: Option<PathBuf>,
    theme: Theme,
    menu_state: MenuBarState,
    editor: CodeEditor,
    terminal: Terminal,
    file_dialog: FileDialog,
    docs_panel: DocsPanel,
    docs_open: bool,
    is_running: bool,
    output_rx: Option<Arc<Mutex<Vec<String>>>>,
    error_message: Option<String>,
    success_message: Option<String>,

    last_autosave: Instant,
}

impl Default for FractalEditor {
    fn default() -> Self {
        let theme = Theme::default();
        let initial_code = String::from("!start\n# code here\n!end\n");
        Self {
            last_saved_code: initial_code.clone(),
            code: initial_code,
            current_file: None,
            editor: CodeEditor::new(theme),
            terminal: Terminal::new(theme),
            file_dialog: FileDialog::new(),
            docs_panel: DocsPanel::new(theme),
            docs_open: false,
            menu_state: MenuBarState::default(),
            theme,
            is_running: false,
            output_rx: None,
            error_message: None,
            success_message: None,
            last_autosave: Instant::now(),
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
                self.code = content.clone();
                self.last_saved_code = content;
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
        self.code = format_code(&self.code);

        match fs::write(path, &self.code) {
            Ok(_) => {
                self.last_saved_code = self.code.clone();
                self.last_autosave = Instant::now();
                self.current_file = Some(path.clone());
                self.success_message = Some(format!("Saved & formatted: {}", path.display()));
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to save: {}", e));
                self.success_message = None;
            }
        }
    }

    fn autosave(&mut self) {
        if let Some(path) = self.current_file.clone() {
            self.code = format_code(&self.code);
            if fs::write(&path, &self.code).is_ok() {
                self.last_saved_code = self.code.clone();
            }
        }

        self.last_autosave = Instant::now();
    }

    fn run_code(&mut self, ctx: &egui::Context) {
        let source_path = if let Some(ref p) = self.current_file {
            let _ = fs::write(p, &self.code);
            p.clone()
        } else {
            let tmp = std::env::temp_dir().join("fractal_temp_run.fr");
            let _ = fs::write(&tmp, &self.code);
            tmp
        };

        self.terminal.clear();
        self.terminal.append("▶ Running fractal-compiler…\n\n");
        self.is_running = true;

        let buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        self.output_rx = Some(buf.clone());

        let ctx_clone = ctx.clone();
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
        self.poll_compiler_output();

        ctx.input_mut(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.mac_cmd;
            if ctrl && i.key_pressed(egui::Key::Backtick) {
                self.terminal.toggle_minimized();
            }
        });

        if self.current_file.is_some()
            && self.code != self.last_saved_code
            && self.last_autosave.elapsed().as_secs() >= AUTOSAVE_INTERVAL_SECS
        {
            self.autosave();
        }

        if self.is_running {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(10));
        }

        ctx.set_visuals(egui::Visuals {
            window_fill: self.theme.background,
            panel_fill: self.theme.background,
            ..egui::Visuals::dark()
        });

        let is_dirty = self.code != self.last_saved_code;
        let is_new = self.current_file.is_none();

        let action = show_menu_bar(
            ctx,
            &mut self.menu_state,
            self.current_file.as_ref(),
            self.is_running,
            self.docs_open,
            is_dirty,
            is_new,
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
                    .unwrap_or_else(|| "untitled.fr".to_string());
                self.file_dialog.open_for_save(&suggested);
            }
            MenuAction::SaveCurrent => {
                if let Some(p) = self.current_file.clone() {
                    self.save_file(&p);
                }
            }
            MenuAction::New => {
                self.code = String::from("!start\n\n!end\n");
                self.last_saved_code = self.code.clone();
                self.current_file = None;
                self.docs_open = false;
            }
            MenuAction::Run => {
                self.run_code(ctx);
            }
            MenuAction::ToggleDocs => {
                self.docs_open = !self.docs_open;
            }
            MenuAction::None => {}
        }

        self.file_dialog.show(ctx);
        if let Some(result) = self.file_dialog.result.take() {
            match result.mode {
                FileDialogMode::Open => self.open_file(&result.path),
                FileDialogMode::Save => self.save_file(&result.path),
            }
        }

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

        self.terminal.show(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.docs_open {
                self.docs_panel.show(ui);
            } else {
                self.editor.show(ui, &mut self.code);
            }
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
