use crate::models::Monitor;

/// Main application state
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct WayDisplay {
    pub monitors: Vec<Monitor>,
    pub selected_idx: Option<usize>,
    pub selected_mode_idx: Option<usize>,
    pub adaptive_sync: bool,
    pub monitor_enabled: bool,
    pub cmd_output: Option<String>,

    #[serde(skip)]
    pub error_msg: Option<String>,
}

impl Default for WayDisplay {
    fn default() -> Self {
        Self {
            monitors: Vec::new(),
            selected_idx: None,
            selected_mode_idx: None,
            error_msg: None,
            adaptive_sync: false,
            monitor_enabled: true,
            cmd_output: None,
        }
    }
}

impl WayDisplay {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: Self = if let Some(storage) = _cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app.refresh_monitors();
        app
    }
}
impl eframe::App for WayDisplay {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_style(ctx);
        self.render_top_panel(ctx);
        self.render_bottom_panel(ctx);
        self.render_side_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_main_content(ui);
        });
    }
}
