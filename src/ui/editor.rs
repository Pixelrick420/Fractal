use crate::ui::highlighter::Highlighter;
use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

pub struct CodeEditor {
    theme: Theme,
}

impl CodeEditor {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn show_with_id(
        &mut self,
        ui: &mut egui::Ui,
        code: &mut String,
        tab_id: usize,
        font_size: f32,
        show_line_numbers: bool,
        select_range: Option<(usize, usize)>,
    ) {
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            egui::Rounding::ZERO,
            self.theme.editor_bg,
        );

        let line_count = code.lines().count().max(1);
        let width_chars = line_count.to_string().len();
        let line_num_width = if show_line_numbers {
            (width_chars as f32 * 9.0 + 24.0).max(44.0)
        } else {
            0.0
        };

        let text_edit_id = egui::Id::new("code_editor").with(tab_id);

        let theme = self.theme;
        let highlighter = Highlighter::new(theme);
        let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let font_id = egui::FontId::monospace(font_size);
            let mut job = highlighter.highlight_to_layout_job(text, font_id);
            job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(job))
        };

        if select_range.is_some() {
            ui.ctx().request_repaint();
        }

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    if show_line_numbers {
                        egui::Frame::none()
                            .fill(self.theme.line_numbers_bg)
                            .inner_margin(egui::Margin::symmetric(8.0, 8.0))
                            .show(ui, |ui| {
                                ui.set_width(line_num_width);
                                ui.style_mut().override_text_style =
                                    Some(egui::TextStyle::Monospace);
                                ui.vertical(|ui| {
                                    for n in 1..=line_count {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:>width$}",
                                                n,
                                                width = width_chars
                                            ))
                                            .size(font_size - 1.0)
                                            .color(self.theme.line_numbers_fg),
                                        );
                                    }
                                });
                            });
                    }

                    let text_edit = egui::TextEdit::multiline(code)
                        .id(text_edit_id)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(40)
                        .frame(false)
                        .layouter(&mut layouter);

                    let mut output = text_edit.show(ui);

                    if let Some((byte_start, byte_end)) = select_range {
                        let char_start = byte_offset_to_char_index(code, byte_start);
                        let char_end = byte_offset_to_char_index(code, byte_end);

                        let cursor_start = output
                            .galley
                            .from_ccursor(egui::text::CCursor::new(char_start));
                        let cursor_end = output
                            .galley
                            .from_ccursor(egui::text::CCursor::new(char_end));

                        let origin = output.galley_pos;

                        let rows = &output.galley.rows;
                        let start_rect = output.galley.pos_from_cursor(&cursor_start);
                        let end_rect = output.galley.pos_from_cursor(&cursor_end);

                        let highlight_color = theme.selection;
                        let painter = ui.painter();

                        if start_rect.min.y == end_rect.min.y {
                            let rect = egui::Rect::from_min_max(
                                origin + start_rect.min.to_vec2(),
                                origin + end_rect.max.to_vec2(),
                            );
                            painter.rect_filled(rect, egui::Rounding::ZERO, highlight_color);
                        } else {
                            for row in rows {
                                let row_min_y = row.rect.min.y;
                                let row_max_y = row.rect.max.y;

                                if row_max_y <= start_rect.min.y || row_min_y >= end_rect.max.y {
                                    continue;
                                }

                                let x_start = if (row_min_y - start_rect.min.y).abs() < 1.0 {
                                    start_rect.min.x
                                } else {
                                    row.rect.min.x
                                };
                                let x_end = if (row_min_y - end_rect.min.y).abs() < 1.0 {
                                    end_rect.max.x
                                } else {
                                    row.rect.max.x
                                };

                                let rect = egui::Rect::from_min_max(
                                    origin + egui::vec2(x_start, row_min_y),
                                    origin + egui::vec2(x_end, row_max_y),
                                );
                                painter.rect_filled(rect, egui::Rounding::ZERO, highlight_color);
                            }
                        }

                        let abs_end = egui::Rect::from_min_size(
                            origin + end_rect.min.to_vec2(),
                            end_rect.size(),
                        );
                        let padded = abs_end.expand2(egui::vec2(0.0, font_size * 2.0));
                        ui.scroll_to_rect(padded, None);
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Some(cursor_range) = output.cursor_range {
                            let pos = cursor_range.primary.ccursor.index;
                            let indent = indent_for_line_above(code, pos);
                            if !indent.is_empty() {
                                use egui::TextBuffer as _;
                                code.insert_text(&indent, pos);
                                let new_pos = pos + indent.chars().count();
                                let new_ccursor = egui::text::CCursor::new(new_pos);
                                output.state.cursor.set_char_range(Some(
                                    egui::text::CCursorRange::one(new_ccursor),
                                ));
                                output.state.store(ui.ctx(), output.response.id);
                            }
                        }
                    }
                });
            });
    }
}

fn byte_offset_to_char_index(s: &str, byte_offset: usize) -> usize {
    let clamped = byte_offset.min(s.len());
    s[..clamped].chars().count()
}

pub enum EmptyStateAction {
    None,
    Open,
    New,
}

pub fn show_empty_state(ui: &mut egui::Ui, t: &Theme, full_rect: egui::Rect) -> EmptyStateAction {
    let mut action = EmptyStateAction::None;

    ui.painter().rect_filled(
        ui.available_rect_before_wrap(),
        egui::Rounding::ZERO,
        t.editor_bg,
    );

    let content_w = 320.0_f32.min(full_rect.width() - 64.0);
    let card_rect = egui::Rect::from_center_size(full_rect.center(), egui::vec2(content_w, 260.0));

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(card_rect), |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(ic::EMPTY_STATE)
                    .size(48.0)
                    .color(t.empty_fg),
            );
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new("No files open")
                    .size(18.0)
                    .color(t.empty_fg)
                    .strong(),
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Open a file or create a new one to get started.")
                    .size(12.5)
                    .color(t.empty_fg),
            );
            ui.add_space(24.0);

            ui.horizontal(|ui| {
                let pad = (content_w - 244.0).max(0.0) * 0.5;
                ui.add_space(pad);

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!("{}  Open File", ic::OPEN_FILE))
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(t.accent)
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(114.0, 34.0)),
                    )
                    .clicked()
                {
                    action = EmptyStateAction::Open;
                }

                ui.add_space(10.0);

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(format!("{}  New File", ic::NEW_FILE))
                                .size(13.0)
                                .color(t.accent),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::new(1.0, t.accent))
                        .rounding(egui::Rounding::same(6.0))
                        .min_size(egui::vec2(114.0, 34.0)),
                    )
                    .clicked()
                {
                    action = EmptyStateAction::New;
                }
            });

            ui.add_space(16.0);
        });
    });

    action
}

fn indent_for_line_above(text: &str, cursor_pos: usize) -> String {
    if cursor_pos == 0 {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    let prev_end = cursor_pos - 1;
    let prev_start = chars[..prev_end]
        .iter()
        .rposition(|&c| c == '\n')
        .map(|p| p + 1)
        .unwrap_or(0);
    chars[prev_start..prev_end]
        .iter()
        .take_while(|&&c| c == ' ' || c == '\t')
        .collect()
}
