use crate::ui::theme::Theme;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
enum Chapter {
    GettingStarted,
    TypesVariables,
    FunctionsControl,
}

impl Chapter {
    fn label(self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting Started",
            Self::TypesVariables => "Types & Variables",
            Self::FunctionsControl => "Functions & Control Flow",
        }
    }
}

const CHAPTERS: &[Chapter] = &[
    Chapter::GettingStarted,
    Chapter::TypesVariables,
    Chapter::FunctionsControl,
];

pub struct DocsWindow {
    pub open: bool,
    chapter: Chapter,
    theme: Theme,
}

impl DocsWindow {
    pub fn new(theme: Theme) -> Self {
        Self {
            open: false,
            chapter: Chapter::GettingStarted,
            theme,
        }
    }

    pub fn update_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    #[allow(dead_code)]
    pub fn show_inline(&mut self, ui: &mut egui::Ui) {
        let t = &self.theme;
        let full = ui.max_rect();

        ui.painter()
            .rect_filled(full, egui::Rounding::ZERO, t.panel_bg);

        let sidebar_w = 210.0;
        let sep_x = full.min.x + sidebar_w;

        ui.painter().rect_filled(
            egui::Rect::from_min_max(full.min, egui::pos2(sep_x, full.max.y)),
            egui::Rounding::ZERO,
            t.line_numbers_bg,
        );

        ui.painter().line_segment(
            [egui::pos2(sep_x, full.min.y), egui::pos2(sep_x, full.max.y)],
            egui::Stroke::new(1.0, t.border),
        );

        ui.painter().text(
            egui::pos2(full.min.x + 18.0, full.min.y + 22.0),
            egui::Align2::LEFT_CENTER,
            "DOCUMENTATION",
            egui::FontId::proportional(9.5),
            t.tab_inactive_fg,
        );

        let close_rect = egui::Rect::from_center_size(
            egui::pos2(sep_x - 18.0, full.min.y + 22.0),
            egui::vec2(22.0, 22.0),
        );
        let close_resp = ui.interact(
            close_rect,
            egui::Id::new("docs_close_btn"),
            egui::Sense::click(),
        );
        if close_resp.hovered() {
            ui.painter()
                .rect_filled(close_rect, egui::Rounding::same(4.0), t.button_hover_bg);
        }
        ui.painter().text(
            close_rect.center(),
            egui::Align2::CENTER_CENTER,
            crate::ui::icons::TAB_CLOSE,
            egui::FontId::proportional(13.0),
            if close_resp.hovered() {
                t.terminal_error
            } else {
                t.tab_inactive_fg
            },
        );
        if close_resp.clicked() {
            self.open = false;
        }

        let mut cursor_y = full.min.y + 48.0;
        for ch in CHAPTERS {
            let row = egui::Rect::from_min_size(
                egui::pos2(full.min.x, cursor_y),
                egui::vec2(sidebar_w, 40.0),
            );
            let id = egui::Id::new(("doc_ch", ch.label()));
            let resp = ui.interact(row, id, egui::Sense::click());

            let selected = *ch == self.chapter;
            let bg = if selected {
                t.button_hover_bg
            } else if resp.hovered() {
                egui::Color32::from_rgba_premultiplied(
                    t.tab_active_fg.r(),
                    t.tab_active_fg.g(),
                    t.tab_active_fg.b(),
                    15,
                )
            } else {
                egui::Color32::TRANSPARENT
            };

            ui.painter().rect_filled(row, egui::Rounding::ZERO, bg);

            if selected {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(row.min, egui::vec2(3.0, row.height())),
                    egui::Rounding::ZERO,
                    t.accent,
                );
            }

            let fg = if selected { t.tab_active_fg } else { t.menu_fg };
            ui.painter().text(
                egui::pos2(row.min.x + 22.0, row.center().y),
                egui::Align2::LEFT_CENTER,
                ch.label(),
                egui::FontId::proportional(13.5),
                fg,
            );

            if resp.clicked() {}
            cursor_y += 40.0;
        }

