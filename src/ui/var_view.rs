use super::debugger::DebugFrame;
use super::theme::Theme;
use eframe::egui;

pub struct VarViewWindow {
    pub open: bool,
    pub title: String,
    /// Accumulated output from all steps so far
    output_history: String,
}

impl VarViewWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            title: "Variable State".into(),
            output_history: String::new(),
        }
    }

    pub fn push_output(&mut self, s: &str) {
        if !s.is_empty() {
            self.output_history.push_str(s);
        }
    }

    pub fn clear_output(&mut self) {
        self.output_history.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context, frame: &DebugFrame, theme: &Theme) {
        if !self.open {
            return;
        }

        let t = *theme;
        let output = self.output_history.clone();
        let mut open = self.open;

        egui::Window::new("⬡  Variable State")
            .id(egui::Id::new("fractal_var_view"))
            .open(&mut open)
            .default_size([380.0, 560.0])
            .min_size([260.0, 200.0])
            .resizable(true)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(t.panel_bg)
                    .stroke(egui::Stroke::new(1.0, t.border)),
            )
            .show(ctx, |ui| {
                // ── Header bar ────────────────────────────────────────────
                egui::Frame::new()
                    .fill(t.tab_bar_bg)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Variables")
                                    .size(12.5)
                                    .color(t.tab_active_fg)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (status, col) = if frame.finished {
                                        ("finished", t.tab_inactive_fg)
                                    } else if frame.error.is_some() {
                                        ("error", t.terminal_error)
                                    } else {
                                        ("running", t.struct_name)
                                    };
                                    ui.label(
                                        egui::RichText::new(status)
                                            .size(10.5)
                                            .color(col),
                                    );
                                },
                            );
                        });
                    });

                // Step label banner
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(
                        t.accent.r(), t.accent.g(), t.accent.b(), 22,
                    ))
                    .inner_margin(egui::Margin::symmetric(12, 5))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let line_txt = if frame.source_line > 0 {
                                format!("  line {}", frame.source_line)
                            } else {
                                String::new()
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "▶  {}{}",
                                    frame.step_label, line_txt
                                ))
                                .size(11.0)
                                .color(t.accent)
                                .monospace(),
                            );
                        });
                    });

                // Separator
                let avail_w = ui.available_width();
                let (sep, _) =
                    ui.allocate_exact_size(egui::vec2(avail_w, 1.0), egui::Sense::hover());
                ui.painter().rect_filled(sep, egui::CornerRadius::ZERO, t.border);

                // ── Scrollable body ───────────────────────────────────────
                egui::ScrollArea::vertical()
                    .id_salt("var_view_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(8.0);

                        // Scopes (innermost first)
                        let scopes: Vec<_> = frame.scopes.iter().rev().collect();
                        for (idx, scope) in scopes.iter().enumerate() {
                            let is_top = idx == 0;

                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                let hdr_text = if scope.label == "global" {
                                    "global scope".into()
                                } else {
                                    format!("fn: {}", scope.label)
                                };
                                let hdr_col =
                                    if is_top { t.accent } else { t.tab_inactive_fg };
                                ui.label(
                                    egui::RichText::new(hdr_text)
                                        .size(10.5)
                                        .color(hdr_col)
                                        .strong(),
                                );
                            });
                            ui.add_space(3.0);

                            if scope.vars.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(16.0);
                                    ui.label(
                                        egui::RichText::new("(no variables)")
                                            .size(11.0)
                                            .color(t.tab_inactive_fg)
                                            .italics(),
                                    );
                                });
                            } else {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    draw_var_table(
                                        ui,
                                        scope.vars.as_slice(),
                                        is_top,
                                        &t,
                                    );
                                });
                            }

                            ui.add_space(4.0);

                            // Scope separator
                            if idx < scopes.len() - 1 {
                                let avail = ui.available_width();
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    let (sr, _) = ui.allocate_exact_size(
                                        egui::vec2(avail - 20.0, 1.0),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().rect_filled(
                                        sr,
                                        egui::CornerRadius::ZERO,
                                        egui::Color32::from_rgba_premultiplied(
                                            t.border.r(),
                                            t.border.g(),
                                            t.border.b(),
                                            100,
                                        ),
                                    );
                                });
                                ui.add_space(4.0);
                            }
                        }

                        // ── Call Stack ────────────────────────────────────
                        ui.add_space(8.0);
                        let avail = ui.available_width();
                        let (cs_sep, _) = ui
                            .allocate_exact_size(egui::vec2(avail, 1.0), egui::Sense::hover());
                        ui.painter()
                            .rect_filled(cs_sep, egui::CornerRadius::ZERO, t.border);
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("call stack")
                                    .size(10.5)
                                    .color(t.tab_inactive_fg)
                                    .strong(),
                            );
                        });
                        ui.add_space(4.0);

                        let stack = &frame.call_stack;
                        for (i, name) in stack.iter().enumerate().rev() {
                            let is_cur = i == stack.len() - 1;
                            let frame_col =
                                if is_cur { t.accent } else { t.tab_inactive_fg };
                            let depth_indent = (stack.len() - 1 - i) as f32 * 12.0;

                            ui.horizontal(|ui| {
                                ui.add_space(16.0 + depth_indent);
                                if is_cur {
                                    ui.label(
                                        egui::RichText::new("▶")
                                            .size(10.0)
                                            .color(t.accent),
                                    );
                                } else {
                                    ui.label(egui::RichText::new("  ").size(10.0));
                                }
                                ui.label(
                                    egui::RichText::new(name)
                                        .size(11.5)
                                        .color(frame_col)
                                        .monospace(),
                                );
                            });
                        }

                        // ── Error banner ──────────────────────────────────
                        if let Some(err) = &frame.error {
                            ui.add_space(8.0);
                            let err_clone = err.clone();
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                egui::Frame::new()
                                    .fill(egui::Color32::from_rgba_premultiplied(
                                        t.terminal_error.r(),
                                        t.terminal_error.g(),
                                        t.terminal_error.b(),
                                        30,
                                    ))
                                    .inner_margin(egui::Margin::symmetric(8, 6))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(format!("⚠  {}", err_clone))
                                                .size(11.0)
                                                .color(t.terminal_error),
                                        );
                                    });
                            });
                        }

                        // ── Output panel ──────────────────────────────────
                        if !output.is_empty() {
                            ui.add_space(8.0);
                            let avail2 = ui.available_width();
                            let (o_sep, _) = ui.allocate_exact_size(
                                egui::vec2(avail2, 1.0),
                                egui::Sense::hover(),
                            );
                            ui.painter()
                                .rect_filled(o_sep, egui::CornerRadius::ZERO, t.border);
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                ui.label(
                                    egui::RichText::new("output so far")
                                        .size(10.5)
                                        .color(t.tab_inactive_fg)
                                        .strong(),
                                );
                            });
                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                egui::Frame::new()
                                    .fill(t.editor_bg)
                                    .corner_radius(egui::CornerRadius::same(4))
                                    .inner_margin(egui::Margin::same(8))
                                    .show(ui, |ui| {
                                        ui.set_width(avail2 - 28.0);
                                        ui.label(
                                            egui::RichText::new(&output)
                                                .size(11.0)
                                                .color(t.terminal_fg)
                                                .monospace(),
                                        );
                                    });
                            });
                        }

                        ui.add_space(12.0);
                    });
            });

        self.open = open;
    }
}

