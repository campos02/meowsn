use crate::models::sign_in_return::SignInReturn;
use crate::screens;
use crate::screens::{contacts, sign_in};
use crate::sqlite::Sqlite;
use eframe::egui;
use msnp11_sdk::{Client, SdkError};
use std::sync::{Arc, Mutex};

enum Screen {
    SignIn(sign_in::sign_in::SignIn),
    Contacts(contacts::contacts::Contacts),
}

pub enum Message {
    SignIn(SignInReturn),
    SignOut,
    OpenPersonalSettings(Option<String>, Option<Arc<Client>>),
    ClosePersonalSettings,
    OpenDialog(String),
    NotificationServerEvent(msnp11_sdk::Event),
    DisplayNameChangeResult(String, Result<(), SdkError>),
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
                    let client = sign_in_return.client.clone();
                    self.screen = Screen::Contacts(contacts::contacts::Contacts::new(
                        sign_in_return,
                        self.sender.clone(),
                        self.sqlite.clone(),
                    ));

                    let sender = self.sender.clone();
                    client.add_event_handler_closure(move |event| {
                        let sender = sender.clone();
                        async move {
                            let _ = sender.send(Message::NotificationServerEvent(event));
                        }
                    });

                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                        egui::UserAttentionType::Informational,
                    ));
                }

                Message::SignOut => {
                    self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                        self.sqlite.clone(),
                        self.sender.clone(),
                    ));
                }

                Message::OpenPersonalSettings(display_name, client) => {
                    if self.personal_settings_window.is_some() {
                        ctx.send_viewport_cmd_to(
                            egui::ViewportId::from_hash_of("personal-settings"),
                            egui::ViewportCommand::Focus,
                        );
                    } else {
                        self.personal_settings_window = Some(Arc::new(Mutex::new(
                            screens::personal_settings::PersonalSettings::new(
                                display_name,
                                client,
                                self.sender.clone(),
                            ),
                        )));
                    }
                }

                Message::ClosePersonalSettings => self.personal_settings_window = None,
                Message::OpenDialog(text) => self.dialog_window_text = Some(text),

                Message::NotificationServerEvent(event) => {
                    if let msnp11_sdk::Event::Disconnected = event {
                        self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some("Lost connection to the server".to_string());
                    } else if let msnp11_sdk::Event::LoggedInAnotherDevice = event {
                        self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some(
                            "Disconnected as you have signed in on another computer".to_string(),
                        );
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(event);
                        ctx.request_repaint();
                    }
                }

                Message::DisplayNameChangeResult(display_name, result) => {
                    if let Err(error) = result {
                        self.dialog_window_text =
                            Some(format!("Error setting display name: {error}"));
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(msnp11_sdk::Event::DisplayName(display_name));
                        ctx.request_repaint();
                    }
                }
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
                    .with_active(true)
                    .with_maximize_button(false)
                    .with_minimize_button(false)
                    .with_resizable(false),
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
                    .with_active(true)
                    .with_maximize_button(false)
                    .with_resizable(false),
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
