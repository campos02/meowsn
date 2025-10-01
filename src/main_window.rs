use crate::models::sign_in_return::SignInReturn;
use crate::screens;
use crate::screens::{contacts, conversation, sign_in};
use crate::sqlite::Sqlite;
use eframe::egui;
use eframe::egui::CornerRadius;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{Client, SdkError};
use std::sync::{Arc, Mutex};

enum Screen {
    SignIn(sign_in::sign_in::SignIn),
    Contacts(contacts::contacts::Contacts),
    Conversation(conversation::Conversation),
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
        if ctx.style().visuals.dark_mode {
            catppuccin_egui::set_theme(&ctx, catppuccin_egui::MOCHA);
        } else {
            catppuccin_egui::set_theme(&ctx, catppuccin_egui::LATTE);
        }

        ctx.style_mut(|style| {
            style.spacing.button_padding = egui::Vec2::splat(5.);
            style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
            style.visuals.indent_has_left_vline = false;
            style.spacing.combo_height = 250.;
        });

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
                Message::OpenDialog(text) => {
                    self.dialog_window_text = Some(text);
                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                        egui::UserAttentionType::Informational,
                    ));
                }

                Message::NotificationServerEvent(event) => {
                    if let msnp11_sdk::Event::Disconnected = event {
                        self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some("Lost connection to the server".to_string());
                        ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
                    } else if let msnp11_sdk::Event::LoggedInAnotherDevice = event {
                        self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some(
                            "Disconnected as you have signed in on another computer".to_string(),
                        );

                        ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(event);
                        ctx.request_repaint();
                    }
                }

                Message::DisplayNameChangeResult(display_name, result) => {
                    if let Err(error) = result {
                        self.dialog_window_text =
                            Some(format!("Error setting display name: {error}"));

                        ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
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
            Screen::Conversation(conversation) => conversation.update(ctx, frame),
        }

        if self.dialog_window_text.is_some() {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("dialog"),
                egui::ViewportBuilder::default()
                    .with_title("meowsn")
                    .with_inner_size([300., 100.])
                    .with_maximize_button(false)
                    .with_minimize_button(false)
                    .with_resizable(false),
                |ctx, _| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        tui(ui, ui.id().with("dialog"))
                            .reserve_available_space()
                            .style(taffy::Style {
                                flex_direction: taffy::FlexDirection::Column,
                                align_items: Some(taffy::AlignItems::Center),
                                justify_content: Some(taffy::JustifyContent::Center),
                                size: taffy::Size {
                                    width: percent(1.),
                                    height: percent(1.),
                                },
                                padding: length(15.),
                                gap: percent(0.15),
                                ..Default::default()
                            })
                            .show(|tui| {
                                tui.style(taffy::Style {
                                    size: taffy::Size {
                                        width: percent(1.),
                                        height: auto(),
                                    },
                                    ..Default::default()
                                })
                                .ui_add(
                                    egui::Label::new(
                                        self.dialog_window_text.as_ref().unwrap_or(&"".to_string()),
                                    )
                                    .halign(egui::Align::Center),
                                );

                                tui.style(taffy::Style {
                                    size: taffy::Size {
                                        width: length(30.),
                                        height: auto(),
                                    },
                                    ..Default::default()
                                })
                                .ui(|ui| {
                                    if ui.button("Ok").clicked() {
                                        self.dialog_window_text = None;
                                    }
                                });
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
