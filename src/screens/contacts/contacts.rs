use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::sign_in_return::SignInReturn;
use crate::screens::contacts::category_collapsing_header::category_collapsing_header;
use crate::screens::contacts::status_selector::{Status, status_selector};
use crate::sqlite::Sqlite;
use crate::svg;
use eframe::egui;
use egui_taffy::taffy::prelude::length;
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{Client, MsnpStatus, PersonalMessage, SdkError};
use std::collections::HashMap;
use std::sync::Arc;

pub enum Message {
    DisplayPictureResult(Result<Arc<[u8]>, Box<dyn std::error::Error + Sync + Send>>),
    StatusResult(Result<(), SdkError>),
    PersonalMessageResult(Result<(), SdkError>),
}

pub struct Contacts {
    user_email: Arc<String>,
    display_name: Arc<String>,
    personal_message: String,
    display_picture: Option<Arc<[u8]>>,
    selected_status: Status,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    show_personal_message_frame: bool,
    online_contacts: HashMap<Arc<String>, Contact>,
    offline_contacts: HashMap<Arc<String>, Contact>,
    selected_contact: Option<Arc<String>>,
    client: Arc<Client>,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
    sqlite: Sqlite,
}

impl Contacts {
    pub fn new(
        sign_in_return: SignInReturn,
        main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
        sqlite: Sqlite,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let selected_status = match sign_in_return.status {
            MsnpStatus::Busy => Status::Busy,
            MsnpStatus::Away => Status::Away,
            MsnpStatus::AppearOffline => Status::AppearOffline,
            _ => Status::Online,
        };

        Self {
            user_email: sign_in_return.email,
            display_name: Arc::new(String::from("Testing 2")),
            personal_message: sign_in_return.personal_message,
            display_picture: sign_in_return.display_picture,
            main_window_sender,
            selected_status,
            show_personal_message_frame: false,
            online_contacts: HashMap::new(),
            offline_contacts: HashMap::new(),
            selected_contact: None,
            client: sign_in_return.client,
            sender,
            receiver,
            sqlite,
        }
    }
}

impl eframe::App for Contacts {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::DisplayPictureResult(result) => {
                    if let Ok(picture) = result {
                        self.display_picture = Some(picture);
                    }
                }

                Message::StatusResult(result) | Message::PersonalMessageResult(result) => {
                    if let Err(_) = result {
                        todo!()
                    }
                }
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                tui(ui, ui.id().with("contacts-screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        padding: length(15.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Row,
                            gap: length(10.),
                            ..Default::default()
                        })
                        .add(|tui| {
                            tui.add_with_border(|tui| {
                                tui.ui(|ui| {
                                    ui.add(if let Some(picture) = self.display_picture.clone() {
                                        egui::Image::from_bytes("bytes://picture.png", picture)
                                            .fit_to_exact_size(egui::Vec2::splat(60.))
                                            .corner_radius(
                                                ui.visuals().widgets.noninteractive.corner_radius,
                                            )
                                            .alt_text("User display picture")
                                    } else {
                                        egui::Image::new(svg::default_display_picture())
                                            .fit_to_exact_size(egui::Vec2::splat(60.))
                                            .alt_text("Default display picture")
                                    })
                                })
                            });

                            tui.ui(|ui| {
                                ui.vertical(|ui| {
                                    ui.add_space(5.);
                                    status_selector(
                                        ui,
                                        self.user_email.clone(),
                                        self.display_name.as_str(),
                                        &mut self.selected_status,
                                        self.sender.clone(),
                                        self.main_window_sender.clone(),
                                        self.sqlite.clone(),
                                        self.client.clone(),
                                    );

                                    let personal_message_edit = ui.add(
                                        egui::text_edit::TextEdit::singleline(
                                            &mut self.personal_message,
                                        )
                                        .hint_text("<Type a personal message>")
                                        .min_size(egui::vec2(180., 5.))
                                        .frame(self.show_personal_message_frame),
                                    );

                                    if personal_message_edit.lost_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                    {
                                        let client = self.client.clone();
                                        let sender = self.sender.clone();

                                        let personal_message = PersonalMessage {
                                            psm: self.personal_message.clone(),
                                            current_media: "".to_string(),
                                        };

                                        run_future(
                                            async move {
                                                client.set_personal_message(&personal_message).await
                                            },
                                            sender,
                                            Message::PersonalMessageResult,
                                        );
                                    }

                                    self.show_personal_message_frame = personal_message_edit
                                        .hovered()
                                        || personal_message_edit.has_focus();
                                });
                            })
                        });

                        tui.ui(|ui| ui.add_space(8.));
                        tui.ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Image::new(svg::add_contact())
                                        .fit_to_exact_size(egui::Vec2::splat(20.))
                                        .alt_text("Add a contact"),
                                );
                                ui.link("Add a Contact")
                            })
                        });

                        tui.style(taffy::Style {
                            padding: length(10.),
                            ..Default::default()
                        })
                        .ui(|ui| {
                            category_collapsing_header(
                                ui,
                                "Online",
                                &mut self.selected_contact,
                                &self.online_contacts,
                            );

                            category_collapsing_header(
                                ui,
                                "Offline",
                                &mut self.selected_contact,
                                &self.offline_contacts,
                            );
                        });
                    })
            });
    }
}
