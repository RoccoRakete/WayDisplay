use std::process::Command;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct WayDisplay {
    monitors: Vec<Monitor>,
    selected_idx: Option<usize>,
    selected_mode_idx: Option<usize>,
    adaptive_sync: bool,
    monitor_enabled: bool,

    #[serde(skip)]
    error_msg: Option<String>,
}
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh: f32,
    pub preferred: bool,
    pub current: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Monitor {
    pub name: String,
    pub model: String,
    pub enabled: bool,
    pub modes: Vec<Mode>, // Hier landen alle Modi aus dem JSON
}
impl Default for WayDisplay {
    fn default() -> Self {
        Self {
            // Example stuff:
            monitors: Vec::new(),
            selected_idx: None,
            selected_mode_idx: None,
            error_msg: None,
            adaptive_sync: false,
            monitor_enabled: true,
        }
    }
}

impl WayDisplay {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        Default::default()
    }
    pub fn refresh_monitors(&mut self) {
        // 1. Befehl ausführen und Output abfangen
        let output = Command::new("wlr-randr").arg("--json").output();

        match output {
            Ok(out) => {
                let json_string = String::from_utf8_lossy(&out.stdout);

                // 3. JSON parsen
                match serde_json::from_str::<Vec<Monitor>>(&json_string) {
                    Ok(data) => {
                        self.monitors = data;
                        self.error_msg = None;
                    }
                    Err(e) => {
                        self.error_msg = Some(format!("Parse error: {e}"));
                    }
                }
            }
            Err(e) => {
                self.error_msg = Some(format!("Failed to execute wlr-randr: {e}"));
            }
        }
    }
    pub fn apply_settings(&mut self) {
        if let (Some(m_idx), Some(mode_idx)) = (self.selected_idx, self.selected_mode_idx) {
            if let Some(monitor) = self.monitors.get(m_idx) {
                if let Some(mode) = monitor.modes.get(mode_idx) {
                    let refresh_str = format!("{}", mode.refresh);
                    let mode_str = format!("{}x{}@{}", mode.width, mode.height, refresh_str);

                    let mut cmd = Command::new("wlr-randr");
                    cmd.arg("--output").arg(&monitor.name);

                    if self.monitor_enabled {
                        cmd.arg("--custom-mode").arg(mode_str);

                        if self.adaptive_sync {
                            cmd.arg("--adaptive-sync").arg("enabled");
                        } else {
                            cmd.arg("--adaptive-sync").arg("disabled");
                        }
                    } else {
                        cmd.arg("--off");
                    }

                    match cmd.output() {
                        Ok(output) => {
                            if output.status.success() {
                                log::info!("Settings applied to {}", monitor.name);
                            } else {
                                let err = String::from_utf8_lossy(&output.stderr);
                                self.error_msg = Some(format!("wlr-randr error: {err}"));
                            }
                        }
                        Err(e) => {
                            self.error_msg = Some(format!("Failed to run wlr-randr: {e}"));
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for WayDisplay {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.25);
        let mut style: egui::Style = (*ctx.style()).clone();
        let mut fonts = egui::FontDefinitions::default();
        style.spacing.button_padding = egui::vec2(5.0, 5.0);
        style.spacing.item_spacing = egui::vec2(7.0, 7.0);
        style.url_in_tooltip = true;
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        ctx.set_style(style);
        ctx.set_fonts(fonts);

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
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.add_space(7.0);

            // Use with_layout to arrange children horizontally
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                let icon_text_github = format!(
                    "{}  {}",
                    egui_phosphor::regular::GITHUB_LOGO,
                    "WayDisplay on GitHub"
                );
                let icon_text_issue =
                    format!("{}  {}", egui_phosphor::regular::BUG, "Report a Bug!");

                ui.hyperlink_to(
                    icon_text_github,
                    "https://github.com/RoccoRakete/WayDisplay/issues",
                );

                ui.add_space(15.0);

                ui.hyperlink_to(icon_text_issue, "https://github.com/RoccoRakete/WayDisplay");
            });

            ui.add_space(1.0);
        });
        egui::SidePanel::left("display_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.heading("Available Displays:");
                ui.add_space(5.0);
                ui.separator();

                self.refresh_monitors();

                ui.add_space(5.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, monitor) in self.monitors.iter().enumerate() {
                        let is_selected = self.selected_idx == Some(i);

                        let icon_text = format!(
                            "{}  {}  {}",
                            egui_phosphor::regular::MONITOR,
                            monitor.name,
                            monitor.model,
                        );
                        let monitor_text = egui::RichText::new(icon_text).size(16.0);
                        if ui.selectable_label(is_selected, monitor_text).clicked() {
                            self.selected_idx = Some(i);
                        }
                    }
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(idx) = self.selected_idx {
                if let Some(monitor) = self.monitors.get(idx) {
                    let monitor_name = monitor.name.clone();
                    let monitor_model = monitor.model.clone();
                    let modes = monitor.modes.clone(); // Copy of the modes list

                    ui.heading(format!("Display: {monitor_name} ({monitor_model})"));
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Calculate width / grab the maximum height before horizontal layout
                    let available_width = ui.available_width();
                    let available_height = ui.available_height();
                    let spacing = ui.spacing().item_spacing.x;
                    let col_width = (available_width - 2.0 * spacing - 10.0) / 3.0;

                    ui.horizontal(|ui| {
                        // Left Column
                        ui.allocate_ui_with_layout(
                            egui::vec2(col_width, available_height), // Use captured height
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label("Available Modes:");
                                ui.separator();

                                egui::ScrollArea::vertical()
                                    .id_salt("mode_scroll")
                                    // Prevent the ScrollArea from shrinking to its content
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for (m_idx, mode) in modes.iter().enumerate() {
                                            let mode_label = format!(
                                                "{} x {} @ {:.2} Hz",
                                                mode.width, mode.height, mode.refresh
                                            );
                                            let is_selected = self.selected_mode_idx == Some(m_idx);

                                            if ui
                                                .selectable_label(is_selected, mode_label)
                                                .clicked()
                                            {
                                                self.selected_mode_idx = Some(m_idx);
                                            }
                                        }
                                    });
                            },
                        );

                        ui.separator();

                        // Center Column
                        ui.allocate_ui_with_layout(
                            egui::vec2(col_width, available_height), // Use captured height
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label("Settings:");
                                ui.separator();
                                ui.checkbox(&mut self.monitor_enabled, "Enable Monitor");
                                ui.checkbox(&mut self.adaptive_sync, "Adaptive Sync");
                            },
                        );

                        ui.separator();

                        // Right Column
                        ui.allocate_ui_with_layout(
                            egui::vec2(col_width, available_height), // Use captured height
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.label("Information:");
                                ui.separator();

                                if let Some(m_idx) = self.selected_mode_idx {
                                    if let Some(selected_mode) = modes.get(m_idx) {
                                        ui.label(format!(
                                            "Resolution: {}x{}",
                                            selected_mode.width, selected_mode.height
                                        ));
                                        ui.label(format!(
                                            "Refresh: {:.2} Hz",
                                            selected_mode.refresh
                                        ));

                                        ui.add_space(10.0);
                                        if ui
                                            .button(egui::RichText::new("Apply").size(16.0))
                                            .clicked()
                                        {
                                            self.apply_settings();
                                        }
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
        });
    }
}
