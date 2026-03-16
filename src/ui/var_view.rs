// src/ui/var_view.rs
//
// Floating egui::Window — Variable State & Call Stack
//
// Shows, for every active call frame from outermost to innermost:
//   • a labelled scope header  (e.g. "global", "fn: push")
//   • a table of  name | type | value  rows, sorted alphabetically
//
// Below the scope table there is a separate "Call Stack" strip.
// Both panels update live on every step().

use eframe::egui;
use super::debugger::DebugFrame;
use super::theme::Theme;

pub struct VarViewWindow {
    pub open:  bool,
    pub title: String,
}

impl VarViewWindow {
    pub fn new() -> Self {
        Self { open: false, title: "Variable State".into() }
    }

    pub fn show(
        &mut self,
        ctx:   &egui::Context,
        frame: &DebugFrame,
        theme: &Theme,
    ) {
        if !self.open { return; }

        let t = theme;

        egui::Window::new(&self.title)
            .id(egui::Id::new("fractal_var_view_window"))
            .default_size([360.0, 500.0])
            .min_size([240.0, 160.0])
            .resizable(true)
            .collapsible(true)
            .frame(
                egui::Frame::none()
                    .fill(t.panel_bg)
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .rounding(egui::Rounding::same(8.0))
                    .shadow(egui::Shadow {
                        offset: egui::vec2(0.0, 6.0),
                        blur:   18.0,
                        spread: 0.0,
                        color:  egui::Color32::from_black_alpha(100),
                    })
                    .inner_margin(egui::Margin::same(0.0)),
            )
            .open(&mut self.open)
            .show(ctx, |ui| {
                // ── title bar ────────────────────────────────────────────────
                egui::Frame::none()
                    .fill(t.tab_bar_bg)
                    .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⬡  Variables")
                                    .size(12.5).color(t.tab_active_fg).strong(),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let status = if frame.finished { "finished" }
                                             else if frame.error.is_some() { "error" }
                                             else { "running" };
                                let col = if frame.finished       { t.tab_inactive_fg }
                                          else if frame.error.is_some() { t.terminal_error }
                                          else { t.struct_name };
                                ui.label(egui::RichText::new(status).size(10.5).color(col));
                            });
                        });
                    });

                // ── step label strip ─────────────────────────────────────────
                let (sep, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.painter().rect_filled(sep, egui::Rounding::ZERO, t.border);

                egui::Frame::none()
                    .fill(egui::Color32::from_rgba_premultiplied(
                        t.accent.r(), t.accent.g(), t.accent.b(), 18))
                    .inner_margin(egui::Margin::symmetric(12.0, 5.0))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(&frame.step_label)
                                .size(11.0)
                                .color(t.accent)
                                .monospace(),
                        );
                    });

                let (sep2, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                ui.painter().rect_filled(sep2, egui::Rounding::ZERO, t.border);

                // ── scrollable body ──────────────────────────────────────────
                egui::ScrollArea::vertical()
                    .id_salt("var_view_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(6.0);

                        // Scope sections (outermost → innermost = bottom of vec = top of display)
                        let scopes: Vec<_> = frame.scopes.iter().rev().collect();
                        for (idx, scope) in scopes.iter().enumerate() {
                            let is_top = idx == 0;  // innermost = most recent call

                            egui::Frame::none()
                                .inner_margin(egui::Margin::symmetric(10.0, 2.0))
                                .show(ui, |ui| {
                                    // Scope header
                                    let hdr_text = if scope.label == "global" {
                                        "global scope".into()
                                    } else {
                                        format!("fn: {}", scope.label)
                                    };
                                    let hdr_col = if is_top { t.accent } else { t.tab_inactive_fg };
                                    ui.label(
                                        egui::RichText::new(hdr_text)
                                            .size(10.5).color(hdr_col).strong(),
                                    );
                                    ui.add_space(2.0);

                                    if scope.vars.is_empty() {
                                        ui.label(
                                            egui::RichText::new("  (no variables)")
                                                .size(11.0).color(t.tab_inactive_fg).italics(),
                                        );
                                    } else {
                                        draw_var_table(ui, scope.vars.as_slice(), is_top, t);
                                    }
                                });

                            if idx < scopes.len() - 1 {
                                ui.add_space(3.0);
                                let (sr, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                                ui.painter().rect_filled(sr, egui::Rounding::ZERO,
                                    egui::Color32::from_rgba_premultiplied(
                                        t.border.r(), t.border.g(), t.border.b(), 100));
                                ui.add_space(3.0);
                            }
                        }

                        // ── call stack ───────────────────────────────────────
                        ui.add_space(8.0);
                        let (cs_sep, _) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
                        ui.painter().rect_filled(cs_sep, egui::Rounding::ZERO, t.border);
                        ui.add_space(4.0);

                        egui::Frame::none()
                            .inner_margin(egui::Margin::symmetric(10.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("call stack")
                                        .size(10.5).color(t.tab_inactive_fg).strong(),
                                );
                                ui.add_space(3.0);

                                let stack = &frame.call_stack;
                                for (i, name) in stack.iter().enumerate().rev() {
                                    let is_cur = i == stack.len() - 1;
                                    let frame_col = if is_cur { t.accent } else { t.tab_inactive_fg };
                                    let depth_str = "  ".repeat(stack.len() - 1 - i);

                                    ui.horizontal(|ui| {
                                        if is_cur {
                                            ui.label(egui::RichText::new("▶").size(10.0).color(t.accent));
                                        } else {
                                            ui.label(egui::RichText::new("  ").size(10.0));
                                        }
                                        ui.label(
                                            egui::RichText::new(format!("{}{}", depth_str, name))
                                                .size(11.5).color(frame_col).monospace(),
                                        );
                                    });
                                }
                            });

                        // ── error banner ─────────────────────────────────────
                        if let Some(err) = &frame.error {
                            ui.add_space(6.0);
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgba_premultiplied(
                                    t.terminal_error.r(), t.terminal_error.g(),
                                    t.terminal_error.b(), 30))
                                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("⚠  {}", err))
                                            .size(11.0).color(t.terminal_error),
                                    );
                                });
                        }

                        ui.add_space(8.0);
                    });
            });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Variable table renderer
