use crate::app::WayDisplay;
use egui::{Align, Context, Layout, RichText, Ui, vec2};

impl WayDisplay {
    pub fn apply_style(&self, ctx: &Context) {
        ctx.set_pixels_per_point(1.25);

        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.button_padding = vec2(10.0, 5.0);
        style.spacing.item_spacing = vec2(7.0, 7.0);
        style.url_in_tooltip = true;
        ctx.set_style(style);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
    }

    pub fn render_top_panel(&self, ctx: &Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(2.0);
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    ui.menu_button("Theme", |ui| {
                        egui::widgets::global_theme_preference_buttons(ui);
                    });
                });
            });
        });
    }

    pub fn render_bottom_panel(&self, ctx: &Context) {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(7.0);
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                ui.hyperlink_to(
                    format!(
                        "{}  WayDisplay on GitHub",
                        egui_phosphor::regular::GITHUB_LOGO
                    ),
                    "https://github.com/RoccoRakete/WayDisplay",
                );
            });
            ui.add_space(1.0);
        });
    }

    pub fn render_side_panel(&mut self, ctx: &Context) {
        egui::SidePanel::left("display_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.heading("Available Displays:");
                    if ui
                        .button(egui_phosphor::regular::ARROWS_CLOCKWISE)
                        .clicked()
                    {
                        self.refresh_monitors();
                    }
                });
                ui.separator();

                ui.add_space(5.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, monitor) in self.monitors.iter().enumerate() {
                        let is_selected = self.selected_idx == Some(i);
                        let icon_text = format!(
                            "{}  {}  ({})",
                            egui_phosphor::regular::MONITOR,
                            monitor.name,
                            monitor.model
                        );
                        if ui
                            .selectable_label(is_selected, RichText::new(icon_text).size(16.0))
                            .clicked()
                        {
                            self.selected_idx = Some(i);
                        }
                    }
                });
            });
    }

    pub fn render_main_content(&mut self, ui: &mut Ui) {
        if let Some(idx) = self.selected_idx {
            if let Some(monitor) = self.monitors.get(idx) {
                let monitor_name = monitor.name.clone();
                let monitor_model = monitor.model.clone();
                let modes = monitor.modes.clone();

                ui.heading(format!("Display: {monitor_name} ({monitor_model})"));
                ui.add_space(8.0);
                ui.separator();

                let available_width = ui.available_width();
                let available_height = ui.available_height();
                let spacing = ui.spacing().item_spacing.x;
                let col_width = (available_width - 2.0 * spacing - 10.0) / 3.0;

                ui.horizontal(|ui| {
                    // Left Column: Modes
                    ui.allocate_ui_with_layout(
                        vec2(col_width, available_height),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.label("Available Modes:");
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .id_salt("mode_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (m_idx, mode) in modes.iter().enumerate() {
                                        let mode_label = format!(
                                            "{}x{} @ {:.2} Hz",
                                            mode.width, mode.height, mode.refresh
                                        );
                                        if ui
                                            .selectable_label(
                                                self.selected_mode_idx == Some(m_idx),
                                                mode_label,
                                            )
                                            .clicked()
                                        {
                                            self.selected_mode_idx = Some(m_idx);
                                        }
                                    }
                                });
                        },
                    );

                    ui.separator();

                    // Center Column: Settings
                    ui.allocate_ui_with_layout(
                        vec2(col_width, available_height),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.label("Settings:");
                            ui.checkbox(&mut self.monitor_enabled, "Enable Monitor");
                            ui.checkbox(&mut self.adaptive_sync, "Adaptive Sync");
                            ui.separator();
                            ui.label("Monitor Alignment:");

                            // Alignment Canvas
                            let (rect, _) = ui.allocate_at_least(
                                vec2(ui.available_width(), 200.0),
                                egui::Sense::hover(),
                            );
                            ui.painter()
                                .rect_filled(rect, 3.0, ui.visuals().extreme_bg_color);
                            let center = rect.center();

                            for i in 0..self.monitors.len() {
                                let preview_size = vec2(80.0, 45.0);
                                let monitor_rect = egui::Rect::from_center_size(
                                    center + self.monitors[i].visual_pos.to_vec2(),
                                    preview_size,
                                );

                                let id = ui.make_persistent_id(format!("mon_drag_{}", i));
                                let res = ui.interact(monitor_rect, id, egui::Sense::drag());

                                if res.dragged() {
                                    // 1. Move the monitor
                                    self.monitors[i].visual_pos += res.drag_delta();

                                    // 2. Define boundaries (The black box is 'rect')
                                    let half_w = rect.width() / 2.0;
                                    let half_h = rect.height() / 2.0;

                                    // Margin (half of our 80x45 preview box)
                                    let margin_x = 40.0;
                                    let margin_y = 22.5;

                                    // 3. Clamp visual_pos relative to the center of the box
                                    self.monitors[i].visual_pos.x = self.monitors[i]
                                        .visual_pos
                                        .x
                                        .clamp(-half_w + margin_x, half_w - margin_x);
                                    self.monitors[i].visual_pos.y = self.monitors[i]
                                        .visual_pos
                                        .y
                                        .clamp(-half_h + margin_y, half_h - margin_y);

                                    let snap_dist = 10.0;
                                    for j in 0..self.monitors.len() {
                                        if i == j {
                                            continue;
                                        }
                                        let other_pos = self.monitors[j].visual_pos;
                                        if (self.monitors[i].visual_pos.x - (other_pos.x - 80.0))
                                            .abs()
                                            < snap_dist
                                        {
                                            self.monitors[i].visual_pos.x = other_pos.x - 80.0;
                                        }
                                    }
                                }
                                let color = if self.selected_idx == Some(i) {
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.visuals().widgets.inactive.bg_fill
                                };
                                ui.painter().rect_filled(monitor_rect, 2.0, color);
                                ui.painter().rect_stroke(
                                    monitor_rect,
                                    2.0,
                                    ui.visuals().widgets.active.fg_stroke,
                                    egui::StrokeKind::Middle,
                                );
                            }
                        },
                    );
                    ui.separator();

                    // Right Column: Info & Apply
                    ui.allocate_ui_with_layout(
                        vec2(col_width, available_height),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.label("Information:");
                            ui.separator();

                            if let Some(selected_mode) =
                                self.selected_mode_idx.and_then(|idx| modes.get(idx))
                            {
                                ui.label(format!(
                                    "Resolution: {}x{}",
                                    selected_mode.width, selected_mode.height
                                ));
                                ui.label(format!("Refresh: {:.2} Hz", selected_mode.refresh));
                                ui.add_space(10.0);

                                if ui.button(RichText::new("Apply").size(14.0)).clicked() {
                                    self.apply_settings();
                                }
                            }

                            if let Some(cmd_str) = &self.cmd_output {
                                ui.add_space(5.0);
                                ui.separator();
                                ui.label("wlr-randr Command:");
                                ui.add(egui::Label::new(RichText::new(cmd_str).code()));

                                if ui.button("Copy Command").clicked() {
                                    ui.ctx().copy_text(cmd_str.clone());
                                }
                            }
                        },
                    );
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Select a display on the left to see available modes.");
            });
        }
    }
}
