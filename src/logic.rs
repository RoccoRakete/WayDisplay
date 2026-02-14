use crate::app::WayDisplay;
use crate::models::Monitor;
use std::process::Command;

impl WayDisplay {
    /// Fetches the current monitor configuration using wlr-randr
    pub fn refresh_monitors(&mut self) {
        let output = Command::new("wlr-randr").arg("--json").output();

        match output {
            Ok(out) => {
                let json_string = String::from_utf8_lossy(&out.stdout);
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
        let (Some(m_idx), Some(mode_idx)) = (self.selected_idx, self.selected_mode_idx) else {
            return;
        };

        let Some(mode) = self.monitors.get(m_idx).and_then(|m| m.modes.get(mode_idx)) else {
            return;
        };

        let monitor = &self.monitors[m_idx];
        let refresh_str = format!("{}", mode.refresh);
        let mode_str = format!("{}x{}@{}", mode.width, mode.height, refresh_str);

        let ui_preview_width = 80.0;
        let scale = mode.width as f32 / ui_preview_width;

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

        let physical_x = ((monitor.visual_pos.x - min_x) * scale).round() as i32;
        let physical_y = ((monitor.visual_pos.y - min_y) * scale).round() as i32;

        let mut cmd = Command::new("wlr-randr");
        cmd.arg("--output").arg(&monitor.name);

        if self.monitor_enabled {
            cmd.arg("--custom-mode").arg(mode_str);

            cmd.arg("--pos")
                .arg(format!("{},{}", physical_x, physical_y));

            let sync_status = if self.adaptive_sync {
                "enabled"
            } else {
                "disabled"
            };
            cmd.arg("--adaptive-sync").arg(sync_status);
        } else {
            cmd.arg("--off");
        }

        let wlr_output = format!(
            "{} {}",
            cmd.get_program().to_string_lossy(),
            cmd.get_args()
                .map(|arg| arg.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );
        self.cmd_output = Some(wlr_output);

        // Execute the command
        match cmd.output() {
            Ok(output) if output.status.success() => {
                log::info!(
                    "Settings applied to {}: Pos {},{}",
                    monitor.name,
                    physical_x,
                    physical_y
                );
            }
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                self.error_msg = Some(format!("wlr-randr error: {err}"));
            }
            Err(e) => {
                self.error_msg = Some(format!("Failed to run wlr-randr: {e}"));
            }
        }
    }
}
