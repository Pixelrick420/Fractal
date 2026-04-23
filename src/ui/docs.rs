use crate::ui::highlighter::Highlighter;
use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
enum Chapter {
    QuickReference,
    GettingStarted,
    TypesVariables,
    Operators,
    FunctionsControl,
    Structs,
    Modules,
    StdLib,
    CommonPatterns,
}

impl Chapter {
    fn label(self) -> &'static str {
        match self {
            Self::QuickReference => "Quick Reference",
            Self::GettingStarted => "Getting Started",
            Self::TypesVariables => "Types & Variables",
            Self::Operators => "Operators",
            Self::FunctionsControl => "Control Flow",
            Self::Structs => "Structs",
            Self::Modules => "Modules",
            Self::StdLib => "Standard Library",
            Self::CommonPatterns => "Common Patterns",
        }
    }
}

const CHAPTERS: &[Chapter] = &[
    Chapter::QuickReference,
    Chapter::GettingStarted,
    Chapter::TypesVariables,
    Chapter::Operators,
    Chapter::FunctionsControl,
    Chapter::Structs,
    Chapter::Modules,
    Chapter::StdLib,
    Chapter::CommonPatterns,
];

pub struct DocsWindow {
    pub open: bool,
    chapter: Chapter,
    theme: Theme,
    search_query: String,
    search_focused: bool,
}

impl DocsWindow {
    pub fn new(theme: Theme) -> Self {
        Self {
            open: false,
            chapter: Chapter::QuickReference,
            theme,
            search_query: String::new(),
            search_focused: false,
        }
    }

    pub fn update_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let full = ui.max_rect();

        ui.painter()
            .rect_filled(full, egui::CornerRadius::ZERO, t.panel_bg);

        let sidebar_w = 220.0;
        let sep_x = full.min.x + sidebar_w;

        ui.painter().rect_filled(
            egui::Rect::from_min_max(full.min, egui::pos2(sep_x, full.max.y)),
            egui::CornerRadius::ZERO,
            t.line_numbers_bg,
        );
        ui.painter().line_segment(
            [egui::pos2(sep_x, full.min.y), egui::pos2(sep_x, full.max.y)],
            egui::Stroke::new(1.0, t.border),
        );

        ui.painter().text(
            egui::pos2(full.min.x + 18.0, full.min.y + 20.0),
            egui::Align2::LEFT_CENTER,
            "DOCUMENTATION",
            egui::FontId::proportional(9.5),
            t.tab_inactive_fg,
        );

        let search_rect = egui::Rect::from_min_size(
            egui::pos2(full.min.x + 10.0, full.min.y + 36.0),
            egui::vec2(sidebar_w - 20.0, 28.0),
        );

