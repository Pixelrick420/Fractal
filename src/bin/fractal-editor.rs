use eframe::egui;
use fractal::ui::close_confirm::{
    CloseConfirmAction, CloseConfirmDialog, QuitConfirmAction, QuitConfirmDialog,
};
use fractal::ui::debugger::{DebugFrame, DebugSession};
use fractal::ui::docs::DocsWindow;
use fractal::ui::editor::{show_empty_state, EmptyStateAction};
use fractal::ui::file_dialog::{FileDialog, FileDialogMode};
use fractal::ui::formatter::format_code;
use fractal::ui::icons::{self as ic, setup_fonts};
use fractal::ui::menu_bar::{show_menu_bar, MenuAction, MenuBarState};
use fractal::ui::search_bar::{SearchBar, SearchBarAction};
use fractal::ui::tab::{show_tab_bar, Tab, TabBarAction};
use fractal::ui::terminal::Terminal;
use fractal::ui::theme::{Theme, ThemeVariant};
use fractal::ui::tree_view::TreeViewWindow;
use fractal::ui::user_profile::{SettingsPanel, UserProfile};
use fractal::ui::var_view::VarViewWindow;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const AUTOSAVE_INTERVAL_SECS: u64 = 120;
const MAX_RECENT_FILES: usize = 10;
const SESSION_FILE: &str = "fractal_session.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SessionState {
    open_files: Vec<PathBuf>,
    active_index: usize,
    recent_files: Vec<PathBuf>,
}

impl SessionState {
    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("fractal-editor")
            .join(SESSION_FILE)
    }

    fn load() -> Self {
        let p = Self::path();
        if let Ok(data) = fs::read_to_string(&p) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&p, data);
        }
    }
}

enum CompileResult {
    Success(PathBuf),
    DebugSuccess(PathBuf, PathBuf),
    Error(String),
    LaunchError(String),
}

thread_local! {
    static PENDING_COMPILE: std::cell::RefCell<Option<Arc<Mutex<Option<CompileResult>>>>> =
        std::cell::RefCell::new(None);
}

struct FractalEditor {
    profile: UserProfile,
    theme: Theme,

    tabs: Vec<Tab>,
    active_tab: usize,

    menu_state: MenuBarState,
    terminal: Terminal,
    file_dialog: FileDialog,
    docs_window: DocsWindow,
    close_confirm: CloseConfirmDialog,
    quit_confirm: QuitConfirmDialog,
    settings_panel: SettingsPanel,
    search_bar: SearchBar,

    pending_close_after_save: Option<usize>,
    pending_run_after_save: bool,
    pending_debug_after_save: bool,
    allow_quit: bool,

    error_message: Option<String>,
    success_message: Option<String>,
    last_autosave: Instant,

    recent_files: Vec<PathBuf>,

    debug_session: Option<DebugSession>,
    debug_frame: Option<DebugFrame>,
    debug_jsonl_path: Option<PathBuf>,
    debug_binary_running: bool,
    pending_debug_source: Option<PathBuf>,
    tree_view_window: TreeViewWindow,
    var_view_window: VarViewWindow,
}

impl FractalEditor {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_fonts(&cc.egui_ctx);
        let profile = UserProfile::load();
        let theme = Theme::from_variant(profile.theme);
        let mut file_dialog = FileDialog::new();
        file_dialog.update_theme(theme);

        let session = SessionState::load();
        let recent_files = session.recent_files.clone();

        let mut editor = Self {
            theme,
            terminal: Terminal::new(theme, &cc.egui_ctx),
            file_dialog,
            docs_window: DocsWindow::new(theme),
            menu_state: MenuBarState::default(),
            close_confirm: CloseConfirmDialog::new(),
            quit_confirm: QuitConfirmDialog::new(),
            settings_panel: {
                let mut sp = SettingsPanel::new();
                sp.update_theme(theme);
                sp
            },
            search_bar: SearchBar::default(),
            pending_close_after_save: None,
            pending_run_after_save: false,
            pending_debug_after_save: false,
            allow_quit: false,
            error_message: None,
            success_message: None,
            last_autosave: Instant::now(),
            tabs: vec![Tab::new(theme)],
            active_tab: 0,
            profile,
            recent_files,
            debug_session: None,
            debug_frame: None,
            debug_jsonl_path: None,
            debug_binary_running: false,
            pending_debug_source: None,
            tree_view_window: TreeViewWindow::new(),
            var_view_window: VarViewWindow::new(),
        };

