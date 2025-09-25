#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]

mod main_window;
mod sign_in;
mod widgets;

use crate::main_window::MainWindow;
use eframe::egui;

pub fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450., 600.])
            .with_min_inner_size([450., 600.])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/meowsn.ico"))
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "meowsn",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MOCHA);

            cc.egui_ctx.set_pixels_per_point(1.2);
            cc.egui_ctx.style_mut(|style| {
                style.spacing.button_padding = egui::Vec2::splat(5.);
            });

            Ok(Box::<MainWindow>::default())
        }),
    )
}
