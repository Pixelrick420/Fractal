use super::debugger::DebugSession;
use super::theme::Theme;
use eframe::egui;

pub struct TreeViewWindow {
    pub open: bool,
    pub title: String,
}

impl TreeViewWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            title: "AST Tree".into(),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        session: &mut DebugSession,
        active_node_id: usize,
        theme: &Theme,
    ) {
        if !self.open {
            return;
        }

        let t = *theme;
        let title = self.title.clone();
        let mut should_close = false;

        session.reveal_node(active_node_id);

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("fractal_tree_view"),
            egui::ViewportBuilder::default()
                .with_title(&title)
                .with_inner_size([440.0, 580.0])
                .with_min_inner_size([300.0, 200.0]),
            |ctx, _class| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    should_close = true;
                }

                apply_theme_to_ctx(ctx, &t);

                egui::TopBottomPanel::top("tree_header")
                    .frame(
                        egui::Frame::new()
                            .fill(t.tab_bar_bg)
                            .inner_margin(egui::Margin::symmetric(12, 8)),
                    )
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⬡  AST Tree")
                                    .size(12.5)
                                    .color(t.tab_active_fg)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let total = session.total_steps();
                                    let cur = session.cursor();
                                    ui.label(
                                        egui::RichText::new(format!("step {cur}/{total}"))
                                            .size(10.5)
                                            .color(t.tab_inactive_fg),
                                    );
                                    ui.add_space(8.0);

                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("⊟ Collapse all")
                                                    .size(10.5)
                                                    .color(t.tab_inactive_fg),
                                            )
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::new(1.0, t.border)),
                                        )
                                        .clicked()
                                    {
                                        session.collapse_all(0);

                                        if let Some(root) = session.tree.get_mut(0) {
                                            root.collapsed = false;
                                        }
                                    }
                                },
                            );
                        });
                    });

                let border_col = t.border;
                egui::TopBottomPanel::top("tree_header_sep")
                    .frame(
                        egui::Frame::new()
                            .fill(border_col)
                            .inner_margin(egui::Margin::ZERO),
                    )
                    .exact_height(1.0)
                    .show(ctx, |_| {});

                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(t.panel_bg))
                    .show(ctx, |ui| {
                        let scroll = egui::ScrollArea::both()
                            .id_salt("tree_view_scroll")
                            .auto_shrink([false; 2]);

                        scroll.show(ui, |ui| {
                            ui.add_space(6.0);

                            let mut toggle_id: Option<usize> = None;

                            let mut active_rect: Option<egui::Rect> = None;

                            draw_tree(
                                ui,
                                &session.tree,
                                0,
                                active_node_id,
                                &t,
                                &mut toggle_id,
                                &mut active_rect,
                            );

                            ui.add_space(12.0);

                            if let Some(id) = toggle_id {
                                session.toggle_collapsed(id);
                            }

                            if let Some(rect) = active_rect {
                                let padded = rect.expand2(egui::vec2(0.0, 40.0));
                                ui.scroll_to_rect(padded, None);
                            }
                        });
                    });
            },
        );

        if should_close {
            self.open = false;
        }
    }
}

fn draw_tree(
    ui: &mut egui::Ui,
    table: &[super::debugger::TreeNode],
    root_id: usize,
    active_node_id: usize,
    t: &Theme,
    toggle_out: &mut Option<usize>,
    active_rect: &mut Option<egui::Rect>,
) {
    draw_node(
        ui,
        table,
        root_id,
        active_node_id,
        t,
        true,
        toggle_out,
        active_rect,
    );
}