        if !session.open_files.is_empty() {
            let mut opened_any = false;
            for path in &session.open_files {
                if path.exists() {
                    editor.open_file(path);
                    opened_any = true;
                }
            }
            if opened_any {
                let saved_idx = session
                    .active_index
                    .min(editor.tabs.len().saturating_sub(1));
                editor.active_tab = saved_idx;
            }
        }

        editor
    }

    fn apply_theme(&mut self, variant: ThemeVariant) {
        self.profile.theme = variant;
        self.theme = Theme::from_variant(variant);
        for tab in &mut self.tabs {
            tab.editor.update_theme(self.theme);
        }
        self.terminal.update_theme(self.theme);
        self.docs_window.update_theme(self.theme);
        self.settings_panel.update_theme(self.theme);
        self.file_dialog.update_theme(self.theme);
    }

    fn open_file(&mut self, path: &PathBuf) {
        if let Some(i) = self
            .tabs
            .iter()
            .position(|t| t.current_file.as_deref() == Some(path.as_path()))
        {
            self.active_tab = i;
            return;
        }
        match fs::read_to_string(path) {
            Ok(content) => {
                if self.tabs.len() == 1 && self.tabs[0].is_pristine_new() {
                    self.tabs[0] = Tab::from_file(path.clone(), content, self.theme);
                    self.active_tab = 0;
                } else {
                    self.tabs
                        .push(Tab::from_file(path.clone(), content, self.theme));
                    self.active_tab = self.tabs.len() - 1;
                }
                self.push_recent(path.clone());
                self.success_message = Some(format!("Opened: {}", path.display()));
                self.error_message = None;
            }
            Err(e) => self.error_message = Some(format!("Failed to open: {e}")),
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
                self.push_recent(path.clone());
                self.success_message = Some(format!("Saved: {}", path.display()));
                self.error_message = None;
                if let Some(idx) = self.pending_close_after_save.take() {
                    self.close_tab(idx);
                }
                if self.pending_debug_after_save {
                    self.pending_debug_after_save = false;
                    if let Some(p) = self
                        .tabs
                        .get(self.active_tab)
                        .and_then(|t| t.current_file.clone())
                    {
                        self.launch_debug_compile(p);
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to save: {e}"));
                self.pending_close_after_save = None;
                self.pending_debug_after_save = false;
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

    fn push_recent(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(MAX_RECENT_FILES);
    }

    fn save_session(&self) {
        let open_files: Vec<PathBuf> = self
            .tabs
            .iter()
            .filter_map(|t| t.current_file.clone())
            .collect();
        SessionState {
            open_files,
            active_index: self.active_tab,
            recent_files: self.recent_files.clone(),
        }
        .save();
    }

    fn close_tab(&mut self, index: usize) {
        if self.tabs.is_empty() {
            return;
        }
        self.tabs.remove(index);
        if self.active_tab >= self.tabs.len() && self.active_tab > 0 {
            self.active_tab = self.tabs.len().saturating_sub(1);
        }
        if self.tabs.is_empty() {
            self.search_bar.visible = false;
        }
    }

    fn request_close_tab(&mut self, index: usize) {
        if self.tabs[index].is_dirty() {
            self.close_confirm
                .open(index, self.tabs[index].display_name());
        } else {
            self.close_tab(index);
        }
    }

    fn search_navigate(&mut self, forward: bool) {
        if self.search_bar.total_matches == 0 {
            return;
        }
        let n = self.search_bar.total_matches;
        if forward {
            self.search_bar.current_match = (self.search_bar.current_match + 1) % n;
        } else {
            self.search_bar.current_match = (self.search_bar.current_match + n - 1) % n;
        }
        if let Some(tab) = self.tabs.get(self.active_tab) {
            let code = tab.code.clone();
            self.search_bar.update_matches(&code);
        }
    }

    fn replace_current(&mut self) {
        if self.tabs.is_empty() || self.search_bar.query.is_empty() {
            return;
        }
        let tab = &mut self.tabs[self.active_tab];
        let query = self.search_bar.query.clone();
        let replacement = self.search_bar.replace_text.clone();
        let match_case = self.search_bar.match_case;
        let idx = self.search_bar.current_match;
        let code = tab.code.clone();
        let mut found = 0usize;
        let mut byte_pos = 0usize;
        let lower_code = if match_case {
            code.clone()
        } else {
            code.to_lowercase()
        };
        let lower_query = if match_case {
            query.clone()
        } else {
            query.to_lowercase()
        };
        while let Some(pos) = lower_code[byte_pos..].find(lower_query.as_str()) {
            let abs_pos = byte_pos + pos;
            if found == idx {
                let mut new_code = String::with_capacity(code.len());
                new_code.push_str(&code[..abs_pos]);
                new_code.push_str(&replacement);
                new_code.push_str(&code[abs_pos + query.len()..]);
                tab.code = new_code;
                self.search_bar.current_match = idx
                    .saturating_sub(0)
                    .min(self.search_bar.total_matches.saturating_sub(1));
                return;
            }
            found += 1;
            byte_pos = abs_pos + lower_query.len();
        }
    }

    fn replace_all(&mut self) {
        if self.tabs.is_empty() || self.search_bar.query.is_empty() {
            return;
        }
        let tab = &mut self.tabs[self.active_tab];
        let query = self.search_bar.query.clone();
        let replacement = self.search_bar.replace_text.clone();
        let new_code = if self.search_bar.match_case {
            tab.code.replace(query.as_str(), &replacement)
        } else {
            let lower_code = tab.code.to_lowercase();
            let lower_query = query.to_lowercase();
            let mut result = String::with_capacity(tab.code.len());
            let mut last = 0usize;
            let mut search_from = 0usize;
            while let Some(pos) = lower_code[search_from..].find(lower_query.as_str()) {
                let abs = search_from + pos;
                result.push_str(&tab.code[last..abs]);
                result.push_str(&replacement);
                last = abs + query.len();
                search_from = last;
            }
            result.push_str(&tab.code[last..]);
            result
        };
        let count = if self.search_bar.match_case {
            tab.code.matches(query.as_str()).count()
        } else {
            tab.code
                .to_lowercase()
                .matches(query.to_lowercase().as_str())
                .count()
        };
        tab.code = new_code;
        self.search_bar.current_match = 0;
        self.success_message = Some(format!("Replaced {count} occurrence(s)."));
    }

    fn run_code(&mut self, ctx: &egui::Context) {
        if self.tabs.is_empty() {
            return;
        }
        if self.tabs[self.active_tab].current_file.is_none() {
            self.pending_run_after_save = true;
            self.file_dialog.open_for_save("untitled.fr");
            return;
        }

        let tab = &mut self.tabs[self.active_tab];
        let source_path = tab.current_file.clone().unwrap();
        let _ = fs::write(&source_path, &tab.code);
        tab.is_running = true;

        self.terminal.minimized = false;
        self.launch_compile(source_path, false, ctx);
    }

    fn launch_compile(&mut self, source_path: PathBuf, debug: bool, ctx: &egui::Context) {
        let result_buf: Arc<Mutex<Option<CompileResult>>> = Arc::new(Mutex::new(None));
        let result_buf2 = result_buf.clone();
        let ctx2 = ctx.clone();
        let path_str = source_path.to_string_lossy().to_string();
        let bin_path = source_path.with_extension("");

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

        let compiler_str = compiler_path.to_string_lossy();
        let bin_str = bin_path.to_string_lossy();
        let shell_cmd = if debug {
            format!("{compiler_str} debug {path_str} 2>&1\n")
        } else {
            format!("{compiler_str} {path_str} 2>&1 && {bin_str}\n")
        };
        self.terminal.append(&shell_cmd);
        self.terminal.minimized = false;

        thread::spawn(move || {
            let mut cmd = Command::new(&compiler_path);
            if debug {
                cmd.arg("debug");
            }
            cmd.arg(&path_str)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let result = match cmd.output() {
                Ok(out) if out.status.success() => {
                    if debug {
                        let meta_path = PathBuf::from(&path_str).with_extension("debug-meta");
                        if let Ok(jsonl_str) = std::fs::read_to_string(&meta_path) {
                            let jp = PathBuf::from(jsonl_str.trim());
                            CompileResult::DebugSuccess(bin_path, jp)
                        } else {
                            CompileResult::Error(
                                "Debug compile succeeded but could not read .debug-meta sidecar file".into(),
                            )
                        }
                    } else {
                        CompileResult::Success(bin_path)
                    }
                }
                Ok(_) => CompileResult::Error(String::new()),
                Err(e) => CompileResult::LaunchError(format!(
                    "Could not launch fractal-compiler: {e}\n\
                     Is it in PATH or next to this binary?"
                )),
            };

            *result_buf2.lock().unwrap() = Some(result);
            ctx2.request_repaint();
        });

        PENDING_COMPILE.with(|cell| {
            *cell.borrow_mut() = Some(result_buf);
        });
    }

    fn poll_compiler_output(&mut self, ctx: &egui::Context) {
        if self.tabs.is_empty() {
            return;
        }
        let tab = &self.tabs[self.active_tab];
        if !tab.is_running {
            return;
        }

        let maybe_result: Option<CompileResult> = PENDING_COMPILE.with(|cell| {
            let borrow = cell.borrow();
            if let Some(ref arc) = *borrow {
                arc.lock().unwrap().take()
            } else {
                None
            }
        });

        if let Some(result) = maybe_result {
            PENDING_COMPILE.with(|cell| *cell.borrow_mut() = None);
            self.tabs[self.active_tab].is_running = false;
            self.tabs[self.active_tab].output_rx = None;

            match result {
                CompileResult::Success(_bin_path) => {}
                CompileResult::DebugSuccess(bin_path, jsonl_path) => {
                    self.start_debug_session_from_source(&jsonl_path, ctx);
                    self.terminal.run_binary(&bin_path);
                    self.terminal.minimized = false;
                    self.debug_binary_running = true;
                    self.debug_jsonl_path = Some(jsonl_path);
                    self.success_message =
                        Some("Debug session started — F5 to step, Shift+F5 to stop.".into());
                }
                CompileResult::Error(msg) => {
                    if !msg.trim().is_empty() {
                        self.error_message = Some(msg);
                    }
                }
                CompileResult::LaunchError(msg) => {
                    self.error_message = Some(msg);
                }
            }
        }
    }

    fn run_debug(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        if self.tabs[self.active_tab].current_file.is_none() {
            self.pending_debug_after_save = true;
            self.file_dialog.open_for_save("untitled.fr");
            return;
        }
        let source_path = self.tabs[self.active_tab].current_file.clone().unwrap();
        let tab = &mut self.tabs[self.active_tab];
        let _ = fs::write(&source_path, &tab.code);
        tab.is_running = true;
        self.terminal.minimized = false;
        self.launch_debug_compile(source_path);
    }

    fn launch_debug_compile(&mut self, source_path: PathBuf) {
        self.pending_debug_source = Some(source_path);
    }

    fn start_debug_session_from_source(&mut self, jsonl_path: &PathBuf, _ctx: &egui::Context) {
        use fractal::compiler::lexer::tokenize_with_source;
        use fractal::compiler::parser::parse_with_source;
        use fractal::compiler::preprocessor::preprocess;
        use fractal::compiler::semanter::analyze;

        let code = match self.tabs.get(self.active_tab) {
            Some(tab) => tab.code.clone(),
            None => return,
        };
        let source_name = self
            .tabs
            .get(self.active_tab)
            .and_then(|t| t.current_file.as_ref())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "<debug>".into());

        let processed = preprocess(&code, &source_name);
        let tokens = tokenize_with_source(&processed, &source_name);
        let root = match parse_with_source(tokens, &source_name) {
            Ok(r) => r,
            Err(e) => {
                self.error_message = Some(format!("Parse error: {}", e.message));
                return;
            }
        };
        let sem = analyze(&root);
        if sem.has_errors() {
            self.error_message = Some(
                sem.errors
                    .iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join("; "),
            );
            return;
        }

        let session = DebugSession::new(&root, jsonl_path.clone());
        let frame = session.current_frame();
        self.debug_frame = Some(frame);
        self.debug_session = Some(session);
        self.tree_view_window.open = true;
        self.var_view_window.open = true;
        self.var_view_window.clear_output();
    }

    fn step_debug(&mut self) {
        if let Some(ref mut session) = self.debug_session {
            session.poll_file();
        }

        let Some(ref mut session) = self.debug_session else {
            return;
        };

        match session.step() {
            None => {
                self.success_message =
                    Some("Waiting for program output… (is input expected?)".into());
            }
            Some(frame) => {
                self.var_view_window.push_output(&frame.buffered_output);

                let finished = frame.finished;
                let errored = frame.error.is_some();
                let err_msg = frame.error.clone();
                self.debug_frame = Some(frame);

                if finished {
                    self.success_message = Some("Debug: program finished.".into());
                    self.debug_binary_running = false;
                } else if errored {
                    let msg = err_msg.unwrap_or_else(|| "Runtime error".into());
                    self.error_message = Some(format!("Debug fault: {msg}"));
                    self.debug_session = None;
                    self.debug_binary_running = false;
                }
            }
        }
    }

    fn stop_debug_session(&mut self) {
        if let Some(ref path) = self.debug_jsonl_path {
            let _ = fs::remove_file(path);
        }
        self.debug_session = None;
        self.debug_frame = None;
        self.debug_jsonl_path = None;
        self.debug_binary_running = false;
        self.var_view_window.clear_output();
        self.success_message = Some("Debug session stopped.".into());
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
            QuitConfirmAction::Keep => {}
            QuitConfirmAction::Discard => {
                self.allow_quit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            QuitConfirmAction::Pending => {}
        }
    }

    fn show_status_bar(&mut self, ctx: &egui::Context) {
        let t = self.theme;
        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::new()
                    .fill(t.status_bar_bg)
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(msg) = self.error_message.clone() {
                        ui.label(
                            egui::RichText::new(ic::ERROR)
                                .size(12.0)
                                .color(t.terminal_error),
                        );
                        ui.label(egui::RichText::new(&msg).size(11.0).color(t.terminal_error));
                        let xr = ui.add(
                            egui::Button::new(
                                egui::RichText::new(ic::TAB_CLOSE)
                                    .size(11.0)
                                    .color(t.status_bar_fg),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE),
                        );
                        if xr.hovered() {
                            ui.painter().rect_filled(
                                xr.rect.expand(2.0),
                                egui::CornerRadius::same(3),
                                t.button_hover_bg,
                            );
                        }
                        if xr.clicked() {
                            self.error_message = None;
                        }
                    } else if let Some(msg) = self.success_message.clone() {
                        ui.label(
                            egui::RichText::new(ic::SUCCESS)
                                .size(12.0)
                                .color(egui::Color32::from_rgb(80, 188, 100)),
                        );
                        ui.label(egui::RichText::new(&msg).size(11.0).color(t.status_bar_fg));
                        let xr = ui.add(
                            egui::Button::new(
                                egui::RichText::new(ic::TAB_CLOSE)
                                    .size(11.0)
                                    .color(t.status_bar_fg),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE),
                        );
                        if xr.hovered() {
                            ui.painter().rect_filled(
                                xr.rect.expand(2.0),
                                egui::CornerRadius::same(3),
                                t.button_hover_bg,
                            );
                        }
                        if xr.clicked() {
                            self.success_message = None;
                        }
                    } else if let Some(tab) = self.tabs.get(self.active_tab) {
                        let path = tab
                            .current_file
                            .as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Untitled".to_string());
                        ui.label(egui::RichText::new(&path).size(11.0).color(t.status_bar_fg));
                        if tab.is_dirty() {
                            ui.label(
                                egui::RichText::new(ic::UNSAVED)
                                    .size(12.0)
                                    .color(t.tab_dirty_dot),
                            );
                        }
                        if let Some(ref session) = self.debug_session {
                            let cur = session.cursor();
                            let total = session.total_steps();
                            ui.label(
                                egui::RichText::new(format!(" 🐛 step {cur}/{total}"))
                                    .size(11.0)
                                    .color(t.accent),
                            );
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(t.variant.label())
                                .size(11.0)
                                .color(t.status_bar_fg),
                        );
                        ui.separator();
                        let n = self.tabs.len();
                        ui.label(
                            egui::RichText::new(format!(
                                "{n} tab{}",
                                if n == 1 { "" } else { "s" }
                            ))
                            .size(11.0)
                            .color(t.status_bar_fg),
                        );
                        if self.search_bar.visible && !self.search_bar.query.is_empty() {
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!(
                                    "🔍 {}/{}",
                                    if self.search_bar.total_matches > 0 {
                                        self.search_bar.current_match + 1
                                    } else {
                                        0
                                    },
                                    self.search_bar.total_matches
                                ))
                                .size(11.0)
                                .color(t.status_bar_fg),
                            );
                        }
                    });
                });
            });
    }
}

