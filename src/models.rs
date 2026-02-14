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
    pub modes: Vec<Mode>,

    #[serde(skip)]
    pub visual_pos: egui::Pos2,
}