        let search_stroke = if self.search_focused {
            t.accent
        } else {
            t.border
        };
        egui::Frame::new()
            .fill(t.button_bg)
            .corner_radius(egui::CornerRadius::same(5))
            .stroke(egui::Stroke::new(1.0, search_stroke))
            .show(
                &mut ui.new_child(egui::UiBuilder::new().max_rect(search_rect)),
                |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(7.0);
                        ui.label(
                            egui::RichText::new(ic::MAGNIFY)
                                .size(11.0)
                                .color(t.tab_inactive_fg),
                        );
                        ui.add_space(4.0);
                        let search_id = egui::Id::new("docs_search_field");
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .id(search_id)
                                .hint_text("Search docs...")
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Small),
                        );
                        self.search_focused = resp.has_focus();

                        if !self.search_query.is_empty() {
                            let clear_id = egui::Id::new("docs_search_clear");
                            let clear_rect = egui::Rect::from_center_size(
                                egui::pos2(search_rect.right() - 14.0, search_rect.center().y),
                                egui::vec2(16.0, 16.0),
                            );
                            let clear_resp =
                                ui.interact(clear_rect, clear_id, egui::Sense::click());
                            ui.painter().text(
                                clear_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                ic::TAB_CLOSE,
                                egui::FontId::proportional(10.0),
                                if clear_resp.hovered() {
                                    t.terminal_error
                                } else {
                                    t.tab_inactive_fg
                                },
                            );
                            if clear_resp.clicked() {
                                self.search_query.clear();
                            }
                        }
                    });
                },
            );

        let mut cursor_y = full.min.y + 76.0;
        let query_lower = self.search_query.to_lowercase();
        let searching = !query_lower.is_empty();
        let mut any_result = false;

        for ch in CHAPTERS {
            if searching {
                let hits = chapter_search_text(*ch)
                    .to_lowercase()
                    .match_indices(query_lower.as_str())
                    .count();
                if hits == 0 {
                    continue;
                }
                any_result = true;

                let row = egui::Rect::from_min_size(
                    egui::pos2(full.min.x, cursor_y),
                    egui::vec2(sidebar_w, 36.0),
                );
                let id = egui::Id::new(("doc_ch_s", ch.label()));
                let resp = ui.interact(row, id, egui::Sense::click());
                let selected = *ch == self.chapter;
                let hovered = resp.hovered();

                draw_chapter_row(ui, row, selected, hovered, ch.label(), &t);

                let badge_str = format!("{hits}");
                let badge_center = egui::pos2(row.right() - 18.0, row.center().y);
                let badge_rect = egui::Rect::from_center_size(badge_center, egui::vec2(22.0, 16.0));
                ui.painter().rect_filled(
                    badge_rect,
                    egui::CornerRadius::same(8),
                    egui::Color32::from_rgba_premultiplied(
                        t.accent.r(),
                        t.accent.g(),
                        t.accent.b(),
                        90,
                    ),
                );
                ui.painter().text(
                    badge_center,
                    egui::Align2::CENTER_CENTER,
                    &badge_str,
                    egui::FontId::proportional(10.0),
                    t.tab_active_fg,
                );

                if resp.clicked() {
                    self.chapter = *ch;
                }
                cursor_y += 36.0;
            } else {
                let row = egui::Rect::from_min_size(
                    egui::pos2(full.min.x, cursor_y),
                    egui::vec2(sidebar_w, 36.0),
                );
                let id = egui::Id::new(("doc_ch", ch.label()));
                let resp = ui.interact(row, id, egui::Sense::click());
                let selected = *ch == self.chapter;
                let hovered = resp.hovered();

                draw_chapter_row(ui, row, selected, hovered, ch.label(), &t);

                if resp.clicked() {
                    self.chapter = *ch;
                }
                cursor_y += 36.0;
            }
        }

        if searching && !any_result {
            ui.painter().text(
                egui::pos2(full.min.x + 18.0, cursor_y + 18.0),
                egui::Align2::LEFT_CENTER,
                "No results",
                egui::FontId::proportional(12.5),
                t.tab_inactive_fg,
            );
        }

        let content_rect = egui::Rect::from_min_max(egui::pos2(sep_x + 1.0, full.min.y), full.max);

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );

        let search_q = self.search_query.clone();
        let chapter = self.chapter;

        egui::ScrollArea::vertical()
            .id_salt("docs_content_scroll")
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                let pad = 48.0;
                ui.add_space(32.0);
                let w = (content_rect.width() - pad * 2.0).max(200.0);
                ui.set_max_width(w + pad * 2.0);
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.vertical(|ui| {
                        ui.set_max_width(w);
                        let t = &self.theme;
                        if !search_q.is_empty() {
                            render_search_header(ui, t, &search_q, chapter);
                        }
                        match chapter {
                            Chapter::QuickReference => render_quick_reference(ui, t),
                            Chapter::GettingStarted => render_getting_started(ui, t),
                            Chapter::TypesVariables => render_types_variables(ui, t),
                            Chapter::Operators => render_operators(ui, t),
                            Chapter::FunctionsControl => render_functions_control(ui, t),
                            Chapter::Structs => render_structs(ui, t),
                            Chapter::Modules => render_modules(ui, t),
                            Chapter::StdLib => render_stdlib(ui, t),
                            Chapter::CommonPatterns => render_common_patterns(ui, t),
                        }
                    });
                });
                ui.add_space(64.0);
            });
    }
}

fn draw_chapter_row(
    ui: &egui::Ui,
    row: egui::Rect,
    selected: bool,
    hovered: bool,
    label: &str,
    t: &Theme,
) {
    let bg = if selected {
        t.button_hover_bg
    } else if hovered {
        egui::Color32::from_rgba_premultiplied(
            t.tab_active_fg.r(),
            t.tab_active_fg.g(),
            t.tab_active_fg.b(),
            12,
        )
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.painter().rect_filled(row, egui::CornerRadius::ZERO, bg);

    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(row.min, egui::vec2(3.0, row.height())),
            egui::CornerRadius::ZERO,
            t.accent,
        );
    }

    let fg = if selected || hovered {
        t.tab_active_fg
    } else {
        t.menu_fg
    };

    ui.painter().text(
        egui::pos2(row.min.x + 18.0, row.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        fg,
    );
}

