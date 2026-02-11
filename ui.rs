use eframe::egui;
use std::fs;
use std::path::PathBuf;

mod lexer;

#[derive(Clone, Copy)]
struct Theme {
    background: egui::Color32,
    text_default: egui::Color32,
    keyword: egui::Color32,
    type_name: egui::Color32,
    number: egui::Color32,
    string: egui::Color32,
    char_lit: egui::Color32,
    comment: egui::Color32,
    operator: egui::Color32,
    identifier: egui::Color32,
    boolean: egui::Color32,
    line_numbers_bg: egui::Color32,
    line_numbers_fg: egui::Color32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: egui::Color32::from_rgb(30, 30, 30),
            text_default: egui::Color32::from_rgb(212, 212, 212),
            keyword: egui::Color32::from_rgb(197, 134, 192),
            type_name: egui::Color32::from_rgb(78, 201, 176),
            number: egui::Color32::from_rgb(181, 206, 168),
            string: egui::Color32::from_rgb(206, 145, 120),
            char_lit: egui::Color32::from_rgb(209, 105, 105),
            comment: egui::Color32::from_rgb(106, 153, 85),
            operator: egui::Color32::from_rgb(212, 212, 212),
            identifier: egui::Color32::from_rgb(156, 220, 254),
            boolean: egui::Color32::from_rgb(86, 156, 214),
            line_numbers_bg: egui::Color32::from_rgb(25, 25, 25),
            line_numbers_fg: egui::Color32::from_rgb(100, 100, 100),
        }
    }
}

struct FractalEditor {
    code: String,
    current_file: Option<PathBuf>,
    theme: Theme,
    show_open_dialog: bool,
    show_save_dialog: bool,
    file_path_input: String,
    error_message: Option<String>,
    success_message: Option<String>,
}

impl Default for FractalEditor {
    fn default() -> Self {
        Self {
            code: String::from("!start\n# Welcome to Fractal Editor\n:int x = 42;\n!end\n"),
            current_file: None,
            theme: Theme::default(),
            show_open_dialog: false,
            show_save_dialog: false,
            file_path_input: String::new(),
            error_message: None,
            success_message: None,
        }
    }
}

impl FractalEditor {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn open_file(&mut self, path: &str) {
        match fs::read_to_string(path) {
            Ok(content) => {
                self.code = content;
                self.current_file = Some(PathBuf::from(path));
                self.success_message = Some(format!("Opened: {}", path));
                self.error_message = None;
                self.show_open_dialog = false;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open file: {}", e));
                self.success_message = None;
            }
        }
    }

    fn save_file(&mut self, path: &str) {
        match fs::write(path, &self.code) {
            Ok(_) => {
                self.current_file = Some(PathBuf::from(path));
                self.success_message = Some(format!("Saved: {}", path));
                self.error_message = None;
                self.show_save_dialog = false;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to save file: {}", e));
                self.success_message = None;
            }
        }
    }

    fn render_highlighted_text(&self, ui: &mut egui::Ui) {
        let lines: Vec<&str> = self.code.lines().collect();
        let num_lines = lines.len();
        let line_num_width = (num_lines.to_string().len() as f32 * 8.0).max(30.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(line_num_width);
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                        let frame = egui::Frame::none()
                            .fill(self.theme.line_numbers_bg)
                            .inner_margin(egui::Margin::symmetric(8.0, 4.0));

                        frame.show(ui, |ui| {
                            for (i, _) in lines.iter().enumerate() {
                                ui.colored_label(
                                    self.theme.line_numbers_fg,
                                    format!(
                                        "{:>width$}",
                                        i + 1,
                                        width = num_lines.to_string().len()
                                    ),
                                );
                            }
                        });
                    });

