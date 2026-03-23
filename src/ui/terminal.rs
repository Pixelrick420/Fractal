use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;
use egui::Vec2;
use egui_term::{
    BackendSettings, ColorPalette, PtyEvent, TerminalBackend, TerminalTheme, TerminalView,
};
use std::sync::mpsc::{self, Receiver};

pub struct Terminal {
    pub minimized: bool,
    saved_height: f32,
    theme: Theme,
    backend: TerminalBackend,
    pty_receiver: Receiver<(u64, PtyEvent)>,
}

impl Terminal {
    pub fn new(theme: Theme, ctx: &egui::Context) -> Self {
        let (pty_sender, pty_receiver) = mpsc::channel();
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let backend = TerminalBackend::new(
            0,
            ctx.clone(),
            pty_sender.clone(),
            BackendSettings {
                shell,
                ..Default::default()
            },
        )
        .expect("failed to create terminal backend");

        Self {
            minimized: false,
            saved_height: 220.0,
            theme,
            backend,
            pty_receiver,
        }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn toggle_minimized(&mut self) {
        self.minimized = !self.minimized;
    }

    pub fn run_binary(&mut self, bin_path: &std::path::Path) {
        let cmd = format!("{}\n", bin_path.to_string_lossy());
        self.backend
            .process_command(egui_term::BackendCommand::Write(cmd.into_bytes()));
        self.minimized = false;
    }

    pub fn run_binary_with_env(&mut self, bin_path: &std::path::Path, lock_path: &std::path::Path) {
        let cmd = format!(
            "FRACTAL_DEBUG_LOCK={} {}\n",
            lock_path.to_string_lossy(),
            bin_path.to_string_lossy()
        );
        self.backend
            .process_command(egui_term::BackendCommand::Write(cmd.into_bytes()));
        self.minimized = false;
    }

    pub fn clear(&mut self) {
        self.backend
            .process_command(egui_term::BackendCommand::Write(b"clear\n".to_vec()));
    }

    pub fn append(&mut self, text: &str) {
        self.backend
            .process_command(egui_term::BackendCommand::Write(text.as_bytes().to_vec()));
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        while let Ok(_event) = self.pty_receiver.try_recv() {}

        let t = self.theme;
        let header_h = 32.0;

        let frame = egui::Frame::new()
            .fill(t.terminal_bg)
            .inner_margin(egui::Margin {
                left: 12,
                right: 12,
                top: 0,
                bottom: 0,
            });

        if self.minimized {
            let resp = egui::TopBottomPanel::bottom("terminal_panel_mini")
                .frame(frame)
                .resizable(false)
                .exact_height(header_h)
                .show_separator_line(false)
                .show(ctx, |ui| {
                    let (_, do_toggle) = self.draw_header(ui, t);
                    if do_toggle {
                        self.minimized = !self.minimized;
                    }
                });
            self.draw_border(ctx, resp.response.rect, t);
        } else {
            let resp = egui::TopBottomPanel::bottom("terminal_panel_expanded")
                .frame(frame)
                .resizable(true)
                .min_height(80.0)
                .max_height(600.0)
                .default_height(self.saved_height)
                .show_separator_line(false)
                .show(ctx, |ui| {
                    let (do_clear, do_toggle) = self.draw_header(ui, t);
                    if do_clear {
                        self.clear();
                    }
                    if do_toggle {
                        self.minimized = !self.minimized;
                    }

                    let avail = ui.available_size();
                    let term_theme = TerminalTheme::new(Box::new(theme_to_palette(&t)));

                    let panel_rect = ui.clip_rect();
                    let pointer_in_panel = ui.ctx().input(|i| {
                        i.pointer
                            .hover_pos()
                            .map_or(false, |p| panel_rect.contains(p))
                    });

                    let scroll_delta = ui.input(|i| i.raw_scroll_delta);
                    if pointer_in_panel && scroll_delta.y.abs() > 0.5 {
                        let lines = ((scroll_delta.y.abs() / 8.0) * 3.0).max(1.0) as i32;
                        if scroll_delta.y > 0.0 {
                            self.backend
                                .process_command(egui_term::BackendCommand::Scroll(lines));
                        } else {
                            self.backend
                                .process_command(egui_term::BackendCommand::Scroll(-lines));
                        }
                        ui.ctx().request_repaint();
                    }

                    let (is_dragging, drag_pos) =
                        ui.input(|i| (i.pointer.is_decidedly_dragging(), i.pointer.hover_pos()));
                    if is_dragging {
                        if let Some(pos) = drag_pos {
                            let panel_rect = ui.clip_rect();
                            let edge_zone = 40.0;
                            if pos.y < panel_rect.min.y + edge_zone && panel_rect.contains(pos) {
                                let t = 1.0 - (pos.y - panel_rect.min.y) / edge_zone;
                                let speed = ((t * 4.0).max(1.0)) as i32;
                                self.backend
                                    .process_command(egui_term::BackendCommand::Scroll(speed));
                                ui.ctx().request_repaint();
                            } else if pos.y > panel_rect.max.y - edge_zone
                                && panel_rect.contains(pos)
                            {
                                let t = (pos.y - (panel_rect.max.y - edge_zone)) / edge_zone;
                                let speed = ((t * 4.0).max(1.0)) as i32;
                                self.backend
                                    .process_command(egui_term::BackendCommand::Scroll(-speed));
                                ui.ctx().request_repaint();
                            }
                        }
                    }

                    let view = TerminalView::new(ui, &mut self.backend)
                        .set_focus(pointer_in_panel)
                        .set_theme(term_theme)
                        .set_size(Vec2::new(avail.x, avail.y));
                    ui.add(view);
                });

            let h = resp.response.rect.height();
            if h >= 80.0 {
                self.saved_height = h;
            }
            self.draw_border(ctx, resp.response.rect, t);
        }
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, t: Theme) -> (bool, bool) {
        let mut do_clear = false;
        let mut do_toggle = false;

        ui.add_space(5.0);
        ui.horizontal(|ui| {
            let toggle_icon = if self.minimized {
                ic::TERM_EXPAND
            } else {
                ic::TERM_COLLAPSE
            };
            let toggle_resp = ui.add(
                egui::Button::new(
                    egui::RichText::new(toggle_icon)
                        .size(13.0)
                        .color(t.tab_inactive_fg),
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE),
            );
            if toggle_resp.hovered() {
                ui.painter().rect_filled(
                    toggle_resp.rect.expand(2.0),
                    egui::CornerRadius::same(4),
                    t.button_hover_bg,
                );
                ui.painter().text(
                    toggle_resp.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    toggle_icon,
                    egui::FontId::proportional(13.0),
                    t.tab_active_fg,
                );
            }
            if toggle_resp.clicked() {
                do_toggle = true;
            }

            ui.label(
                egui::RichText::new(format!("{}  TERMINAL", ic::TERMINAL))
                    .size(11.0)
                    .color(t.tab_inactive_fg)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let clear_resp = ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(ic::TERM_CLEAR)
                                .size(13.0)
                                .color(t.tab_inactive_fg),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE),
                    )
                    .on_hover_text("Clear terminal");

                if clear_resp.hovered() {
                    ui.painter().rect_filled(
                        clear_resp.rect.expand(2.0),
                        egui::CornerRadius::same(4),
                        t.button_hover_bg,
                    );
                    ui.painter().text(
                        clear_resp.rect.center(),
                        egui::Align2::CENTER_CENTER,
                        ic::TERM_CLEAR,
                        egui::FontId::proportional(13.0),
                        t.terminal_error,
                    );
                }
                if clear_resp.clicked() {
                    do_clear = true;
                }
            });
        });

        (do_clear, do_toggle)
    }

    fn draw_border(&self, ctx: &egui::Context, rect: egui::Rect, t: Theme) {
        ctx.layer_painter(egui::LayerId::background()).line_segment(
            [rect.left_top(), rect.right_top()],
            egui::Stroke::new(1.0, t.border),
        );
    }
}

