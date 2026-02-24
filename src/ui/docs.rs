use super::theme::Theme;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
pub enum DocChapter {
    GettingStarted,
    TypesAndVariables,
    FunctionsAndControl,
}

impl DocChapter {
    fn title(&self) -> &'static str {
        match self {
            DocChapter::GettingStarted => "Getting Started",
            DocChapter::TypesAndVariables => "Types & Variables",
            DocChapter::FunctionsAndControl => "Functions & Control Flow",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            DocChapter::GettingStarted => "1.",
            DocChapter::TypesAndVariables => "2.",
            DocChapter::FunctionsAndControl => "3.",
        }
    }
}

pub struct DocsPanel {
    pub current_chapter: DocChapter,
    theme: Theme,
}

impl DocsPanel {
    pub fn new(theme: Theme) -> Self {
        Self {
            current_chapter: DocChapter::GettingStarted,
            theme,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let chapters = [
            DocChapter::GettingStarted,
            DocChapter::TypesAndVariables,
            DocChapter::FunctionsAndControl,
        ];

        let full_rect = ui.available_rect_before_wrap();
        let sidebar_w = 210.0;
        let sep_w = 1.0;

        let sidebar_rect =
            egui::Rect::from_min_size(full_rect.min, egui::vec2(sidebar_w, full_rect.height()));
        let content_rect = egui::Rect::from_min_size(
            full_rect.min + egui::vec2(sidebar_w + sep_w + 1.0, 0.0),
            egui::vec2(
                (full_rect.width() - sidebar_w - sep_w - 1.0).max(0.0),
                full_rect.height(),
            ),
        );

        let mut sidebar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(sidebar_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );

        egui::Frame::none()
            .fill(self.theme.line_numbers_bg)
            .show(&mut sidebar_ui, |ui| {
                ui.set_min_size(egui::vec2(sidebar_w, full_rect.height()));
                ui.add_space(16.0);

                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.label(
                        egui::RichText::new("FRACTAL DOCS")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(90, 90, 90))
                            .strong(),
                    );
                });

                ui.add_space(12.0);

                for chapter in &chapters {
                    let is_selected = *chapter == self.current_chapter;
                    let text = format!("  {}  {}", chapter.icon(), chapter.title());

                    let label = egui::RichText::new(&text).size(13.0).color(if is_selected {
                        egui::Color32::from_rgb(100, 200, 255)
                    } else {
                        egui::Color32::from_rgb(170, 170, 170)
                    });

                    let resp = ui.add_sized(
                        [sidebar_w, 36.0],
                        egui::SelectableLabel::new(is_selected, label),
                    );

                    if resp.clicked() {
                        self.current_chapter = *chapter;
                    }

                    if is_selected {
                        let r = resp.rect;
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(r.min, egui::vec2(3.0, r.height())),
                            0.0,
                            egui::Color32::from_rgb(100, 200, 255),
                        );
                    }
                }
            });

        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                full_rect.min + egui::vec2(sidebar_w, 0.0),
                egui::vec2(sep_w, full_rect.height()),
            ),
            0.0,
            egui::Color32::from_rgb(55, 55, 55),
        );

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                ui.set_max_width(content_rect.width());

                let pad_h = 40.0_f32.min(content_rect.width() * 0.05);
                egui::Frame::none()
                    .inner_margin(egui::Margin {
                        left: pad_h,
                        right: pad_h,
                        top: 24.0,
                        bottom: 40.0,
                    })
                    .show(ui, |ui| {
                        let w = ui.available_width();
                        ui.set_max_width(w);

                        match self.current_chapter {
                            DocChapter::GettingStarted => self.show_getting_started(ui),
                            DocChapter::TypesAndVariables => self.show_types_variables(ui),
                            DocChapter::FunctionsAndControl => self.show_functions_control(ui),
                        }
                    });
            });

        ui.allocate_rect(full_rect, egui::Sense::hover());
    }

    fn heading(&self, ui: &mut egui::Ui, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(28.0)
                .color(egui::Color32::from_rgb(230, 230, 230))
                .strong(),
        );
        ui.add_space(6.0);
    }

    fn subheading(&self, ui: &mut egui::Ui, text: &str) {
        ui.add_space(22.0);
        ui.label(
            egui::RichText::new(text)
                .size(18.0)
                .color(egui::Color32::from_rgb(100, 200, 255))
                .strong(),
        );
        ui.add_space(6.0);
    }

    fn body(&self, ui: &mut egui::Ui, text: &str) {
        ui.label(
            egui::RichText::new(text)
                .size(14.0)
                .color(egui::Color32::from_rgb(195, 195, 195)),
        );
        ui.add_space(4.0);
    }

    fn code_block(&self, ui: &mut egui::Ui, code: &str) {
        ui.add_space(6.0);
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(20, 20, 28))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(16.0))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 62)))
            .show(ui, |ui| {
                for line in code.lines() {
                    let color = self.line_color(line);
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

    fn line_color(&self, line: &str) -> egui::Color32 {
        let t = line.trim_start();
        if t.starts_with('#') {
            egui::Color32::from_rgb(106, 153, 85)
        } else if t.starts_with('!') {
            egui::Color32::from_rgb(197, 134, 192)
        } else if t.starts_with(':') {
            egui::Color32::from_rgb(78, 201, 176)
        } else if t.starts_with('"') {
            egui::Color32::from_rgb(206, 145, 120)
        } else {
            egui::Color32::from_rgb(212, 212, 212)
        }
    }

    fn divider(&self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        ui.separator();
        ui.add_space(16.0);
    }

    fn note_box(&self, ui: &mut egui::Ui, text: &str) {
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(20, 30, 45))
            .rounding(egui::Rounding::same(4.0))
            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(50, 100, 160),
            ))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new("ℹ")
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 160, 255)),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(text)
                            .size(13.0)
                            .color(egui::Color32::from_rgb(170, 200, 240)),
                    );
                });
            });
        ui.add_space(8.0);
    }

    fn show_getting_started(&self, ui: &mut egui::Ui) {
        self.heading(ui, "Getting Started");
        self.body(
            ui,
            "Fractal is a statically-typed, expression-oriented language designed for \
             clarity and performance. Every Fractal program begins with !start and ends \
             with !end — anything outside those markers is ignored by the compiler.",
        );

        self.divider(ui);

        self.subheading(ui, "Program Structure");
        self.body(
            ui,
            "The !start and !end markers delimit your program. Comments begin with # and \
             run to the end of the line. Block comments are wrapped in ### on both sides.",
        );
        self.code_block(
            ui,
            "!start\n# Single-line comment\n\n###\n  Multi-line\n  block comment\n###\n\n!end",
        );

        self.subheading(ui, "Hello, World");
        self.body(
            ui,
            "A minimal Fractal program declares a main function that returns an integer \
             exit code. The !return keyword exits the function with a value.",
        );
        self.code_block(
            ui,
            "!start\n\n!func main() -> :int {\n    print(\"Hello, World!\\n\");\n    !return 0;\n}\n\n!end",
        );

        self.note_box(
            ui,
            "The compiler looks for fractal-compiler in your PATH or next to the editor \
             binary. Press ▶ Run in the toolbar to compile and execute.",
        );

        self.subheading(ui, "Importing Modules");
        self.body(
            ui,
            "Use !import to bring in another .fr source file. The path is relative to \
             the current file. Declarations from the imported file are accessible via \
             its filename (without extension) as a namespace prefix.",
        );
        self.code_block(
            ui,
            "!start\n!import \"./math.fr\";\n!import \"./utils.fr\";\n\n!func main() -> :int {\n    :float pi  = math.pi;\n    :int   cap = utils.max_value;\n    !return 0;\n}\n\n!end",
        );
        self.body(
            ui,
            "Circular imports are detected at compile time and produce an error with the \
             full import chain printed to the terminal.",
        );

        self.subheading(ui, "Editor Shortcuts");
        self.body(ui, "Keyboard shortcuts available in this editor:");
        ui.add_space(4.0);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(22, 22, 30))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(14.0))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 62)))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([24.0, 8.0])
                    .show(ui, |ui| {
                        let shortcuts = [
                            ("Ctrl + S", "Save and format the current file"),
                            ("Ctrl + Shift + S", "Save As — choose a new path"),
                            ("▶ Run", "Compile and run the current file"),
                        ];
                        for (key, desc) in &shortcuts {
                            ui.label(
                                egui::RichText::new(*key)
                                    .monospace()
                                    .color(egui::Color32::from_rgb(181, 206, 168)),
                            );
                            ui.label(
                                egui::RichText::new(*desc)
                                    .color(egui::Color32::from_rgb(190, 190, 190)),
                            );
                            ui.end_row();
                        }
                    });
            });
    }

    fn show_types_variables(&self, ui: &mut egui::Ui) {
        self.heading(ui, "Types & Variables");
        self.body(
            ui,
            "Fractal is statically typed — every variable must carry an explicit type \
             annotation. Types are written with a leading colon (e.g. :int, :float). \
             The compiler rejects any program with a type mismatch.",
        );

        self.divider(ui);

        self.subheading(ui, "Primitive Types");
        self.body(ui, "Fractal has six built-in primitive types:");
        ui.add_space(6.0);

        egui::Frame::none()
            .fill(egui::Color32::from_rgb(20, 20, 28))
            .rounding(egui::Rounding::same(6.0))
            .inner_margin(egui::Margin::same(14.0))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 48, 62)))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width());
                egui::Grid::new("types_grid")
                    .num_columns(3)
                    .spacing([28.0, 8.0])
                    .show(ui, |ui| {
                        for header in ["Type", "Description", "Examples"] {
                            ui.label(
                                egui::RichText::new(header)
                                    .strong()
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(100, 200, 255)),
                            );
                        }
                        ui.end_row();
                        for _ in 0..3 {
                            ui.separator();
                        }
                        ui.end_row();

                        let rows = [
                            (":int", "64-bit signed integer", "42, -7, 0xFF, 0b1010"),
                            (":float", "64-bit floating point", "3.14, 2.0e-5"),
                            (":char", "Single Unicode character", "'A', '\\n'"),
                            (":boolean", "Boolean value", "true, false"),
                            (":array", "Fixed-size typed sequence", ":array<:int, 8>"),
                            (":list", "Dynamically-sized sequence", ":list<:float>"),
                        ];
                        for (t, desc, ex) in &rows {
                            ui.label(
                                egui::RichText::new(*t)
                                    .monospace()
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(78, 201, 176)),
                            );
                            ui.label(
                                egui::RichText::new(*desc)
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(190, 190, 190)),
                            );
                            ui.label(
                                egui::RichText::new(*ex)
                                    .monospace()
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(181, 206, 168)),
                            );
                            ui.end_row();
                        }
                    });
            });
        ui.add_space(8.0);

        self.subheading(ui, "Variable Declaration");
        self.body(
            ui,
            "Variables are declared by writing the colon-prefixed type, then the name, \
             then = and an initial value. Every statement is terminated by a semicolon.",
        );
        self.code_block(
            ui,
            ":int     count  = 0;\n:float   ratio  = 0.618;\n:char    letter = 'F';\n:boolean active = true;\n\n# Integer literals in multiple bases\n:int hex = 0xFF;    # 255 — hexadecimal\n:int bin = 0b1010;  # 10  — binary\n:int oct = 0o17;    # 15  — octal",
        );

        self.subheading(ui, "Assignment & Compound Assignment");
        self.body(
            ui,
            "Once declared, variables can be updated with = or any of the compound \
             operators below. Compound operators apply an arithmetic or bitwise operation \
             and store the result back into the variable in one step.",
        );
        self.code_block(
            ui,
            ":int x = 10;\nx  = 20;   # plain assignment      → 20\nx += 5;    # add and assign        → 25\nx -= 3;    # subtract and assign   → 22\nx *= 2;    # multiply and assign   → 44\nx /= 4;    # divide and assign     → 11\nx %= 3;    # remainder and assign  → 2\nx &= 0xF;  # bitwise AND assign\nx |= 0x1;  # bitwise OR assign\nx ^= 0x3;  # bitwise XOR assign",
        );

        self.subheading(ui, "The NULL Literal");
        self.body(
            ui,
            "NULL represents the absence of a value. It can be assigned to pointer-like \
             or optional-typed variables and compared with == or ~=.",
        );
        self.code_block(
            ui,
            ":int ptr = NULL;\n\n!if (ptr == NULL) {\n    # handle uninitialized case\n}",
        );
    }

    fn show_functions_control(&self, ui: &mut egui::Ui) {
        self.heading(ui, "Functions & Control Flow");
        self.body(
            ui,
            "Functions are the primary unit of code organisation in Fractal. Control flow \
             uses !if / !else for branching and !for / !while for loops. All blocks are \
             delimited by curly braces { }.",
        );

        self.divider(ui);

        self.subheading(ui, "Declaring Functions");
        self.body(
            ui,
            "A function declaration uses !func followed by the name, a parenthesised \
             parameter list, an arrow ->, and the return type. Parameters use the same \
             colon-type syntax as variable declarations.",
        );
        self.code_block(
            ui,
            "!func add(:int a, :int b) -> :int {\n    !return a + b;\n}\n\n!func is_even(:int n) -> :boolean {\n    !return (n % 2) == 0;\n}",
        );

        self.subheading(ui, "Calling Functions");
        self.body(
            ui,
            "Call a function by writing its name followed by arguments in parentheses. \
             The result can be stored in a variable or used inline in any expression.",
        );
        self.code_block(
            ui,
            ":int     sum  = add(10, 32);     # sum == 42\n:boolean even = is_even(sum);    # even == true",
        );

        self.subheading(ui, "Conditionals — !if / !else");
        self.body(
            ui,
            "!if evaluates a condition and executes its block when the condition is true. \
             Chain additional branches with !else !if, and provide a fallback with !else.",
        );
        self.code_block(
            ui,
            ":int score = 85;\n\n!if (score >= 90) {\n    print(\"Grade: A\\n\");\n} !else !if (score >= 75) {\n    print(\"Grade: B\\n\");\n} !else {\n    print(\"Grade: F\\n\");\n}",
        );

        self.note_box(
            ui,
            "Comparison operators: == (equal), ~= (not equal), > < >= <=",
        );

        self.subheading(ui, "While Loops — !while");
        self.body(
            ui,
            "!while repeatedly executes its block for as long as the condition remains \
             true. The condition is checked before each iteration.",
        );
        self.code_block(
            ui,
            ":int i = 0;\n!while (i < 10) {\n    i += 1;\n}\n# i is now 10",
        );

        self.subheading(ui, "For Loops — !for");
        self.body(
            ui,
            "!for provides C-style counted iteration. The header contains three \
             semicolon-separated parts: an initializer, a loop condition, and a step.",
        );
        self.code_block(
            ui,
            ":int sum = 0;\n!for (:int n = 1; n <= 100; n += 1) {\n    sum += n;\n}\n# sum == 5050",
        );

        self.subheading(ui, "Complete Example — Factorial");
        self.body(
            ui,
            "Putting it all together: a recursive factorial function called from main.",
        );
        self.code_block(
            ui,
            "!start\n\n!func factorial(:int n) -> :int {\n    !if (n <= 1) {\n        !return 1;\n    }\n    !return n * factorial(n - 1);\n}\n\n!func main() -> :int {\n    :int result = factorial(10);\n    print(\"10! = \");\n    print(result);\n    print(\"\\n\");\n    !return 0;\n}\n\n!end",
        );
    }
}
