use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

pub struct Terminal {
    pub output: String,
    pub minimized: bool,

    panel_height: f32,

    saved_height: f32,
    theme: Theme,
}

impl Terminal {
    pub fn new(theme: Theme) -> Self {
        Self {
            output: String::new(),
            minimized: false,
            panel_height: 200.0,
            saved_height: 200.0,
            theme,
        }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
    pub fn append(&mut self, text: &str) {
        self.output.push_str(text);
        self.minimized = false;
    }
    pub fn clear(&mut self) {
        self.output.clear();
    }

    pub fn toggle_minimized(&mut self) {
        if self.minimized {
            self.panel_height = self.saved_height;
            self.minimized = false;
        } else {
            self.saved_height = self.panel_height;
            self.minimized = true;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let t = self.theme;
        let header_h = 32.0;

        egui::TopBottomPanel::bottom("terminal_panel")
            .frame(
                egui::Frame::none()
                    .fill(t.terminal_bg)
                    .inner_margin(egui::Margin {
                        left: 12.0,
                        right: 12.0,
                        top: 0.0,
                        bottom: 0.0,
                    }),
            )
            .resizable(!self.minimized)
            .min_height(if self.minimized { header_h } else { 80.0 })
            .max_height(600.0)
            .default_height(if self.minimized {
                header_h
            } else {
                self.panel_height
            })
            .show(ctx, |ui| {
                let top = ui.min_rect().left_top();
                ui.painter().line_segment(
                    [top, top + egui::vec2(ui.min_rect().width(), 0.0)],
                    egui::Stroke::new(1.0, t.border),
                );

                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    let toggle_icon = if self.minimized {
                        ic::TERM_EXPAND
                    } else {
                        ic::TERM_COLLAPSE
                    };
                    let toggle_resp = ui.add(
                        egui::Button::new(egui::RichText::new(toggle_icon).size(13.0).color(
                            if ui.rect_contains_pointer(egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(28.0, 24.0),
                            )) {
                                t.tab_active_fg
                            } else {
                                t.tab_inactive_fg
                            },
                        ))
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE),
                    );
                    if toggle_resp.hovered() {
                        ui.painter().rect_filled(
                            toggle_resp.rect.expand(2.0),
                            egui::Rounding::same(4.0),
                            t.button_hover_bg,
                        );
                    }
                    if toggle_resp.clicked() {
                        self.toggle_minimized();
                    }

                    ui.label(
                        egui::RichText::new(format!("{}  TERMINAL", ic::TERMINAL))
                            .size(11.0)
                            .color(t.tab_inactive_fg)
                            .strong(),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let clear_id = egui::Id::new("terminal_clear_btn");
                        let clear_size = egui::vec2(64.0, 24.0);
                        let (clear_rect, _) =
                            ui.allocate_exact_size(clear_size, egui::Sense::hover());
                        let clear_hovered = ui.rect_contains_pointer(clear_rect);

                        let clear_color = if clear_hovered {
                            t.terminal_error
                        } else {
                            t.tab_inactive_fg
                        };

                        ui.painter().text(
                            clear_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{}  Clear", ic::TERM_CLEAR),
                            egui::FontId::proportional(11.5),
                            clear_color,
                        );
                        let clear_resp = ui.interact(clear_rect, clear_id, egui::Sense::click());
                        if clear_resp.clicked() {
                            self.output.clear();
                        }
                    });
                });

                if self.minimized {
                    return;
                }

                ui.add_space(3.0);

                egui::Frame::none()
                    .fill(t.terminal_bg)
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.style_mut().override_text_style =
                                    Some(egui::TextStyle::Monospace);
                                if self.output.is_empty() {
                                    ui.label(
                                        egui::RichText::new(
                                            "No output yet. Press Run to compile and execute.",
                                        )
                                        .size(12.0)
                                        .color(
                                            egui::Color32::from_rgba_premultiplied(
                                                t.terminal_fg.r(),
                                                t.terminal_fg.g(),
                                                t.terminal_fg.b(),
                                                140,
                                            ),
                                        ),
                                    );
                                } else {
                                    for line in self.output.lines() {
                                        let lower = line.to_ascii_lowercase();
                                        let color = if lower.contains("error") {
                                            t.terminal_error
                                        } else if lower.contains("warning") {
                                            t.terminal_warning
                                        } else {
                                            t.terminal_fg
                                        };
                                        ui.label(
                                            egui::RichText::new(line)
                                                .size(12.5)
                                                .color(color)
                                                .monospace(),
                                        );
                                    }
                                }
                            });
                    });

                let rendered_h = ui.min_rect().height();
                if !self.minimized && rendered_h > header_h + 20.0 {
                    self.panel_height = rendered_h;

                    self.saved_height = rendered_h;
                }
            });
    }
}