impl eframe::App for FractalEditor {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_session();
        self.profile.save();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(path) = self.pending_debug_source.take() {
            self.tabs[self.active_tab].is_running = true;
            self.launch_compile(path, true, ctx);
        }

        self.poll_compiler_output(ctx);

        if self.debug_binary_running {
            if let Some(ref mut session) = self.debug_session {
                session.poll_file();
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && !self.allow_quit {
            let dirty: Vec<String> = self
                .tabs
                .iter()
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

        let toggle_terminal = ctx.input(|i| {
            (i.modifiers.ctrl || i.modifiers.mac_cmd) && i.key_pressed(egui::Key::Backtick)
        });
        if toggle_terminal {
            self.terminal.toggle_minimized();
        }

        let needs_autosave = self
            .tabs
            .get(self.active_tab)
            .map(|t| {
                t.current_file.is_some()
                    && t.is_dirty()
                    && self.last_autosave.elapsed().as_secs() >= AUTOSAVE_INTERVAL_SECS
            })
            .unwrap_or(false);
        if needs_autosave {
            self.autosave();
        }

        let is_running = self
            .tabs
            .get(self.active_tab)
            .map(|t| t.is_running)
            .unwrap_or(false);

        if is_running {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else if !self.debug_binary_running {
            ctx.request_repaint_after(std::time::Duration::from_secs(10));
        }

        apply_egui_style(ctx, &self.theme);

        let current_file = self
            .tabs
            .get(self.active_tab)
            .and_then(|t| t.current_file.as_ref())
            .cloned();
        let docs_open = self.docs_window.open;
        let is_debug_running = self.debug_session.is_some();
        let tree_view_open = self.tree_view_window.open;
        let var_view_open = self.var_view_window.open;

        let action = show_menu_bar(
            ctx,
            &mut self.menu_state,
            current_file.as_ref(),
            is_running,
            is_debug_running,
            docs_open,
            tree_view_open,
            var_view_open,
            &self.theme,
            &self.recent_files,
            self.search_bar.visible,
        );

        match action {
            MenuAction::OpenDialog => self.file_dialog.open_for_open(),
            MenuAction::SaveDialog => {
                let (name, dir) = self
                    .tabs
                    .get(self.active_tab)
                    .map(|t| {
                        if let Some(ref path) = t.current_file {
                            let fname = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "untitled.fr".to_string());
                            let dir = path.parent().map(|p| p.to_path_buf());
                            (fname, dir)
                        } else {
                            (t.display_name(), None)
                        }
                    })
                    .unwrap_or_else(|| ("untitled.fr".to_string(), None));
                self.file_dialog.open_for_save_in(&name, dir.as_deref());
            }
            MenuAction::SaveCurrent => {
                if let Some(path) = self
                    .tabs
                    .get(self.active_tab)
                    .and_then(|t| t.current_file.clone())
                {
                    self.save_file(&path);
                }
            }
            MenuAction::New => {
                self.tabs.push(Tab::new(self.theme));
                self.active_tab = self.tabs.len() - 1;
                self.docs_window.open = false;
            }
            MenuAction::Run => self.run_code(ctx),
            MenuAction::StepRun => {
                if self.debug_session.is_some() {
                    self.step_debug();
                } else {
                    self.run_debug();
                }
            }
            MenuAction::StepStop => self.stop_debug_session(),
            MenuAction::ToggleTreeView => self.tree_view_window.open = !self.tree_view_window.open,
            MenuAction::ToggleVarView => self.var_view_window.open = !self.var_view_window.open,
            MenuAction::ToggleDocs => self.docs_window.open = !self.docs_window.open,
            MenuAction::OpenSettings => self.settings_panel.open(),
            MenuAction::OpenRecent(path) => {
                self.open_file(&path.clone());
                self.docs_window.open = false;
            }
            MenuAction::Search => {
                if !self.search_bar.visible {
                    self.search_bar.open_search();
                }
            }
            MenuAction::Replace => {
                if !self.search_bar.visible {
                    self.search_bar.open_replace();
                } else if self.search_bar.replace_mode {
                    self.search_bar.replace_mode = false;
                } else {
                    self.search_bar.replace_mode = true;
                    self.search_bar.focus_replace = true;
                }
            }
            MenuAction::None => {}
        }

        if !self.tabs.is_empty() {
            match show_tab_bar(ctx, &self.tabs, self.active_tab, &self.theme) {
                TabBarAction::Activate(i) => {
                    self.active_tab = i;
                    self.docs_window.open = false;
                }
                TabBarAction::Close(i) => self.request_close_tab(i),
                TabBarAction::New => {
                    self.tabs.push(Tab::new(self.theme));
                    self.active_tab = self.tabs.len() - 1;
                    self.docs_window.open = false;
                }
                TabBarAction::None => {}
            }
        }

        if self.search_bar.visible && self.tabs.is_empty() {
            self.search_bar.visible = false;
        }
        if self.search_bar.visible {
            if let Some(tab) = self.tabs.get(self.active_tab) {
                let code = tab.code.clone();
                self.search_bar.update_matches(&code);
            }
        }
        match self.search_bar.show(ctx, &self.theme) {
            SearchBarAction::FindNext => self.search_navigate(true),
            SearchBarAction::FindPrev => self.search_navigate(false),
            SearchBarAction::ReplaceOne => self.replace_current(),
            SearchBarAction::ReplaceAll => self.replace_all(),
            SearchBarAction::Close => {}
            SearchBarAction::None => {}
        }

        self.handle_close_confirm(ctx);
        self.handle_quit_confirm(ctx);

        self.file_dialog.show(ctx);
        if let Some(result) = self.file_dialog.result.take() {
            match result.mode {
                FileDialogMode::Open => {
                    self.open_file(&result.path);
                    self.docs_window.open = false;
                }
                FileDialogMode::Save => {
                    let run_after = self.pending_run_after_save;
                    self.pending_run_after_save = false;
                    self.save_file(&result.path);
                    if run_after {
                        self.run_code(ctx);
                    }
                }
            }
        }

        if self.settings_panel.show(ctx, &mut self.profile) {
            let v = self.profile.theme;
            self.apply_theme(v);
        }

        self.show_status_bar(ctx);
        self.terminal.show(ctx);

        let active_node_id = self
            .debug_frame
            .as_ref()
            .map(|f| f.active_node_id)
            .unwrap_or(0);

        if self.tree_view_window.open {
            if let Some(ref mut session) = self.debug_session {
                let theme = self.theme;
                self.tree_view_window
                    .show(ctx, session, active_node_id, &theme);
            } else {
                use fractal::compiler::parser::ParseNode;
                use fractal::ui::debugger::DebugSession;
                let dummy_root = ParseNode::Program(vec![]);
                let mut dummy_session =
                    DebugSession::new(&dummy_root, std::path::PathBuf::from(""));
                let theme = self.theme;
                self.tree_view_window
                    .show(ctx, &mut dummy_session, 0, &theme);
            }
        }

        if self.var_view_window.open {
            use fractal::ui::debugger::DebugFrame;
            let placeholder = DebugFrame {
                active_node_id: 0,
                step_label: "No debug session active".into(),
                source_line: 0,
                scopes: vec![],
                call_stack: vec![],
                finished: false,
                error: None,
                buffered_output: String::new(),
            };
            let frame_ref: &DebugFrame = self.debug_frame.as_ref().unwrap_or(&placeholder);
            let theme = self.theme;
            self.var_view_window.show(ctx, frame_ref, &theme);
        }

        let debug_line: Option<usize> = {
            if let Some(ref frame) = self.debug_frame {
                if frame.source_line > 0 {
                    Some(frame.source_line)
                } else {
                    let term = debug_search_term(&frame.step_label);
                    let code = self.tabs.get(self.active_tab).map(|t| t.code.as_str());
                    if !term.is_empty() {
                        let hint = frame.active_node_id;
                        code.and_then(|c| find_code_line_at(c, &term, hint))
                    } else if let Some(c) = code {
                        let step = frame.active_node_id;
                        let _total_steps = step.max(1);
                        let code_lines: Vec<usize> = c
                            .lines()
                            .enumerate()
                            .filter(|(_, l)| {
                                let t = l.trim();
                                !t.is_empty() && !t.starts_with('#') && t != "!start" && t != "!end"
                            })
                            .map(|(i, _)| i + 1)
                            .collect();
                        if code_lines.is_empty() {
                            None
                        } else {
                            let idx = step.min(code_lines.len().saturating_sub(1));
                            Some(code_lines[idx])
                        }
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(self.theme.editor_bg))
            .show(ctx, |ui| {
                if self.docs_window.open {
                    self.docs_window.show(ui);
                } else if self.tabs.is_empty() {
                    match show_empty_state(ui, &self.theme, ctx.viewport_rect()) {
                        EmptyStateAction::Open => self.file_dialog.open_for_open(),
                        EmptyStateAction::New => {
                            self.tabs.push(Tab::new(self.theme));
                            self.active_tab = 0;
                        }
                        EmptyStateAction::None => {}
                    }
                } else if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let fs = self.profile.font_size;
                    let ln = self.profile.show_line_numbers;
                    let sel = if self.search_bar.visible {
                        self.search_bar.current_match_byte_range
                    } else {
                        None
                    };
                    tab.editor
                        .show_with_id(ui, &mut tab.code, tab.id, fs, ln, sel, debug_line);
                }
            });
    }
}

fn apply_egui_style(ctx: &egui::Context, t: &Theme) {
    let mut s = (*ctx.style()).clone();
    s.visuals.window_fill = t.panel_bg;
    s.visuals.panel_fill = t.panel_bg;
    s.visuals.extreme_bg_color = t.editor_bg;
    s.visuals.faint_bg_color = t.button_bg;
    s.visuals.code_bg_color = t.editor_bg;
    s.visuals.window_stroke = egui::Stroke::new(1.0, t.border);
    s.visuals.window_corner_radius = egui::CornerRadius::same(8);
    s.visuals.menu_corner_radius = egui::CornerRadius::same(6);
    s.visuals.window_shadow = egui::Shadow {
        offset: [0, 6],
        blur: 20,
        spread: 0,
        color: egui::Color32::from_black_alpha(96),
    };
    s.visuals.widgets.noninteractive.bg_fill = t.panel_bg;
    s.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, t.border);
    s.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.5, t.tab_active_fg);
    s.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(4);
    s.visuals.widgets.inactive.bg_fill = t.button_bg;
    s.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, t.border);
    s.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.5, t.button_fg);
    s.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(5);
    s.visuals.widgets.inactive.expansion = 0.0;
    s.visuals.widgets.hovered.bg_fill = t.button_hover_bg;
    s.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, t.accent);
    s.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, t.tab_active_fg);
    s.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(5);
    s.visuals.widgets.hovered.expansion = 1.0;
    s.visuals.widgets.active.bg_fill = t.accent;
    s.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, t.accent);
    s.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, egui::Color32::WHITE);
    s.visuals.widgets.active.corner_radius = egui::CornerRadius::same(5);
    s.visuals.widgets.active.expansion = 1.0;
    s.visuals.widgets.open.bg_fill = t.button_hover_bg;
    s.visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, t.accent);
    s.visuals.widgets.open.fg_stroke = egui::Stroke::new(1.5, t.tab_active_fg);
    s.visuals.widgets.open.corner_radius = egui::CornerRadius::same(5);
    s.visuals.selection.bg_fill = t.selection;
    s.visuals.selection.stroke = egui::Stroke::new(1.0, t.accent);
    s.visuals.text_cursor = egui::style::TextCursorStyle {
        stroke: egui::Stroke::new(2.0, t.accent),
        ..Default::default()
    };
    s.visuals.hyperlink_color = t.accent;
    s.spacing.item_spacing = egui::vec2(8.0, 6.0);
    s.spacing.button_padding = egui::vec2(10.0, 6.0);
    s.spacing.window_margin = egui::Margin::same(14);
    s.spacing.menu_margin = egui::Margin::same(4);
    s.visuals.override_text_color = Some(t.tab_active_fg);
    ctx.set_style(s);
}

