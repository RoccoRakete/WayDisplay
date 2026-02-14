use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Mode {
    pub width: i32,
    pub height: i32,
    pub refresh: f32,
    pub preferred: bool,
    pub current: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Monitor {
    pub name: String,
    pub model: String,
    pub enabled: bool,
    pub modes: Vec<Mode>,

    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,

    #[serde(skip)]
    pub visual_pos: egui::Pos2,
}