// ─────────────────────────────────────────────────────────────────────────────

fn draw_var_table(
    ui:     &mut egui::Ui,
    vars:   &[super::debugger::VarRow],
    active: bool,
    t:      &Theme,
) {
    let col_name_w  = 100.0_f32;
    let col_type_w  =  64.0_f32;
    let row_h       =  20.0_f32;
    let available_w = ui.available_width();

    // Header
    let hdr_h = 18.0_f32;
    let (hdr_rect, _) = ui.allocate_exact_size(egui::vec2(available_w, hdr_h), egui::Sense::hover());
    ui.painter().rect_filled(hdr_rect,
        egui::Rounding::same(3.0),
        egui::Color32::from_rgba_premultiplied(t.border.r(), t.border.g(), t.border.b(), 80));

    let hcol = t.tab_inactive_fg;
    let hfont = egui::FontId::proportional(9.5);
    ui.painter().text(egui::pos2(hdr_rect.left() + 6.0, hdr_rect.center().y),
        egui::Align2::LEFT_CENTER, "NAME", hfont.clone(), hcol);
    ui.painter().text(egui::pos2(hdr_rect.left() + col_name_w + 6.0, hdr_rect.center().y),
        egui::Align2::LEFT_CENTER, "TYPE", hfont.clone(), hcol);
    ui.painter().text(egui::pos2(hdr_rect.left() + col_name_w + col_type_w + 6.0, hdr_rect.center().y),
        egui::Align2::LEFT_CENTER, "VALUE", hfont, hcol);

    // Rows
    for (idx, row) in vars.iter().enumerate() {
        let (row_rect, _) = ui.allocate_exact_size(egui::vec2(available_w, row_h), egui::Sense::hover());

        // zebra stripe
        if idx % 2 == 0 {
            ui.painter().rect_filled(row_rect, egui::Rounding::ZERO,
                egui::Color32::from_rgba_premultiplied(
                    t.editor_bg.r(), t.editor_bg.g(), t.editor_bg.b(), 120));
        }

        let text_col = if active { t.tab_active_fg } else { t.tab_inactive_fg };
        let font     = egui::FontId::monospace(11.0);
        let ty_col   = t.type_name;
        let val_col  = t.number;

        ui.painter().text(
            egui::pos2(row_rect.left() + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &row.name,
            font.clone(), text_col,
        );
        ui.painter().text(
            egui::pos2(row_rect.left() + col_name_w + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &row.type_label,
            font.clone(), ty_col,
        );

        // Truncate long values.
        let val_str: &str = &row.value;
        let short_val = if val_str.len() > 34 {
            format!("{}…", &val_str[..33])
        } else {
            val_str.to_string()
        };

        ui.painter().text(
            egui::pos2(row_rect.left() + col_name_w + col_type_w + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            &short_val,
            font, val_col,
        );
    }
}