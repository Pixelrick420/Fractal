use super::highlighter::Highlighter;
use super::theme::Theme;
use eframe::egui;

pub struct CodeEditor {
    theme: Theme,
}

impl CodeEditor {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, code: &mut String) {
        let line_count = code.lines().count().max(1);
        let width_chars = line_count.to_string().len();
        let line_num_width = (width_chars as f32 * 9.0 + 24.0).max(44.0);

        let highlighter = Highlighter::new(self.theme);
        let mut layouter = move |ui: &egui::Ui, text: &str, wrap_width: f32| {
            let font_id = egui::FontId::monospace(14.0);
            let mut job = highlighter.highlight_to_layout_job(text, font_id);
            job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(job))
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    egui::Frame::none()
                        .fill(self.theme.line_numbers_bg)
                        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                        .show(ui, |ui| {
                            ui.set_width(line_num_width);
                            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                            ui.vertical(|ui| {
                                for n in 1..=line_count {
                                    ui.colored_label(
                                        self.theme.line_numbers_fg,
                                        format!("{:>width$}", n, width = width_chars),
                                    );
                                }
                            });
                        });

                    ui.add_space(4.0);

                    let text_edit = egui::TextEdit::multiline(code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(40)
                        .frame(false)
                        .layouter(&mut layouter);

                    ui.add(text_edit);
                });
            });
    }
}