fn chapter_search_text(ch: Chapter) -> &'static str {
    match ch {
        Chapter::QuickReference => {
            "quick reference syntax cheat sheet keywords types operators !start !end \
             !func !if !for !while :int :float :array :list"
        }
        Chapter::GettingStarted => {
            "getting started program structure !start !end comments # ### import \
             keyboard shortcuts run compile Ctrl+S Ctrl+O terminal !exit exit code \
             hello world philosophy design goals strongly typed compiled compiler errors"
        }
        Chapter::TypesVariables => {
            "types variables int float char boolean void array list string declaration \
             assignment default values literals binary hex octal decimal 0b 0x 0o 0d \
             type casting :int :float :array :list strong typing static"
        }
        Chapter::Operators => {
            "operators arithmetic unary binary plus minus multiply divide modulo \
             bitwise NOT AND OR XOR shift left right logical !not !and !or comparison \
             greater less equal not-equal ~= assignment += -= *= /= precedence"
        }
        Chapter::FunctionsControl => {
            "functions !func return !return control flow !if !elif !else !for !while \
             !break !continue conditionals loops for loop while recursion factorial"
        }
        Chapter::Structs => {
            "structs struct user defined types :struct member access :: fields \
             nested struct fixed size Vec2 Vec3 Rect Particle initialise self-referential"
        }
        Chapter::Modules => {
            "modules !module !import file organization cross-file namespace scope \
             Constants helper functions library"
        }
        Chapter::StdLib => {
            "standard library print input append pop insert delete find len array list \
             io format string placeholder math import abs sqrt pow floor ceil min max"
        }
        Chapter::CommonPatterns => {
            "common patterns code recipes stack queue BST linked list sorting \
             bubble sort merge sort data structures algorithms"
        }
    }
}

fn render_search_header(ui: &mut egui::Ui, t: &Theme, query: &str, chapter: Chapter) {
    note(
        ui,
        &format!(
            "Showing chapter \"{}\" - keywords matching \"{}\" highlighted in sidebar.",
            chapter.label(),
            query
        ),
        t,
    );
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

fn h1(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.label(
        egui::RichText::new(text)
            .size(24.0)
            .strong()
            .color(t.tab_active_fg),
    );
    ui.add_space(4.0);
}

fn h2(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(26.0);
    ui.label(
        egui::RichText::new(text)
            .size(14.0)
            .strong()
            .color(t.accent),
    );
    ui.add_space(6.0);
}

fn h3(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(text)
            .size(12.5)
            .strong()
            .color(t.tab_active_fg),
    );
    ui.add_space(4.0);
}

fn para(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.label(egui::RichText::new(text).size(13.5).color(t.text_default));
    ui.add_space(4.0);
}

fn rule(ui: &mut egui::Ui, t: &Theme) {
    ui.add_space(10.0);
    let (r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(
        r,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgba_premultiplied(t.border.r(), t.border.g(), t.border.b(), 120),
    );
    ui.add_space(10.0);
}

fn code(ui: &mut egui::Ui, src: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let job =
                Highlighter::new(*t).highlight_to_layout_job(src, egui::FontId::monospace(12.5));
            ui.label(egui::WidgetText::LayoutJob(job.into()));
        });
    ui.add_space(8.0);
}

fn note(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_premultiplied(
            t.border.r(),
            t.border.g(),
            t.border.b(),
            60,
        ))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Note  ")
                        .size(12.5)
                        .strong()
                        .color(t.tab_active_fg),
                );
                ui.label(egui::RichText::new(text).size(13.0).color(t.tab_active_fg));
            });
        });
    ui.add_space(8.0);
}

fn warning(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_premultiplied(
            t.border.r(),
            t.border.g(),
            t.border.b(),
            60,
        ))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Warning  ")
                        .size(12.5)
                        .strong()
                        .color(t.tab_active_fg),
                );
                ui.label(egui::RichText::new(text).size(13.0).color(t.tab_active_fg));
            });
        });
    ui.add_space(8.0);
}

