use crate::widgets::window_fill_combo_box::WindowFillComboBox;
use eframe::egui::Ui;
use std::fmt::Display;

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
    display_name: &str,
    selected_status: &mut Status,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
) {
    let old_status = *selected_status;
    WindowFillComboBox::from_label("")
        .selected_text(format!(
            "{display_name}   ({})",
            selected_status.to_string()
        ))
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
        Status::PersonalSettings | Status::ChangeDisplayPicture => *selected_status = old_status,
        Status::SignOut => {
            *selected_status = old_status;
            let _ = main_window_sender.send(crate::main_window::Message::SignOut);
        }

        _ => (),
    }
}
