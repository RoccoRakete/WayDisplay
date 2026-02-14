#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::env;
use std::fs;
use std::process;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let wlr = "wlr-randr";

    if check_wlr_randr(wlr) {
        println!("wlr-randr found!");
    } else {
        process::exit(0);
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        // .with_inner_size([400.0, 300.0])
        // .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "WayDisplay",
        native_options,
        Box::new(|cc| Ok(Box::new(way_display::WayDisplay::new(cc)))),
    )
}

fn check_wlr_randr(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(':') {
            let p_str = format!("{p}/{program}");
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}