fn draw_node(
    ui: &mut egui::Ui,
    table: &[super::debugger::TreeNode],
    id: usize,
    active_node_id: usize,
    t: &Theme,
    is_last: bool,
    toggle_out: &mut Option<usize>,
    active_rect: &mut Option<egui::Rect>,
) {
    let Some(node) = table.get(id) else { return };

    let indent = node.depth as f32 * 18.0;
    let row_h = 24.0_f32;
    let is_active = id == active_node_id;
    let has_children = !node.children.is_empty();
    let is_collapsed = node.collapsed;

    let avail_w = ui.available_width();
    let (row_rect, _) = ui.allocate_exact_size(egui::vec2(avail_w, row_h), egui::Sense::hover());

    if is_active {
        ui.painter().rect_filled(
            row_rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgba_premultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 40),
        );
        ui.painter().rect_filled(
            egui::Rect::from_min_size(row_rect.min, egui::vec2(3.0, row_h)),
            egui::CornerRadius::same(2),
            t.accent,
        );
        *active_rect = Some(row_rect);
    } else if ui.rect_contains_pointer(row_rect) {
        ui.painter().rect_filled(
            row_rect,
            egui::CornerRadius::same(3),
            egui::Color32::from_rgba_premultiplied(
                t.button_hover_bg.r(),
                t.button_hover_bg.g(),
                t.button_hover_bg.b(),
                120,
            ),
        );
    }

    let mid_y = row_rect.center().y;

    if node.depth > 0 {
        let connector_x = row_rect.left() + indent - 2.0;
        ui.painter().line_segment(
            [
                egui::pos2(connector_x, mid_y),
                egui::pos2(connector_x + 12.0, mid_y),
            ],
            egui::Stroke::new(1.0, t.border),
        );
        let top_y = row_rect.top();
        ui.painter().line_segment(
            [
                egui::pos2(connector_x, top_y),
                egui::pos2(connector_x, mid_y),
            ],
            egui::Stroke::new(1.0, t.border),
        );
        if !is_last {
            ui.painter().line_segment(
                [
                    egui::pos2(connector_x, mid_y),
                    egui::pos2(connector_x, row_rect.bottom()),
                ],
                egui::Stroke::new(1.0, t.border),
            );
        }
    }

    let label_start_x = row_rect.left() + indent + 14.0;

    if has_children {
        let toggle_size = 16.0;
        let toggle_x = row_rect.left() + indent + 2.0;
        let toggle_rect = egui::Rect::from_center_size(
            egui::pos2(toggle_x, mid_y),
            egui::vec2(toggle_size, toggle_size),
        );
        let toggle_resp = ui.interact(
            toggle_rect,
            egui::Id::new(("tree_toggle", id)),
            egui::Sense::click(),
        );
        if toggle_resp.hovered() {
            ui.painter().rect_filled(
                toggle_rect,
                egui::CornerRadius::same(3),
                egui::Color32::from_rgba_premultiplied(
                    t.accent.r(),
                    t.accent.g(),
                    t.accent.b(),
                    50,
                ),
            );
        }
        let arrow = if is_collapsed { "▶" } else { "▼" };
        let arrow_col = if is_active {
            t.accent
        } else {
            t.tab_inactive_fg
        };
        ui.painter().text(
            toggle_rect.center(),
            egui::Align2::CENTER_CENTER,
            arrow,
            egui::FontId::proportional(9.0),
            arrow_col,
        );
        if toggle_resp.clicked() {
            *toggle_out = Some(id);
        }
    } else {
        let dot_x = row_rect.left() + indent + 8.0;
        ui.painter().circle_filled(
            egui::pos2(dot_x, mid_y),
            2.5,
            if is_active { t.accent } else { t.border },
        );
    }

    let badge = node_kind_badge(&node.label);
    let badge_bg = node_badge_color(&node.label, t);
    let mut text_x = label_start_x + 2.0;

    if !badge.is_empty() {
        let badge_w = (badge.len() as f32 * 6.5 + 8.0).max(36.0);
        let badge_rect =
            egui::Rect::from_min_size(egui::pos2(text_x, mid_y - 8.0), egui::vec2(badge_w, 16.0));
        ui.painter()
            .rect_filled(badge_rect, egui::CornerRadius::same(3), badge_bg);
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge,
            egui::FontId::monospace(9.0),
            t.editor_bg,
        );
        text_x += badge_w + 6.0;
    }

    let (text_col, _) = node_text_color(is_active, t, node.depth, has_children);
    let label_font = if is_active {
        egui::FontId::monospace(11.5)
    } else {
        egui::FontId::monospace(11.0)
    };

    let max_text_w = (row_rect.right() - text_x - 8.0).max(20.0);
    let full_label = &node.label;
    let display_label = if full_label.len() > 60 {
        format!("{}…", &full_label[..58])
    } else {
        full_label.clone()
    };

    ui.painter().text(
        egui::pos2(text_x, mid_y),
        egui::Align2::LEFT_CENTER,
        &display_label,
        label_font.clone(),
        text_col,
    );

    if is_collapsed && has_children {
        let count_str = format!("  +{} hidden", node.children.len());
        let main_w = ui.fonts_mut(|f| f.glyph_width(&label_font, 'x')) * display_label.len() as f32;
        ui.painter().text(
            egui::pos2(text_x + main_w + 4.0, mid_y),
            egui::Align2::LEFT_CENTER,
            &count_str,
            egui::FontId::proportional(9.5),
            t.tab_inactive_fg,
        );
    }

    let body_rect =
        egui::Rect::from_min_max(egui::pos2(label_start_x, row_rect.top()), row_rect.max);
    let body_resp = ui.interact(
        body_rect,
        egui::Id::new(("tree_body", id)),
        egui::Sense::click(),
    );
    if body_resp.clicked() && has_children && toggle_out.is_none() {
        *toggle_out = Some(id);
    }

    if !is_collapsed {
        let n = node.children.len();

        let children: Vec<usize> = node.children.clone();
        for (i, &child_id) in children.iter().enumerate() {
            draw_node(
                ui,
                table,
                child_id,
                active_node_id,
                t,
                i == n - 1,
                toggle_out,
                active_rect,
            );
        }
    }

    let _ = max_text_w;
}

