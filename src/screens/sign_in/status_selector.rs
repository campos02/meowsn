use crate::widgets::left_label_combo_box::LeftLabelComboBox;
use eframe::egui::Ui;
use std::fmt::Display;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Status {
    Online,
    Busy,
    Away,
    AppearOffline,
    PersonalSettings,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Online => "Online",
            Self::Busy => "Busy",
            Self::Away => "Away",
            Self::AppearOffline => "Appear Offline",
            Self::PersonalSettings => "Personal Settings...",
        })
    }
}

pub fn status_selector(
    ui: &mut Ui,
    selected_status: &mut Status,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
) {
    let old_status = *selected_status;
    LeftLabelComboBox::from_label("Status:")
        .selected_text(selected_status.to_string())
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
                Status::PersonalSettings,
                Status::PersonalSettings.to_string(),
            );
        });

    if *selected_status == Status::PersonalSettings {
        *selected_status = old_status;
        let _ = main_window_sender.send(crate::main_window::Message::OpenPersonalSettings(
            None, None,
        ));
    }
}
