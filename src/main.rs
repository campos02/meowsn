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
mod widgets;

use crate::main_window::MainWindow;
use eframe::egui;
use eframe::egui::CornerRadius;

fn common_main() -> eframe::Result {
    tokio::spawn(async move { helpers::notify_new_version::notify_new_version().await });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([350., 600.])
            .with_min_inner_size([350., 500.])
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

            cc.egui_ctx.set_pixels_per_point(1.1);
            cc.egui_ctx.style_mut(|style| {
                style.spacing.button_padding = egui::Vec2::splat(5.);
                style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
                style.visuals.indent_has_left_vline = false;
                style.spacing.combo_height = 250.;
            });

            Ok(Box::<MainWindow>::default())
        }),
    )
}

#[cfg(target_os = "macos")]
#[tokio::main]
pub async fn main() -> eframe::Result {
    let id = notify_rust::get_bundle_identifier_or_default("meowsn");
    notify_rust::set_application(&id).expect("Could not set application name");
    common_main()
}

#[cfg(not(target_os = "macos"))]
#[tokio::main]
pub async fn main() -> eframe::Result {
    common_main()
}
