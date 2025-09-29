use crate::models::sign_in_return::SignInReturn;
use crate::screens;
use crate::screens::{contacts, sign_in};
use crate::sqlite::Sqlite;
use eframe::egui;
use std::sync::{Arc, Mutex};

enum Screen {
    SignIn(sign_in::sign_in::SignIn),
    Contacts(contacts::contacts::Contacts),
}

pub enum Message {
    SignIn(SignInReturn),
    SignOut,
    OpenPersonalSettings(Option<String>),
    ClosePersonalSettings,
    OpenDialog(String),
}

pub struct MainWindow {
    screen: Screen,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
    personal_settings_window: Option<Arc<Mutex<screens::personal_settings::PersonalSettings>>>,
    dialog_window_text: Option<String>,
    sqlite: Sqlite,
}

impl Default for MainWindow {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let sqlite = Sqlite::new().expect("Could not create database");

        Self {
            screen: Screen::SignIn(sign_in::sign_in::SignIn::new(
                sqlite.clone(),
                sender.clone(),
            )),
            sender,
            receiver,
            personal_settings_window: None,
            dialog_window_text: None,
            sqlite,
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::SignIn(sign_in_return) => {
                    self.screen = Screen::Contacts(contacts::contacts::Contacts::new(
                        sign_in_return,
                        self.sender.clone(),
                        self.sqlite.clone(),
                    ))
                }

                Message::SignOut => {
                    self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                        self.sqlite.clone(),
                        self.sender.clone(),
                    ));
                }

                Message::OpenPersonalSettings(display_name) => {
                    if self.personal_settings_window.is_some() {
                        ctx.send_viewport_cmd_to(
                            egui::ViewportId::from_hash_of("personal-settings"),
                            egui::ViewportCommand::Focus,
                        );
                    } else {
                        self.personal_settings_window = Some(Arc::new(Mutex::new(
                            screens::personal_settings::PersonalSettings::new(display_name),
                        )));
                    }
                }

                Message::ClosePersonalSettings => self.personal_settings_window = None,
                Message::OpenDialog(text) => self.dialog_window_text = Some(text),
            }
        }

        match &mut self.screen {
            Screen::SignIn(sign_in) => sign_in.update(ctx, frame),
            Screen::Contacts(contacts) => contacts.update(ctx, frame),
        }

        if self.dialog_window_text.is_some() {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("dialog"),
                egui::ViewportBuilder::default()
                    .with_title("meowsn")
                    .with_inner_size([300.0, 100.0])
                    .with_min_inner_size([300.0, 100.0])
                    .with_maximize_button(false)
                    .with_minimize_button(false),
                |ctx, _| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame {
                                fill: ctx.style().visuals.window_fill,
                                ..Default::default()
                            }
                            .inner_margin(10.),
                        )
                        .show(ctx, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    self.dialog_window_text.as_ref().unwrap_or(&"".to_string()),
                                );

                                ui.add_space(10.);
                                if ui.button("Ok").clicked() {
                                    self.dialog_window_text = None;
                                }
                            })
                        });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.dialog_window_text = None;
                    }
                },
            );
        }

        if let Some(personal_settings_window) = self.personal_settings_window.clone() {
            let sender = self.sender.clone();

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of("personal-settings"),
                egui::ViewportBuilder::default()
                    .with_title("Personal settings")
                    .with_inner_size([400., 350.])
                    .with_min_inner_size([400., 350.])
                    .with_active(true)
                    .with_maximize_button(false),
                move |ctx, _| {
                    personal_settings_window
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .personal_settings(ctx);

                    if ctx.input(|input| input.viewport().close_requested()) {
                        let _ = sender.send(Message::ClosePersonalSettings);
                    }
                },
            );
        }
    }
}