fn c32_hex(c: egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

fn theme_to_palette(t: &Theme) -> ColorPalette {
    use crate::ui::theme::ThemeVariant;
    match t.variant {
        ThemeVariant::Dark => ColorPalette {
            foreground: c32_hex(t.text_default),
            background: c32_hex(t.terminal_bg),
            black: "#0d1117".into(),
            red: c32_hex(t.terminal_error),
            green: c32_hex(t.struct_name),
            yellow: c32_hex(t.number),
            blue: c32_hex(t.type_name),
            magenta: c32_hex(t.fn_name),
            cyan: c32_hex(t.terminal_hint),
            white: c32_hex(t.text_default),
            bright_black: c32_hex(t.tab_inactive_fg),
            bright_red: "#ff7b7b".into(),
            bright_green: "#7ee787".into(),
            bright_yellow: "#ffd700".into(),
            bright_blue: "#79c0ff".into(),
            bright_magenta: "#d2a8ff".into(),
            bright_cyan: "#56d364".into(),
            bright_white: "#f0f6fc".into(),
            bright_foreground: None,
            dim_foreground: c32_hex(t.tab_inactive_fg),
            dim_black: "#0d1117".into(),
            dim_red: c32_hex(t.terminal_error),
            dim_green: c32_hex(t.struct_name),
            dim_yellow: c32_hex(t.number),
            dim_blue: c32_hex(t.type_name),
            dim_magenta: c32_hex(t.fn_name),
            dim_cyan: c32_hex(t.terminal_hint),
            dim_white: c32_hex(t.tab_inactive_fg),
        },
        ThemeVariant::Light => ColorPalette {
            foreground: c32_hex(t.text_default),
            background: c32_hex(t.terminal_bg),
            black: "#e8ebf5".into(),
            red: c32_hex(t.terminal_error),
            green: c32_hex(t.struct_name),
            yellow: c32_hex(t.number),
            blue: c32_hex(t.type_name),
            magenta: c32_hex(t.fn_name),
            cyan: c32_hex(t.terminal_hint),
            white: c32_hex(t.text_default),
            bright_black: c32_hex(t.tab_inactive_fg),
            bright_red: "#c01c1c".into(),
            bright_green: "#007000".into(),
            bright_yellow: "#945a00".into(),
            bright_blue: "#3c50c8".into(),
            bright_magenta: "#6f42c1".into(),
            bright_cyan: "#00787c".into(),
            bright_white: "#1c2236".into(),
            bright_foreground: None,
            dim_foreground: c32_hex(t.tab_inactive_fg),
            dim_black: "#e8ebf5".into(),
            dim_red: c32_hex(t.terminal_error),
            dim_green: c32_hex(t.struct_name),
            dim_yellow: c32_hex(t.number),
            dim_blue: c32_hex(t.type_name),
            dim_magenta: c32_hex(t.fn_name),
            dim_cyan: c32_hex(t.terminal_hint),
            dim_white: c32_hex(t.tab_inactive_fg),
        },
    }
}
