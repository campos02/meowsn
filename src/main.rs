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
use std::sync::Arc;

fn common_main() -> eframe::Result {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime");

    rt.spawn(async move { helpers::notify_new_version::notify_new_version().await });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("meowsn :3")
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
            cc.egui_ctx.set_pixels_per_point(1.02);

            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "noto_sans".to_string(),
                Arc::new(egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/NotoSans-Regular.ttf"
                ))),
            );

            fonts.font_data.insert(
                "noto_sans_bold".to_string(),
                Arc::new(egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/NotoSans-Bold.ttf"
                ))),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "noto_sans".to_string());

            fonts
                .families
                .entry(egui::FontFamily::Name("Bold".into()))
                .or_default()
                .insert(0, "noto_sans_bold".to_string());

            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(MainWindow::new(rt.handle().clone())))
        }),
    )
}

#[cfg(target_os = "macos")]
pub fn main() -> eframe::Result {
    let id = notify_rust::get_bundle_identifier_or_default("meowsn");
    notify_rust::set_application(&id).expect("Could not set application name");
    common_main()
}

#[cfg(not(target_os = "macos"))]
pub fn main() -> eframe::Result {
    common_main()
}
