use crate::widgets::custom_combo_box::CustomComboBox;
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
    CustomComboBox::from_label("Status:")
        .selected_text(selected_status.to_string())
        .fill_color(ui.visuals().window_fill)
        .label_on_right(false)
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
        })
        .response
        .on_hover_text("Select the status you will have after signing in");

    if *selected_status == Status::PersonalSettings {
        *selected_status = old_status;
        let _ = main_window_sender.send(crate::main_window::Message::OpenPersonalSettings(
            None, None, None, None, None,
        ));
    }
}
