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
#[cfg(not(target_os = "linux"))]
use std::cell::RefCell;
#[cfg(not(target_os = "linux"))]
use std::rc::Rc;
use std::sync::Arc;

fn common_main() -> eframe::Result {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build tokio runtime");

    rt.spawn(async move { helpers::notify_new_version::notify_new_version().await });
    let icon = include_bytes!("../assets/meowsn.ico");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("meowsn")
            .with_inner_size([350., 600.])
            .with_min_inner_size([350., 500.])
            .with_icon(eframe::icon_data::from_png_bytes(icon).unwrap_or_default()),
        centered: true,
        ..Default::default()
    };

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory_with_format(icon, image::ImageFormat::Ico)
            .expect("Failed to load tray icon")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to load tray icon");

    let open_item = tray_icon::menu::MenuItem::new("Open meowsn", true, None);
    let exit_item = tray_icon::menu::MenuItem::new("Exit", true, None);

    #[cfg(not(target_os = "linux"))]
    let tray_menu = tray_icon::menu::Menu::with_items(&[&open_item, &exit_item])
        .expect("Failed to create tray icon menu");

    let open_item = open_item.into_id();
    let exit_item = exit_item.into_id();

    let thread_open_item = open_item.clone();
    let thread_exit_item = exit_item.clone();

    #[cfg(target_os = "linux")]
    std::thread::spawn(|| {
        if gtk::init().is_ok() {
            let tray_menu = tray_icon::menu::Menu::with_items(&[
                &tray_icon::menu::MenuItem::with_id(thread_open_item, "Open meowsn", true, None),
                &tray_icon::menu::MenuItem::with_id(thread_exit_item, "Exit", true, None),
            ])
            .expect("Failed to create tray icon menu");

            if let Ok(_tray_icon) = tray_icon::TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_icon(icon)
                .build()
            {
                gtk::main();
            }
        }
    });

    #[cfg(not(target_os = "linux"))]
    // The icon only works inside a Rc<RefCell<>>
    let tray_icon = Rc::new(RefCell::new(None));

    #[cfg(not(target_os = "linux"))]
    // It also needs to be cloned
    let tray_icon = tray_icon.clone();

    eframe::run_native(
        "meowsn",
        options,
        Box::new(move |cc| {
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

            #[cfg(not(target_os = "linux"))]
            if let Ok(icon) = tray_icon::TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip("meowsn")
                .with_icon(icon)
                .build()
            {
                tray_icon.borrow_mut().replace(icon);
            }

            cc.egui_ctx.set_fonts(fonts);
            let ctx = cc.egui_ctx.clone();

            tray_icon::menu::MenuEvent::set_event_handler(Some(move |event| {
                let tray_icon::menu::MenuEvent { id } = event;
                if id == open_item {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                } else if id == exit_item {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }));

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