fn kv2(ui: &mut egui::Ui, id: &str, cols: [&str; 2], rows: &[(&str, &str)], t: &Theme) {
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(2)
                .spacing([32.0, 8.0])
                .show(ui, |ui| {
                    for h in cols {
                        ui.label(egui::RichText::new(h).size(11.5).strong().color(t.accent));
                    }
                    ui.end_row();
                    for (a, b) in rows {
                        ui.label(
                            egui::RichText::new(*a)
                                .monospace()
                                .size(12.5)
                                .color(t.type_name),
                        );
                        ui.label(egui::RichText::new(*b).size(13.0).color(t.text_default));
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

fn kv3(ui: &mut egui::Ui, id: &str, cols: [&str; 3], rows: &[(&str, &str, &str)], t: &Theme) {
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(3)
                .spacing([28.0, 8.0])
                .show(ui, |ui| {
                    for h in cols {
                        ui.label(egui::RichText::new(h).size(11.5).strong().color(t.accent));
                    }
                    ui.end_row();
                    for (a, b, c) in rows {
                        ui.label(
                            egui::RichText::new(*a)
                                .monospace()
                                .size(12.5)
                                .color(t.type_name),
                        );
                        ui.label(egui::RichText::new(*b).size(13.0).color(t.text_default));
                        ui.label(
                            egui::RichText::new(*c)
                                .monospace()
                                .size(12.5)
                                .color(t.number),
                        );
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

fn shortcuts_table(ui: &mut egui::Ui, id: &str, rows: &[(&str, &str)], t: &Theme) {
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    for (k, v) in rows {
                        ui.label(
                            egui::RichText::new(*k)
                                .monospace()
                                .size(12.5)
                                .color(t.number),
                        );
                        ui.label(egui::RichText::new(*v).size(13.0).color(t.text_default));
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

// ---------------------------------------------------------------------------
// Quick Reference
// ---------------------------------------------------------------------------

fn render_quick_reference(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Quick Reference", t);
    para(
        ui,
        "This page provides a quick syntax overview. Click chapters in the sidebar for detailed documentation.",
        t,
    );

    rule(ui, t);
    h2(ui, "Program Structure", t);
    code(ui, "!start\n    # your code here\n!end", t);

    rule(ui, t);
    h2(ui, "Keywords", t);
    kv2(
        ui,
        "keywords",
        ["Keyword", "Description"],
        &[
            ("!start / !end", "Program delimiters"),
            ("!func", "Declare a function"),
            ("!if / !elif / !else", "Conditional"),
            ("!for", "Counted loop"),
            ("!while", "Condition loop"),
            ("!return", "Return from function"),
            ("!break / !continue", "Loop control"),
            ("!import", "Import another file"),
            ("!module", "Define a module"),
            ("!exit", "Terminate program"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Types", t);
    kv2(
        ui,
        "types",
        ["Type", "Description"],
        &[
            (":int", "64-bit integer"),
            (":float", "64-bit float"),
            (":char", "Unicode character"),
            (":boolean", "true or false"),
            (":void", "Null type"),
            (":array<T, N>", "Fixed array"),
            (":list<T>", "Dynamic list"),
            (":struct<Name>", "User struct"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Operators", t);
    kv2(
        ui,
        "operators",
        ["Operator", "Description"],
        &[
            ("+ - * / %", "Arithmetic"),
            ("& | ^ ~", "Bitwise"),
            ("!not !and !or", "Logical"),
            ("== ~= > < >= <=", "Comparison"),
            ("+= -= *= /= %=", "Compound assign"),
            ("::", "Struct member"),
            ("->", "Return type"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Type Casting", t);
    para(ui, "Use :Type(expression) to convert between types:", t);
    code(
        ui,
        ":int n = :int(3.99);    # 3\n:float f = :float(5);    # 5.0\n:char c = :char(65);     # 'A'",
        t,
    );

    rule(ui, t);
    h2(ui, "Common Patterns", t);
    code(
        ui,
        "# Function\n!func add(:int a, :int b) -> :int {\n    !return a + b;\n}\n\n# For loop\n!for (:int i, 0, 10, 1) {\n    print(\"{}\", i);\n}\n\n# Struct\n:struct<Point> {\n    :int x;\n    :int y;\n};",
        t,
    );

    rule(ui, t);
    h2(ui, "Keyboard Shortcuts", t);
    shortcuts_table(
        ui,
        "shortcuts",
        &[
            ("Ctrl+S", "Save"),
            ("Ctrl+O", "Open"),
            ("Ctrl+N", "New tab"),
            ("Ctrl+D", "Toggle docs"),
            ("Ctrl+F", "Find"),
        ],
        t,
    );
}

// ---------------------------------------------------------------------------
// Getting Started
// ---------------------------------------------------------------------------

fn render_getting_started(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Getting Started", t);
    para(
        ui,
        "Welcome to Fractal - a statically-typed, strongly-typed language designed for \
        clarity and correctness.",
        t,
    );

    rule(ui, t);
    h2(ui, "Hello World", t);
    para(
        ui,
        "Every program must begin with !start and end with !end.",
        t,
    );
    code(ui, "!start\n    print(\"Hello, World!\\n\");\n!end", t);

    rule(ui, t);
    h2(ui, "Variables & Functions", t);
    para(ui, "A simple program with variables and a function:", t);
    code(
        ui,
        "!start\n    :int x = 10;\n    :int y = 20;\n\n    !func sum(:int a, :int b) -> :int {\n        !return a + b;\n    }\n\n    print(\"{} + {} = {}\", x, y, sum(x, y));\n!end",
        t,
    );

    rule(ui, t);
    h2(ui, "Loops & Conditionals", t);
    para(ui, "Using !for loop and !if statement:", t);
    code(
        ui,
        "!start\n    :int total = 0;\n\n    !for (:int i, 1, 6, 1) {\n        !if (i % 2 == 0) {\n            total = total + i;\n        }\n    }\n\n    print(\"Sum of evens 1-5: {}\", total);\n!end",
        t,
    );

    rule(ui, t);
    h2(ui, "Why Fractal?", t);
    para(ui, "Fractal was built around three principles:", t);
    kv2(
        ui,
        "philosophy",
        ["Principle", "What it means"],
        &[
            (
                "No implicit casts",
                "Every type conversion is explicit with :Type(expr)",
            ),
            ("Compile-time safety", "Type errors caught before running"),
            (
                "Minimal syntax",
                "Keywords use ! prefix, never conflict with variables",
            ),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Comments", t);
    code(
        ui,
        "# single-line comment\n\n###\n    multi-line\n    comment\n###",
        t,
    );

    rule(ui, t);
    h2(ui, "Importing Files", t);
    code(ui, "!import \"math\";\n!import \"./utils.fr\";", t);

    rule(ui, t);
    h2(ui, "Compiler Errors", t);
    para(ui, "The compiler catches type errors at compile time:", t);
    code(
        ui,
        "!start\n    :int a = 10;\n    :float b = 3.0;\n\n    # ERROR: cannot assign float to int\n    :int c = b;\n\n    # CORRECT: use explicit cast\n    :int d = :int(b);   # 3\n!end",
        t,
    );

    rule(ui, t);
    h2(ui, "Keyboard Shortcuts", t);
    shortcuts_table(
        ui,
        "shortcuts",
        &[
            ("Ctrl+S", "Save and format"),
            ("Ctrl+O", "Open file"),
            ("Ctrl+N", "New tab"),
            ("Ctrl+D", "Toggle docs"),
            ("Ctrl+F", "Find in file"),
        ],
        t,
    );
}

// ---------------------------------------------------------------------------
// Types & Variables
// ---------------------------------------------------------------------------

fn render_types_variables(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Types & Variables", t);
    para(
        ui,
        "Fractal is statically typed. Every variable must have an explicit type annotation.",
        t,
    );

    rule(ui, t);
    h2(ui, "Simple Types", t);
    kv3(
        ui,
        "simple_types",
        ["Type", "Description", "Default"],
        &[
            (":int", "64-bit signed integer", "0"),
            (":float", "64-bit IEEE 754 float", "0.0"),
            (":char", "Unicode character", "'\\0'"),
            (":boolean", "true or false", "false"),
            (":void", "Null type", "-"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Collection Types", t);
    kv2(
        ui,
        "coll_types",
        ["Type", "Description"],
        &[
            (":array<T, N>", "Fixed-size array of N elements"),
            (":list<T>", "Dynamic list"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Integer Literals", t);
    shortcuts_table(
        ui,
        "int_lit",
        &[
            ("255", "Decimal"),
            ("0xFF", "Hexadecimal"),
            ("0b1111", "Binary"),
            ("0o377", "Octal"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Float Literals", t);
    shortcuts_table(
        ui,
        "float_lit",
        &[("1.5", "Plain"), ("1.5e6", "Scientific")],
        t,
    );

    rule(ui, t);
    h2(ui, "Declarations", t);
    code(
        ui,
        ":int     count = 42;\n:float   ratio = 0.618;\n:char    letter = 'F';\n:boolean flag = true;\n\n# Default values\n:int zero;     # 0\n:float f;    # 0.0",
        t,
    );

    rule(ui, t);
    h2(ui, "Type Casting", t);
    para(ui, "Use :Type(expression) syntax:", t);
    kv2(
        ui,
        "casts",
        ["Cast", "Effect"],
        &[
            (":int(expr)", "Convert to int (truncates)"),
            (":float(expr)", "Convert to float"),
            (":char(expr)", "Convert to char"),
            (":boolean(expr)", "Convert to boolean"),
        ],
        t,
    );
    code(
        ui,
        ":float f = :float(42);    # 42.0\n:int n = :int(3.99);    # 3\n:char c = :char(65);    # 'A'",
        t,
    );

    rule(ui, t);
    h2(ui, "Indexing", t);
    code(
        ui,
        ":int first = arr[0];\n:int i = 2;\n:int val = arr[i];",
        t,
    );
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

fn render_operators(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Operators", t);
    para(
        ui,
        "Fractal uses ~= for not-equal and ! prefix for logical operators.",
        t,
    );

    rule(ui, t);
    h2(ui, "Arithmetic", t);
    kv2(
        ui,
        "arith",
        ["Operator", "Description"],
        &[
            ("+ -", "Unary plus/negate"),
            ("+", "Addition"),
            ("-", "Subtraction"),
            ("*", "Multiplication"),
            ("/", "Division"),
            ("%", "Modulo (int only)"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Bitwise (int only)", t);
    kv2(
        ui,
        "bitwise",
        ["Operator", "Description"],
        &[
            ("~", "NOT"),
            ("&", "AND"),
            ("|", "OR"),
            ("^", "XOR"),
            ("<< >>", "Shift"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Logical (boolean)", t);
    kv2(
        ui,
        "logical",
        ["Operator", "Description"],
        &[("!not", "NOT"), ("!and", "AND"), ("!or", "OR")],
        t,
    );
    code(
        ui,
        ":boolean a = true;\n:boolean b = false;\n:boolean c = !not a;        # false\n:boolean d = a !and b;      # false\n:boolean e = a !or b;       # true",
        t,
    );

    rule(ui, t);
    h2(ui, "Comparison", t);
    kv2(
        ui,
        "cmp",
        ["Operator", "Description"],
        &[
            (">", "Greater than"),
            ("<", "Less than"),
            (">=", "> or equal"),
            ("<=", "< or equal"),
            ("==", "Equal"),
            ("~=", "Not equal"),
        ],
        t,
    );
    note(ui, "Fractal uses ~= for not-equal, not !=", t);

    rule(ui, t);
    h2(ui, "Assignment", t);
    kv2(
        ui,
        "assign",
        ["Operator", "Equivalent"],
        &[
            ("=", "Assign"),
            ("+=", "a = a + b"),
            ("-=", "a = a - b"),
            ("*=", "a = a * b"),
            ("/=", "a = a / b"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Precedence", t);
    para(
        ui,
        "Operators evaluated in this order (highest to lowest):",
        t,
    );
    code(
        ui,
        "1. unary: - ~ !not\n2. * / %\n3. + -\n4. << >>\n5. &\n6. ^\n7. |\n8. == ~= > < >= <=\n9. !and\n10. !or",
        t,
    );
}

// ---------------------------------------------------------------------------
// Functions & Control Flow
// ---------------------------------------------------------------------------

fn render_functions_control(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Control Flow", t);
    para(
        ui,
        "Fractal has !func, !if, !for, and !while. All blocks use curly braces.",
        t,
    );

    rule(ui, t);
    h2(ui, "Functions", t);
    code(
        ui,
        "!func add(:int a, :int b) -> :int {\n    !return a + b;\n}\n\n!func greet(:char name) -> :void {\n    print(\"Hello {}\\n\", name);\n}",
        t,
    );
    note(
        ui,
        "!func definitions must appear at the top level, not inside any block.",
        t,
    );

    rule(ui, t);
    h2(ui, "Conditionals", t);
    code(
        ui,
        "!if (x > 0) {\n    print(\"positive\\n\");\n}\n!elif (x < 0) {\n    print(\"negative\\n\");\n}\n!else {\n    print(\"zero\\n\");\n}",
        t,
    );
    warning(
        ui,
        "The opening { must be on the same line as the condition.",
        t,
    );

    rule(ui, t);
    h2(ui, "For Loop", t);
    code(
        ui,
        "# Count 0 to 9\n!for (:int i, 0, 10, 1) {\n    print(\"{}\", i);\n}\n\n# Nested loops\n!for (:int r, 0, 3, 1) {\n    !for (:int c, 0, 3, 1) {\n        print(\"({},{})\", r, c);\n    }\n}",
        t,
    );
    note(
        ui,
        "The loop variable must be :int and must not shadow outer variables.",
        t,
    );

    rule(ui, t);
    h2(ui, "While Loop", t);
    code(
        ui,
        ":int n = 10;\n!while (n > 0) {\n    print(\"{}\", n);\n    n = n - 1;\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Break & Continue", t);
    code(
        ui,
        "!for (:int i, 0, 100, 1) {\n    !if (i == 42) { !break; }\n}\n\n!for (:int i, 0, 10, 1) {\n    !if (i % 2 == 0) { !continue; }\n    print(\"{}\", i);  # prints 1,3,5,7,9\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Recursion", t);
    para(ui, "Functions can call themselves:", t);
    code(
        ui,
        "!func factorial(:int n) -> :int {\n    !if (n <= 1) { !return 1; }\n    !return n * factorial(n - 1);\n}\n\nprint(\"{}\", factorial(5));  # 120",
        t,
    );

    rule(ui, t);
    h2(ui, "Variable Scope", t);
    code(
        ui,
        ":int global = 10;\n\n!if (true) {\n    :int local = 20;\n    global = local;\n}\n# local is out of scope here",
        t,
    );
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

fn render_structs(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Structs", t);
    para(
        ui,
        "User-defined types that group related fields. Members accessed with ::.",
        t,
    );

    rule(ui, t);
    h2(ui, "Defining Structs", t);
    code(
        ui,
        ":struct<Vec2> {\n    :float x;\n    :float y;\n};\n\n:struct<Rect> {\n    :float x;\n    :float y;\n    :float w;\n    :float h;\n};",
        t,
    );

    rule(ui, t);
    h2(ui, "Nested Structs", t);
    code(
        ui,
        ":struct<Particle> {\n    :struct<Vec2> pos;\n    :struct<Vec2> vel;\n    :float mass;\n};",
        t,
    );

    rule(ui, t);
    h2(ui, "Initialisation", t);
    code(
        ui,
        ":struct<Vec2> origin = { x = 0.0, y = 0.0 };\n:struct<Vec2> p = { x = 3.0, y = 4.0 };",
        t,
    );

    rule(ui, t);
    h2(ui, "Member Access", t);
    code(ui, ":float x = p::x;\n:float y = p::y;", t);

    rule(ui, t);
    h2(ui, "Structs in Functions", t);
    code(
        ui,
        "!func distance(:struct<Vec2> a, :struct<Vec2> b) -> :float {\n    :float dx = b::x - a::x;\n    :float dy = b::y - a::y;\n    !return :float(dx * dx + dy * dy);\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Self-Referential", t);
    para(
        ui,
        "Structs can reference themselves for linked structures:",
        t,
    );
    code(
        ui,
        ":struct<Node> {\n    :int value;\n    :struct<Node> next;\n};\n\n# Create a linked list\n:struct<Node> head = { value = 1, next = !null };\nhead::next = { value = 2, next = !null };",
        t,
    );

    warning(
        ui,
        "Declaring a struct without an initializer produces a warning - structs have no default value.",
        t,
    );
}

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

fn render_modules(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Modules", t);
    para(
        ui,
        "Modules organize code across files. Use !module to define, !import to use.",
        t,
    );

    rule(ui, t);
    h2(ui, "Defining a Module", t);
    para(ui, "A module wraps related code:", t);
    code(
        ui,
        "!module Math {\n    :float pi = 3.14159;\n\n    !func square(:int n) -> :int {\n        !return n * n;\n    }\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Importing Files", t);
    para(ui, "Use !import to bring in another file:", t);
    code(ui, "!import \"./math.fr\";\n!import \"utils\";", t);

    rule(ui, t);
    h2(ui, "Module Example", t);
    para(ui, "File: constants.fr", t);
    code(
        ui,
        "!start\n    :float golden_ratio = 1.618;\n    :float e = 2.718;\n!end",
        t,
    );

    para(ui, "File: math.fr that imports constants:", t);
    code(
        ui,
        "!import \"./constants.fr\";\n\n!start\n    !func add(:int a, :int b) -> :int {\n        !return a + b;\n}\n!end",
        t,
    );

    para(ui, "Using in main.fr:", t);
    code(
        ui,
        "!import \"./math.fr\";\n\n!start\n    print(\"{}\", math::add(3, 4));\n!end",
        t,
    );

    rule(ui, t);
    h2(ui, "Best Practices", t);
    kv2(
        ui,
        "practices",
        ["Tip", "Description"],
        &[
            ("One module per file", "Keep related code together"),
            ("Meaningful names", "Use descriptive module names"),
            ("Clear boundaries", "Group related functions"),
        ],
        t,
    );
}

// ---------------------------------------------------------------------------
// Standard Library
// ---------------------------------------------------------------------------

fn render_stdlib(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Standard Library", t);
    para(ui, "Built-in functions for I/O and collections.", t);

    rule(ui, t);
    h2(ui, "I/O Functions", t);

    h3(ui, "print", t);
    code(
        ui,
        "print(\"Value: {}\", 42);\nprint(\"{} + {} = {}\", 1, 2, 3);",
        t,
    );

    h3(ui, "input", t);
    code(ui, ":int n;\ninput(\"{}\", n);", t);

    rule(ui, t);
    h2(ui, "List Functions", t);
    kv3(
        ui,
        "list_fns",
        ["Function", "Description", "Returns"],
        &[
            ("append(lst, v)", "Add to end", ":void"),
            ("pop(lst)", "Remove last", "T"),
            ("insert(lst, idx, v)", "Insert value at index", ":void"),
            ("delete(lst, idx)", "Delete at index", ":void"),
            ("find(lst, v)", "Find index", ":int"),
            ("len(lst)", "Get length", ":int"),
        ],
        t,
    );
    code(
        ui,
        ":list<:int> nums = [1, 2, 3];\nappend(nums, 4);\n:int last = pop(nums);            # 4\n:int idx = find(nums, 2);        # 1\ninsert(nums, 0, 99);              # insert 99 at index 0\ndelete(nums, 0);                  # delete index 0",
        t,
    );

    rule(ui, t);
    h2(ui, "Array Functions", t);
    kv3(
        ui,
        "arr_fns",
        ["Function", "Description", "Returns"],
        &[
            ("find(arr, v)", "Find index", ":int"),
            ("len(arr)", "Array length (fixed)", ":int"),
        ],
        t,
    );

    rule(ui, t);
    h2(ui, "Math Functions", t);
    kv3(
        ui,
        "math_fns",
        ["Function", "Description", "Returns"],
        &[
            ("abs(n)", "Absolute value", "int/float"),
            ("sqrt(n)", "Square root", ":float"),
            ("pow(a, b)", "a raised to b", ":float"),
            ("floor(n)", "Floor", ":int"),
            ("ceil(n)", "Ceiling", ":int"),
            ("min(a, b)", "Minimum", "int/float"),
            ("max(a, b)", "Maximum", "int/float"),
        ],
        t,
    );
    code(
        ui,
        ":int a = abs(-5);           # 5\n:float s = sqrt(16.0);        # 4.0\n:float p = pow(2.0, 3.0);     # 8.0\n:int f = floor(3.9);         # 3\n:int c = ceil(3.1);          # 4",
        t,
    );

    rule(ui, t);
    h2(ui, "Full Example", t);
    code(
        ui,
        "!start\n    :list<:int> evens = [];\n\n    !for (:int i, 1, 10, 1) {\n        !if (i % 2 == 0) {\n            append(evens, i);\n        }\n    }\n\n    :int count = len(evens);\n    print(\"Found {} evens\", count);\n!end",
        t,
    );
}

// ---------------------------------------------------------------------------
// Common Patterns
// ---------------------------------------------------------------------------

fn render_common_patterns(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Common Patterns", t);
    para(
        ui,
        "Code recipes for common data structures and algorithms.",
        t,
    );

    rule(ui, t);
    h2(ui, "Stack", t);
    code(
        ui,
        ":struct<Stack> {\n    :list<:int> data;\n    :int top;\n};\n\n!func make_stack() -> :struct<Stack> {\n    :struct<Stack> s;\n    s::data = [];\n    s::top = -1;\n    !return s;\n}\n\n!func push(:struct<Stack> s, :int v) -> :void {\n    append(s::data, v);\n    s::top = s::top + 1;\n}\n\n!func stack_pop(:struct<Stack> s) -> :int {\n    !if (s::top < 0) { !return -1; }\n    :int val = s::data[s::top];\n    s::top = s::top - 1;\n    !return val;\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Queue", t);
    code(
        ui,
        ":struct<Queue> {\n    :list<:int> data;\n    :int head;\n    :int tail;\n};\n\n!func make_queue() -> :struct<Queue> {\n    :struct<Queue> q;\n    q::data = [];\n    q::head = 0;\n    q::tail = 0;\n    !return q;\n}\n\n!func enqueue(:struct<Queue> q, :int v) -> :void {\n    append(q::data, v);\n    q::tail = q::tail + 1;\n}\n\n!func dequeue(:struct<Queue> q) -> :int {\n    !if (q::head >= q::tail) { !return -1; }\n    :int val = q::data[q::head];\n    q::head = q::head + 1;\n    !return val;\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Binary Search Tree", t);
    code(
        ui,
        ":struct<BSTNode> {\n    :int value;\n    :int left;\n    :int right;\n};\n\n!func insert(:int val, :list<:struct<BSTNode>> nodes, :int root) -> :void {\n    :int cur = root;\n    !for (:int i, 0, 1000, 1) {\n        :struct<BSTNode> n = nodes[cur];\n        !if (val < n::value) {\n            !if (n::left == -1) { n::left = val; !break; }\n            cur = n::left;\n        } !else {\n            !if (n::right == -1) { n::right = val; !break; }\n            cur = n::right;\n        }\n    }\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Bubble Sort", t);
    code(
        ui,
        "!func bubble_sort(:list<:int> arr) -> :void {\n    :int n = len(arr);\n\n    !for (:int i, 0, n, 1) {\n        !for (:int j, 0, n - i - 1, 1) {\n            !if (arr[j] > arr[j + 1]) {\n                :int tmp = arr[j];\n                arr[j] = arr[j + 1];\n                arr[j + 1] = tmp;\n            }\n        }\n    }\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Merge Sort", t);
    code(
        ui,
        "!func merge(:list<:int> a, :list<:int> b) -> :list<:int> {\n    :list<:int> result = [];\n    :int i = 0;\n    :int j = 0;\n\n    !while (i < len(a) !and j < len(b)) {\n        !if (a[i] <= b[j]) {\n            append(result, a[i]);\n            i = i + 1;\n        } !else {\n            append(result, b[j]);\n            j = j + 1;\n        }\n    }\n\n    !while (i < len(a)) {\n        append(result, a[i]);\n        i = i + 1;\n    }\n\n    !while (j < len(b)) {\n        append(result, b[j]);\n        j = j + 1;\n    }\n\n    !return result;\n}",
        t,
    );

    rule(ui, t);
    h2(ui, "Linked List", t);
    code(
        ui,
        ":struct<Node> {\n    :int value;\n    :struct<Node> next;\n};\n\n# Traverse list\n!func traverse(:struct<Node> head) -> :void {\n    :struct<Node> cur = head;\n    !while (cur ~= !null) {\n        print(\"{}\", cur::value);\n        cur = cur::next;\n    }\n}",
        t,
    );
}
