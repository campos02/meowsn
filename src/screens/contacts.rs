use eframe::egui;
use eframe::egui::ComboBox;
use std::fmt::Display;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Status {
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

pub struct Contacts {
    selected_status: Status,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
}

impl Contacts {
    pub fn new(main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>) -> Self {
        Self {
            main_window_sender,
            selected_status: Status::Online,
        }
    }
}

impl eframe::App for Contacts {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let old_status = self.selected_status;
                ComboBox::from_label("")
                    .selected_text(self.selected_status.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::Online,
                            Status::Online.to_string(),
                        );

                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::Busy,
                            Status::Busy.to_string(),
                        );

                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::Away,
                            Status::Away.to_string(),
                        );

                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::AppearOffline,
                            Status::AppearOffline.to_string(),
                        );

                        ui.separator();
                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::ChangeDisplayPicture,
                            Status::ChangeDisplayPicture.to_string(),
                        );

                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::PersonalSettings,
                            Status::PersonalSettings.to_string(),
                        );

                        ui.separator();
                        ui.selectable_value(
                            &mut self.selected_status,
                            Status::SignOut,
                            Status::SignOut.to_string(),
                        );
                    });

                match self.selected_status {
                    Status::PersonalSettings | Status::ChangeDisplayPicture => {
                        self.selected_status = old_status
                    }

                    Status::SignOut => {
                        self.selected_status = old_status;
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::SignOut);
                    }

                    _ => (),
                }
            });
    }
}