        let content_rect = egui::Rect::from_min_max(egui::pos2(sep_x + 1.0, full.min.y), full.max);

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );

        egui::ScrollArea::vertical()
            .id_salt("docs_content_scroll")
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                let pad = 44.0;
                ui.add_space(28.0);
                let w = (content_rect.width() - pad * 2.0).max(200.0);
                ui.set_max_width(w + pad * 2.0);
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.vertical(|ui| {
                        ui.set_max_width(w);
                        match self.chapter {
                            Chapter::GettingStarted => render_getting_started(ui, t),
                            Chapter::TypesVariables => render_types_variables(ui, t),
                            Chapter::FunctionsControl => render_functions_control(ui, t),
                        }
                    });
                });
                ui.add_space(48.0);
            });
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let full = ui.max_rect();

        ui.painter()
            .rect_filled(full, egui::Rounding::ZERO, t.panel_bg);

        let sidebar_w = 210.0;
        let sep_x = full.min.x + sidebar_w;

        ui.painter().rect_filled(
            egui::Rect::from_min_max(full.min, egui::pos2(sep_x, full.max.y)),
            egui::Rounding::ZERO,
            t.line_numbers_bg,
        );
        ui.painter().line_segment(
            [egui::pos2(sep_x, full.min.y), egui::pos2(sep_x, full.max.y)],
            egui::Stroke::new(1.0, t.border),
        );
        ui.painter().text(
            egui::pos2(full.min.x + 18.0, full.min.y + 22.0),
            egui::Align2::LEFT_CENTER,
            "DOCUMENTATION",
            egui::FontId::proportional(9.5),
            t.tab_inactive_fg,
        );

        let mut cursor_y = full.min.y + 48.0;
        for ch in CHAPTERS {
            let row = egui::Rect::from_min_size(
                egui::pos2(full.min.x, cursor_y),
                egui::vec2(sidebar_w, 40.0),
            );
            let id = egui::Id::new(("doc_ch", ch.label()));
            let resp = ui.interact(row, id, egui::Sense::click());

            let selected = *ch == self.chapter;
            let bg = if selected {
                t.button_hover_bg
            } else if resp.hovered() {
                egui::Color32::from_rgba_premultiplied(
                    t.tab_active_fg.r(),
                    t.tab_active_fg.g(),
                    t.tab_active_fg.b(),
                    15,
                )
            } else {
                egui::Color32::TRANSPARENT
            };

            ui.painter().rect_filled(row, egui::Rounding::ZERO, bg);
            if selected {
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(row.min, egui::vec2(3.0, row.height())),
                    egui::Rounding::ZERO,
                    t.accent,
                );
            }
            let fg = if selected { t.tab_active_fg } else { t.menu_fg };
            ui.painter().text(
                egui::pos2(row.min.x + 22.0, row.center().y),
                egui::Align2::LEFT_CENTER,
                ch.label(),
                egui::FontId::proportional(13.5),
                fg,
            );
            if resp.clicked() {
                self.chapter = *ch;
            }
            cursor_y += 40.0;
        }

        let content_rect = egui::Rect::from_min_max(egui::pos2(sep_x + 1.0, full.min.y), full.max);
        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );
        egui::ScrollArea::vertical()
            .id_salt("docs_content_scroll")
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                let pad = 44.0;
                ui.add_space(28.0);
                let w = (content_rect.width() - pad * 2.0).max(200.0);
                ui.set_max_width(w + pad * 2.0);
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.vertical(|ui| {
                        ui.set_max_width(w);
                        match self.chapter {
                            Chapter::GettingStarted => render_getting_started(ui, &t),
                            Chapter::TypesVariables => render_types_variables(ui, &t),
                            Chapter::FunctionsControl => render_functions_control(ui, &t),
                        }
                    });
                });
                ui.add_space(48.0);
            });
    }
}

fn h1(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.label(
        egui::RichText::new(text)
            .size(26.0)
            .strong()
            .color(t.tab_active_fg),
    );
    ui.add_space(8.0);
}

fn h2(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(22.0);
    ui.label(
        egui::RichText::new(text)
            .size(15.0)
            .strong()
            .color(t.accent),
    );
    ui.add_space(6.0);
}

fn para(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.label(egui::RichText::new(text).size(13.5).color(t.text_default));
    ui.add_space(4.0);
}

fn rule(ui: &mut egui::Ui, t: &Theme) {
    ui.add_space(14.0);
    let (r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(r, egui::Rounding::ZERO, t.border);
    ui.add_space(14.0);
}

fn code(ui: &mut egui::Ui, src: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            for line in src.lines() {
                let trimmed = line.trim_start();
                let color = if trimmed.starts_with('#') {
                    t.comment
                } else if trimmed.starts_with('!') {
                    t.keyword
                } else if trimmed.starts_with(':') {
                    t.type_name
                } else if trimmed.starts_with('"') {
                    t.string
                } else {
                    t.text_default
                };
                ui.label(
                    egui::RichText::new(line)
                        .monospace()
                        .size(13.0)
                        .color(color),
                );
            }
        });
    ui.add_space(8.0);
}

fn note(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(6.0);
    let a = t.accent;
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_premultiplied(
            a.r(),
            a.g(),
            a.b(),
            16,
        ))
        .rounding(egui::Rounding::same(5.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_premultiplied(a.r(), a.g(), a.b(), 80),
        ))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Note  ").size(12.5).strong().color(a));
                ui.label(egui::RichText::new(text).size(13.0).color(t.text_default));
            });
        });
    ui.add_space(8.0);
}

fn kv(ui: &mut egui::Ui, id: &str, cols: [&str; 3], rows: &[(&str, &str, &str)], t: &Theme) {
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(3)
                .spacing([28.0, 8.0])
                .show(ui, |ui| {
                    for h in cols {
                        ui.label(egui::RichText::new(h).size(12.0).strong().color(t.accent));
                    }
                    ui.end_row();
                    for (a, b, c) in rows {
                        ui.label(
                            egui::RichText::new(*a)
                                .monospace()
                                .size(13.0)
                                .color(t.type_name),
                        );
                        ui.label(egui::RichText::new(*b).size(13.0).color(t.text_default));
                        ui.label(
                            egui::RichText::new(*c)
                                .monospace()
                                .size(13.0)
                                .color(t.number),
                        );
                        ui.end_row();
                    }
                });
        });
}