fn draw_var_table(
    ui: &mut egui::Ui,
    vars: &[super::debugger::VarRow],
    active: bool,
    t: &Theme,
) {
    let col_name_w = 100.0_f32;
    let col_type_w = 64.0_f32;
    let row_h = 22.0_f32;
    let available_w = (ui.available_width() - 10.0).max(200.0);

    // Header row
    let hdr_h = 18.0;
    let (hdr_rect, _) =
        ui.allocate_exact_size(egui::vec2(available_w, hdr_h), egui::Sense::hover());
    ui.painter().rect_filled(
        hdr_rect,
        egui::CornerRadius::same(3),
        egui::Color32::from_rgba_premultiplied(
            t.border.r(), t.border.g(), t.border.b(), 80,
        ),
    );

    let hcol = t.tab_inactive_fg;
    let hfont = egui::FontId::proportional(9.5);
    for (col_x, lbl) in [
        (hdr_rect.left() + 6.0, "NAME"),
        (hdr_rect.left() + col_name_w + 6.0, "TYPE"),
        (hdr_rect.left() + col_name_w + col_type_w + 6.0, "VALUE"),
    ] {
        ui.painter().text(
            egui::pos2(col_x, hdr_rect.center().y),
            egui::Align2::LEFT_CENTER,
            lbl,
            hfont.clone(),
            hcol,
        );
    }

    // Variable rows
    for (idx, row) in vars.iter().enumerate() {
        let (row_rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, row_h), egui::Sense::hover());

        if row.changed {
            ui.painter().rect_filled(
                row_rect,
                egui::CornerRadius::same(2),
                egui::Color32::from_rgba_premultiplied(
                    t.tab_dirty_dot.r(),
                    t.tab_dirty_dot.g(),
                    t.tab_dirty_dot.b(),
                    40,
                ),
            );
        } else if idx % 2 == 0 {
            ui.painter().rect_filled(
                row_rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_premultiplied(
                    t.editor_bg.r(), t.editor_bg.g(), t.editor_bg.b(), 120,
                ),
            );
        }

        let text_col = if active { t.tab_active_fg } else { t.tab_inactive_fg };
        let val_col = if row.changed { t.tab_dirty_dot } else { t.number };
        let font = egui::FontId::monospace(11.0);

        ui.painter().text(
            egui::pos2(row_rect.left() + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &row.name,
            font.clone(),
            text_col,
        );
        ui.painter().text(
            egui::pos2(row_rect.left() + col_name_w + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &row.type_label,
            font.clone(),
            t.type_name,
        );

        let val_str = &row.value;
        let short_val = if val_str.len() > 34 {
            format!("{}…", &val_str[..33])
        } else {
            val_str.clone()
        };

        ui.painter().text(
            egui::pos2(
                row_rect.left() + col_name_w + col_type_w + 6.0,
                row_rect.center().y,
            ),
            egui::Align2::LEFT_CENTER,
            &short_val,
            font,
            val_col,
        );

        if row.changed {
            ui.painter().circle_filled(
                egui::pos2(row_rect.right() - 8.0, row_rect.center().y),
                3.5,
                t.tab_dirty_dot,
            );
        }
    }
}