fn node_text_color(
    is_active: bool,
    t: &Theme,
    depth: usize,
    has_children: bool,
) -> (egui::Color32, egui::Color32) {
    if is_active {
        return (t.accent, t.accent);
    }
    let text = if depth == 0 {
        t.tab_active_fg
    } else if !has_children {
        t.tab_inactive_fg
    } else {
        t.text_default
    };
    (text, t.border)
}

fn node_badge_color(label: &str, t: &Theme) -> egui::Color32 {
    match node_kind(label) {
        NodeKind::Decl => t.type_name,
        NodeKind::Control => t.keyword,
        NodeKind::Expr => t.fn_name,
        NodeKind::Literal => t.number,
        NodeKind::Type => t.type_name,
        NodeKind::Structural => t.struct_name,
        NodeKind::Other => t.tab_inactive_fg,
    }
}

fn node_kind_badge(label: &str) -> &'static str {
    match node_kind(label) {
        NodeKind::Decl => "DECL",
        NodeKind::Control => "CTRL",
        NodeKind::Expr => "EXPR",
        NodeKind::Literal => "LIT",
        NodeKind::Type => "TYPE",
        NodeKind::Structural => "STRUCT",
        NodeKind::Other => "",
    }
}

enum NodeKind {
    Decl,
    Control,
    Expr,
    Literal,
    Type,
    Structural,
    Other,
}

fn node_kind(label: &str) -> NodeKind {
    if label.starts_with("Decl") || label.starts_with("StructDecl") {
        NodeKind::Decl
    } else if label.starts_with("If")
        || label.starts_with("While")
        || label.starts_with("For")
        || label.starts_with("Return")
        || label.starts_with("Exit")
        || label.starts_with("Break")
        || label.starts_with("Continue")
    {
        NodeKind::Control
    } else if label.starts_with("Add")
        || label.starts_with("Mul")
        || label.starts_with("Cmp")
        || label.starts_with("Log")
        || label.starts_with("Bit")
        || label.starts_with("Shift")
        || label.starts_with("Unary")
        || label.starts_with("Cast")
        || label.starts_with("Assign")
        || label.starts_with("Chain")
        || label.starts_with("ExprStmt")
    {
        NodeKind::Expr
    } else if label.starts_with("Int")
        || label.starts_with("Float")
        || label.starts_with("Char")
        || label.starts_with("Bool")
        || label.starts_with("Str")
        || label.starts_with("Null")
        || label.starts_with("Ident")
        || label.starts_with("ArrayLit")
        || label.starts_with("StructLit")
    {
        NodeKind::Literal
    } else if label.starts_with(':') || label.starts_with("Type") {
        NodeKind::Type
    } else if label.starts_with("StructDef")
        || label.starts_with("FuncDef")
        || label.starts_with("Param")
        || label.starts_with("Program")
    {
        NodeKind::Structural
    } else {
        NodeKind::Other
    }
}

fn apply_theme_to_ctx(ctx: &egui::Context, t: &Theme) {
    let mut s = (*ctx.style()).clone();
    s.visuals.window_fill = t.panel_bg;
    s.visuals.panel_fill = t.panel_bg;
    s.visuals.extreme_bg_color = t.editor_bg;
    s.visuals.override_text_color = Some(t.tab_active_fg);
    s.visuals.window_stroke = egui::Stroke::new(1.0, t.border);
    s.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, t.tab_active_fg);
    s.visuals.widgets.inactive.bg_fill = t.button_bg;
    s.visuals.widgets.hovered.bg_fill = t.button_hover_bg;
    s.visuals.widgets.active.bg_fill = t.accent;
    ctx.set_style(s);
}
