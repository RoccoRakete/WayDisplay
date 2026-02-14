use crate::app::WayDisplay;
use crate::models::Monitor;
use std::process::Command;

impl WayDisplay {
    pub fn refresh_monitors(&mut self) {
        let output = Command::new("wlr-randr").arg("--json").output();

        match output {
            Ok(out) => {
                let json_string = String::from_utf8_lossy(&out.stdout);
                match serde_json::from_str::<Vec<Monitor>>(&json_string) {
                    Ok(mut data) => {
                        let scale = 80.0 / 1920.0;
                        for m in &mut data {
                            m.visual_pos = egui::pos2(m.x as f32 * scale, m.y as f32 * scale);
                        }
                        self.monitors = data;
                        self.error_msg = None;
                    }
                    Err(e) => {
                        self.error_msg = Some(format!("JSON Parse Error: {e}"));
                    }
                }
            }
            Err(e) => {
                self.error_msg = Some(format!("System Error: {e}"));
            }
        }
    }

    pub fn apply_settings(&mut self) {
        let (Some(m_idx), Some(mode_idx)) = (self.selected_idx, self.selected_mode_idx) else {
            return;
        };
        let Some(mode) = self.monitors.get(m_idx).and_then(|m| m.modes.get(mode_idx)) else {
            return;
        };

        let monitor = &self.monitors[m_idx];
        let scale = mode.width as f32 / 80.0;

        // Find normalization point (top-left)
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        for m in &self.monitors {
            if m.visual_pos.x < min_x {
                min_x = m.visual_pos.x;
            }
            if m.visual_pos.y < min_y {
                min_y = m.visual_pos.y;
            }
        }

        let px = ((monitor.visual_pos.x - min_x) * scale).round() as i32;
        let py = ((monitor.visual_pos.y - min_y) * scale).round() as i32;

        let mut cmd = Command::new("wlr-randr");
        cmd.arg("--output").arg(&monitor.name);

        if self.monitor_enabled {
            cmd.arg("--custom-mode")
                .arg(format!("{}x{}@{}", mode.width, mode.height, mode.refresh));
            cmd.arg("--pos").arg(format!("{},{}", px, py));
            cmd.arg("--adaptive-sync").arg(if self.adaptive_sync {
                "enabled"
            } else {
                "disabled"
            });
        } else {
            cmd.arg("--off");
        }

        let cmd_full = format!(
            "{} {}",
            cmd.get_program().to_string_lossy(),
            cmd.get_args()
                .map(|a| a.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );
        self.cmd_output = Some(cmd_full);

        match cmd.output() {
            Ok(output) if output.status.success() => {
                self.error_msg = None;
            }
            Ok(output) => {
                self.error_msg = Some(String::from_utf8_lossy(&output.stderr).to_string());
            }
            Err(e) => {
                self.error_msg = Some(e.to_string());
            }
        }
    }
}