fn debug_search_term(label: &str) -> String {
    let parts: Vec<&str> = label.splitn(4, ' ').collect();
    match parts.as_slice() {
        [kw, name, ..] if *kw == "Decl" || *kw == "StructDecl" || *kw == "FuncDef" => {
            name.to_string()
        }
        ["For", name, ..] => name.to_string(),
        ["If", ..] => "!if".to_string(),
        ["While", ..] => "!while".to_string(),
        ["Return", ..] => "!return".to_string(),
        ["Break", ..] => "!break".to_string(),
        ["Continue", ..] => "!continue".to_string(),
        ["Assign", op, ..] => match *op {
            "MinusEq" => "-=".to_string(),
            "PlusEq" => "+=".to_string(),
            "StarEq" => "*=".to_string(),
            "SlashEq" => "/=".to_string(),
            "PercentEq" => "%=".to_string(),
            "AmpEq" => "&=".to_string(),
            "PipeEq" => "|=".to_string(),
            "CaretEq" => "^=".to_string(),
            "Eq" => " = ".to_string(),
            _ => String::new(),
        },
        _ => String::new(),
    }
}

#[allow(dead_code)]
fn find_code_line(code: &str, term: &str) -> Option<usize> {
    find_code_line_at(code, term, 0)
}

fn find_code_line_at(code: &str, term: &str, step_hint: usize) -> Option<usize> {
    if term.is_empty() {
        return None;
    }
    let matches: Vec<usize> = code
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            if let Some(idx) = line.find(term) {
                let b = idx;
                let before_ok = b == 0
                    || !line.as_bytes()[b - 1].is_ascii_alphanumeric()
                        && line.as_bytes()[b - 1] != b'_';
                let after = b + term.len();
                let after_ok = after >= line.len()
                    || !line.as_bytes()[after].is_ascii_alphanumeric()
                        && line.as_bytes()[after] != b'_';
                if before_ok && after_ok {
                    return Some(i + 1);
                }
            }
            None
        })
        .collect();

    if matches.is_empty() {
        return None;
    }
    let target = step_hint + 1;
    matches
        .iter()
        .min_by_key(|&&ln| (ln as isize - target as isize).unsigned_abs())
        .copied()
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Fractal Editor",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Fractal Editor")
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([600.0, 400.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(FractalEditor::new(cc)))),
    )
}
