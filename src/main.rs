#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod contact_repository;
mod helpers;
mod main_window;
mod models;
mod screens;
mod settings;
mod sqlite;
mod svg;
mod visuals;
mod widgets;

use crate::main_window::MainWindow;
use eframe::egui;

fn main() -> eframe::Result {
    #[cfg(target_os = "macos")]
    let id = notify_rust::get_bundle_identifier_or_default("meowsn");
    #[cfg(target_os = "macos")]
    notify_rust::set_application(&id).expect("Could not set application name");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime");

    rt.spawn(async move { helpers::notify_new_version::notify_new_version().await });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("meowsn")
            .with_inner_size([350., 600.])
            .with_min_inner_size([350., 500.])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/meowsn.ico"))
                    .expect("Failed to load icon"),
            ),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "meowsn",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_fonts(visuals::load_fonts());
            cc.egui_ctx.global_style_mut(|style| {
                style.spacing.button_padding = egui::Vec2::splat(5.);
                style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
                style.visuals.indent_has_left_vline = false;
                style.spacing.combo_height = 250.;
            });

            Ok(Box::new(MainWindow::new(rt.handle().clone())))
        }),
    )
}