fn render_getting_started(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Getting Started", t);
    para(ui, "Fractal is a statically-typed language. Every program starts with !start and ends with !end.", t);
    rule(ui, t);

    h2(ui, "Comments", t);
    para(
        ui,
        "Single-line comments start with #. Block comments are delimited by ###.",
        t,
    );
    code(
        ui,
        "!start\n# this is a comment\n\n###\n  multi-line block\n###\n\n!end",
        t,
    );

    h2(ui, "Hello, World", t);
    code(ui, "!start\n\n!func main() -> :int {\n    print(\"Hello, World!\\n\");\n    !return 0;\n}\n\n!end", t);
    note(ui, "Press Run in the toolbar to compile and execute. The compiler must be on PATH or beside this binary.", t);

    h2(ui, "Importing", t);
    code(ui, "!start\n!import \"./math.fr\";\n\n!func main() -> :int {\n    :float pi = math.pi;\n    !return 0;\n}\n\n!end", t);

    h2(ui, "Keyboard Shortcuts", t);
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new("shortcuts_grid")
                .num_columns(2)
                .spacing([32.0, 8.0])
                .show(ui, |ui| {
                    for (k, v) in [
                        ("Ctrl+S", "Save and format"),
                        ("Ctrl+Shift+S", "Save As"),
                        ("Ctrl+O", "Open file"),
                        ("Ctrl+N", "New tab"),
                        ("Ctrl+`", "Toggle terminal"),
                    ] {
                        ui.label(
                            egui::RichText::new(k)
                                .monospace()
                                .size(13.0)
                                .color(t.number),
                        );
                        ui.label(egui::RichText::new(v).size(13.0).color(t.text_default));
                        ui.end_row();
                    }
                });
        });
}

fn render_types_variables(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Types & Variables", t);
    para(ui, "Every variable must carry an explicit type annotation prefixed with ':'. The compiler rejects all mismatches at compile time.", t);
    rule(ui, t);

    h2(ui, "Primitive Types", t);
    kv(
        ui,
        "types_grid",
        ["Type", "Description", "Example"],
        &[
            (":int", "64-bit signed integer", "42, 0xFF, 0b1010"),
            (":float", "64-bit float", "3.14, 2.0e-5"),
            (":char", "Unicode character", "'A', '\\n'"),
            (":boolean", "Boolean", "true, false"),
            (":array", "Fixed-size sequence", ":array<:int, 8>"),
            (":list", "Dynamic sequence", ":list<:float>"),
        ],
        t,
    );

    h2(ui, "Declaration & Assignment", t);
    code(ui, ":int   count = 0;\n:float ratio = 0.618;\n:char  letter = 'F';\n\n# compound assignment\ncount += 1;\ncount *= 2;", t);

    h2(ui, "NULL", t);
    para(
        ui,
        "NULL represents the absence of a value and can be assigned to any type.",
        t,
    );
    code(
        ui,
        ":int ptr = NULL;\n\n!if (ptr == NULL) {\n    # handle missing value\n}",
        t,
    );
}

fn render_functions_control(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Functions & Control Flow", t);
    para(
        ui,
        "Functions use !func. Branching uses !if / !else. Loops use !for and !while.",
        t,
    );
    rule(ui, t);

    h2(ui, "Functions", t);
    code(ui, "!func add(:int a, :int b) -> :int {\n    !return a + b;\n}\n\n!func is_even(:int n) -> :boolean {\n    !return (n % 2) == 0;\n}", t);
    code(
        ui,
        ":int     sum  = add(10, 32);\n:boolean even = is_even(sum);",
        t,
    );

    h2(ui, "Conditionals", t);
    code(ui, "!if (score >= 90) {\n    print(\"A\\n\");\n} !else !if (score >= 75) {\n    print(\"B\\n\");\n} !else {\n    print(\"F\\n\");\n}", t);
    note(ui, "Comparison operators:  ==  ~=  >  <  >=  <=", t);

    h2(ui, "While Loop", t);
    code(ui, ":int i = 0;\n!while (i < 10) {\n    i += 1;\n}", t);

    h2(ui, "For Loop", t);
    code(
        ui,
        ":int s = 0;\n!for (:int n = 1; n <= 100; n += 1) {\n    s += n;\n}\n# s == 5050",
        t,
    );

    h2(ui, "Full Example — Factorial", t);
    code(ui, "!start\n\n!func factorial(:int n) -> :int {\n    !if (n <= 1) { !return 1; }\n    !return n * factorial(n - 1);\n}\n\n!func main() -> :int {\n    print(factorial(10));\n    !return 0;\n}\n\n!end", t);
}