                    ui.add_space(2.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.vertical(|ui| {
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

                        for line in lines.iter() {
                            self.render_line(ui, line);
                        }
                    });
                });
            });
    }

    fn render_line(&self, ui: &mut egui::Ui, line: &str) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            let trimmed = line.trim_start();
            if trimmed.starts_with("#") {
                ui.colored_label(self.theme.comment, line);
                return;
            }

            let tokens = self.tokenize_line(line);
            for (text, color) in tokens {
                ui.colored_label(color, text);
            }
        });
    }

    fn tokenize_line(&self, line: &str) -> Vec<(String, egui::Color32)> {
        let mut result = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].is_whitespace() {
                let start = i;
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }
                result.push((chars[start..i].iter().collect(), self.theme.text_default));
                continue;
            }

            if chars[i] == '#' {
                result.push((chars[i..].iter().collect(), self.theme.comment));
                break;
            }

            if chars[i] == '"' {
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() {
                    i += 1;
                }
                result.push((chars[start..i].iter().collect(), self.theme.string));
                continue;
            }

            if chars[i] == '\'' {
                let start = i;
                i += 1;
                if i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                if i < chars.len() && chars[i] == '\'' {
                    i += 1;
                }
                result.push((chars[start..i].iter().collect(), self.theme.char_lit));
                continue;
            }

            if chars[i] == '!' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let color = if self.is_keyword(&text[1..]) {
                    self.theme.keyword
                } else {
                    self.theme.operator
                };
                result.push((text, color));
                continue;
            }

            if chars[i] == ':' {
                let start = i;
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let color = if self.is_type(&text[1..]) {
                    self.theme.type_name
                } else {
                    self.theme.operator
                };
                result.push((text, color));
                continue;
            }

            if self.is_operator_char(chars[i]) {
                let start = i;
                while i < chars.len() && self.is_operator_char(chars[i]) {
                    i += 1;
                }
                result.push((chars[start..i].iter().collect(), self.theme.operator));
                continue;
            }

            if chars[i].is_numeric()
                || (chars[i] == '0' && i + 1 < chars.len() && "box".contains(chars[i + 1]))
            {
                let start = i;
                while i < chars.len()
                    && (chars[i].is_alphanumeric()
                        || chars[i] == '.'
                        || chars[i] == 'x'
                        || chars[i] == 'b'
                        || chars[i] == 'o')
                {
                    i += 1;
                }
                result.push((chars[start..i].iter().collect(), self.theme.number));
                continue;
            }

            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let color = match text.as_str() {
                    "true" | "false" => self.theme.boolean,
                    "NULL" => self.theme.keyword,
                    _ => self.theme.identifier,
                };
                result.push((text, color));
                continue;
            }

            result.push((chars[i].to_string(), self.theme.text_default));
            i += 1;
        }

        result
    }

    fn is_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "start"
                | "end"
                | "exit"
                | "if"
                | "else"
                | "for"
                | "while"
                | "func"
                | "return"
                | "struct"
                | "import"
                | "module"
        )
    }

    fn is_type(&self, s: &str) -> bool {
        matches!(
            s,
            "int" | "float" | "char" | "boolean" | "array" | "list" | "struct"
        )
    }

    fn is_operator_char(&self, c: char) -> bool {
        matches!(
            c,
            '+' | '-'
                | '*'
                | '/'
                | '%'
                | '&'
                | '|'
                | '~'
                | '^'
                | '='
                | '>'
                | '<'
                | '('
                | ')'
                | '{'
                | '}'
                | '['
                | ']'
                | '.'
                | ','
                | ';'
        )
    }
}

impl eframe::App for FractalEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dark_visuals = egui::Visuals {
            window_fill: self.theme.background,
            panel_fill: self.theme.background,
            ..egui::Visuals::dark()
        };
        ctx.set_visuals(dark_visuals);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📂 Open").clicked() {
                        self.show_open_dialog = true;
                        self.file_path_input.clear();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save").clicked() {
                        if let Some(path) = self.current_file.clone() {
                            self.save_file(path.to_str().unwrap_or(""));
                        } else {
                            self.show_save_dialog = true;
                            self.file_path_input.clear();
                        }
                        ui.close_menu();
                    }
                    if ui.button("💾 Save As").clicked() {
                        self.show_save_dialog = true;
                        self.file_path_input.clear();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🆕 New").clicked() {
                        self.code = String::from("!start\n\n!end\n");
                        self.current_file = None;
                        ui.close_menu();
                    }
                });

                ui.separator();

                if let Some(path) = &self.current_file {
                    ui.label(format!("📄 {}", path.display()));
                } else {
                    ui.label("📄 Untitled");
                }
            });
        });

        if let Some(msg) = self.error_message.clone() {
            egui::TopBottomPanel::bottom("error_panel").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "❌ Error:");
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
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "✓ Success:");
                    ui.label(&msg);
                    if ui.button("✖").clicked() {
                        self.success_message = None;
                    }
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() * 0.5);
                    ui.heading("Editor");
                    ui.separator();

                    let text_edit = egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(30);

                    ui.add(text_edit);
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.heading("Preview");
                    ui.separator();

                    let frame = egui::Frame::none()
                        .fill(self.theme.background)
                        .inner_margin(egui::Margin::same(8.0));

                    frame.show(ui, |ui| {
                        self.render_highlighted_text(ui);
                    });
                });
            });
        });

        if self.show_open_dialog {
            egui::Window::new("Open File")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter file path:");
                    ui.text_edit_singleline(&mut self.file_path_input);
                    ui.horizontal(|ui| {
                        if ui.button("Open").clicked() {
                            self.open_file(&self.file_path_input.clone());
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_open_dialog = false;
                        }
                    });
                });
        }

        if self.show_save_dialog {
            egui::Window::new("Save File")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter file path:");
                    ui.text_edit_singleline(&mut self.file_path_input);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            self.save_file(&self.file_path_input.clone());
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_save_dialog = false;
                        }
                    });
                });
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Fractal Code Editor",
        options,
        Box::new(|cc| Ok(Box::new(FractalEditor::new(cc)))),
    )
}
