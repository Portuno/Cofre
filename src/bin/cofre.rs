// Cofre Vault — Native Desktop UI (egui/eframe)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod app;
use app::CofreApp;

fn main() -> eframe::Result<()> {
    dotenv::dotenv().ok();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Cofre Vault")
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Cofre Vault",
        options,
        Box::new(|cc| Box::new(CofreApp::new(cc))),
    )
}
