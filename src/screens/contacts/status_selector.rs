use crate::helpers::pick_display_picture::pick_display_picture;
use crate::helpers::run_future::run_future;
use crate::sqlite::Sqlite;
use crate::widgets::custom_fill_combo_box::CustomFillComboBox;
use eframe::egui::Ui;
use msnp11_sdk::{Client, MsnpStatus};
use rfd::AsyncFileDialog;
use std::fmt::Display;
use std::sync::Arc;
use tokio::runtime::Handle;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Status {
    Online,
    Busy,
    Away,
    AppearOffline,
    ChangeDisplayPicture,
    PersonalSettings,
    SignOut,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Online => "Online",
            Self::Busy => "Busy",
            Self::Away => "Away",
            Self::AppearOffline => "Appear Offline",
            Self::ChangeDisplayPicture => "Change Display Picture...",
            Self::PersonalSettings => "Personal Settings...",
            Self::SignOut => "Sign Out",
        })
    }
}

pub fn status_selector(
    ui: &mut Ui,
    email: Arc<String>,
    display_name: &str,
    selected_status: &mut Status,
    contacts_sender: std::sync::mpsc::Sender<crate::screens::contacts::contacts::Message>,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    handle: Handle,
    sqlite: Sqlite,
    client: Arc<Client>,
) {
    let old_status = *selected_status;
    CustomFillComboBox::from_label("")
        .selected_text(format!(
            "{display_name}   ({})",
            selected_status.to_string()
        ))
        .fill_color(ui.style().visuals.window_fill)
        .show_ui(ui, |ui| {
            ui.selectable_value(selected_status, Status::Online, Status::Online.to_string());
            ui.selectable_value(selected_status, Status::Busy, Status::Busy.to_string());
            ui.selectable_value(selected_status, Status::Away, Status::Away.to_string());
            ui.selectable_value(
                selected_status,
                Status::AppearOffline,
                Status::AppearOffline.to_string(),
            );

            ui.separator();
            ui.selectable_value(
                selected_status,
                Status::ChangeDisplayPicture,
                Status::ChangeDisplayPicture.to_string(),
            );

            ui.selectable_value(
                selected_status,
                Status::PersonalSettings,
                Status::PersonalSettings.to_string(),
            );

            ui.separator();
            ui.selectable_value(
                selected_status,
                Status::SignOut,
                Status::SignOut.to_string(),
            );
        });

    match selected_status {
        Status::ChangeDisplayPicture => {
            *selected_status = old_status;
            let picture = AsyncFileDialog::new()
                .add_filter("Images", &["png", "jpeg", "jpg"])
                .set_directory("/")
                .set_title("Select a display picture")
                .pick_file();

            run_future(
                handle.clone(),
                pick_display_picture(picture, email, client, sqlite),
                contacts_sender,
                crate::screens::contacts::contacts::Message::DisplayPictureResult,
            );
        }

        Status::PersonalSettings => {
            *selected_status = old_status;
            let _ = main_window_sender.send(crate::main_window::Message::OpenPersonalSettings(
                Some(display_name.to_string()),
                Some(client.clone()),
            ));
        }

        Status::SignOut => {
            *selected_status = old_status;
            let _ = handle.block_on(async { client.disconnect().await });
            let _ = main_window_sender.send(crate::main_window::Message::SignOut);
        }

        _ => {
            let status = match selected_status {
                Status::Busy => MsnpStatus::Busy,
                Status::Away => MsnpStatus::Away,
                Status::AppearOffline => MsnpStatus::AppearOffline,
                _ => MsnpStatus::Online,
            };

            let client = client.clone();
            run_future(
                handle.clone(),
                async move { client.set_presence(status).await },
                contacts_sender,
                crate::screens::contacts::contacts::Message::StatusResult,
            )
        }
    }
}
