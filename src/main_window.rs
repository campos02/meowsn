use crate::contact_repository::ContactRepository;
use crate::models::contact::Contact;
use crate::models::display_picture::DisplayPicture;
use crate::models::sign_in_return::SignInReturn;
use crate::screens::contacts::contacts;
use crate::screens::conversation::conversation;
use crate::screens::personal_settings;
use crate::screens::sign_in::sign_in;
use crate::sqlite::Sqlite;
use crate::visuals;
use eframe::egui;
use msnp11_sdk::{Client, MsnpStatus, SdkError};
use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use tokio::runtime::Handle;

enum Screen {
    SignIn(sign_in::SignIn),
    Contacts(Box<contacts::Contacts>),
}

pub enum Message {
    SignIn(SignInReturn),
    SignOut,
    OpenPersonalSettings(
        Option<String>,
        Option<Arc<Client>>,
        Option<ContactRepository>,
        Option<mpsc::Sender<contacts::Message>>,
        Option<bool>,
    ),

    ClosePersonalSettings,
    OpenDialog(String),
    NotificationServerEvent(msnp11_sdk::Event),
    SwitchboardEvent(Arc<String>, msnp11_sdk::Event),
    UserDisplayPictureChanged(DisplayPicture),
    UserStatusChanged(MsnpStatus),
    ContactDisplayPictureEvent {
        email: String,
        data: Arc<[u8]>,
    },

    DisplayNameChangeResult(String, Result<(), SdkError>),
    OpenConversation {
        user_email: Arc<String>,
        user_display_name: Arc<String>,
        user_display_picture: Option<DisplayPicture>,
        user_status: MsnpStatus,
        contact_repository: ContactRepository,
        contact: Contact,
        client: Arc<Client>,
    },

    CloseConversation(egui::ViewportId),
    ContactChatWindowFocused(Arc<String>),
}

pub struct MainWindow {
    screen: Screen,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    personal_settings_window: Option<personal_settings::PersonalSettings>,
    dialog_window_text: Option<String>,
    conversations: HashMap<egui::ViewportId, conversation::Conversation>,
    handle: Handle,
    sqlite: Sqlite,
}

impl MainWindow {
    pub fn new(handle: Handle) -> Self {
        let (sender, receiver) = mpsc::channel();
        let sqlite = Sqlite::new().expect("Could not create database");

        Self {
            screen: Screen::SignIn(sign_in::SignIn::new(
                handle.clone(),
                sqlite.clone(),
                sender.clone(),
            )),
            sender,
            receiver,
            personal_settings_window: None,
            dialog_window_text: None,
            conversations: HashMap::new(),
            handle,
            sqlite,
        }
    }
}

impl eframe::App for MainWindow {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let old = ui.visuals().clone();
        if ui.visuals().dark_mode {
            ui.set_visuals(visuals::dark_mode(old));
        } else {
            ui.set_visuals(visuals::light_mode(old));
        }

        if let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::SignIn(sign_in_return) => {
                    let client = sign_in_return.client.clone();
                    self.screen = Screen::Contacts(Box::new(contacts::Contacts::new(
                        sign_in_return,
                        self.sender.clone(),
                        self.sqlite.clone(),
                        self.handle.clone(),
                    )));

                    let sender = self.sender.clone();
                    let main_ui = ui.clone();

                    self.handle.block_on(async {
                        client.add_event_handler_closure(move |event| {
                            let sender = sender.clone();
                            let ui = main_ui.clone();

                            async move {
                                let _ = sender.send(Message::NotificationServerEvent(event));
                                ui.request_repaint();
                            }
                        });
                    });

                    ui.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                        egui::UserAttentionType::Informational,
                    ));
                }

                Message::SignOut => {
                    self.screen = Screen::SignIn(sign_in::SignIn::new(
                        self.handle.clone(),
                        self.sqlite.clone(),
                        self.sender.clone(),
                    ));
                }

                Message::OpenPersonalSettings(
                    display_name,
                    client,
                    contact_repository,
                    contacts_sender,
                    blp_bl,
                ) => {
                    if self.personal_settings_window.is_some() {
                        ui.send_viewport_cmd_to(
                            egui::ViewportId::from_hash_of("personal-settings"),
                            egui::ViewportCommand::Focus,
                        );
                    } else {
                        self.personal_settings_window =
                            Some(personal_settings::PersonalSettings::new(
                                display_name,
                                client,
                                contact_repository,
                                self.sender.clone(),
                                contacts_sender,
                                blp_bl,
                                self.handle.clone(),
                            ));
                    }
                }

                Message::ClosePersonalSettings => self.personal_settings_window = None,
                Message::OpenDialog(text) => {
                    self.dialog_window_text = Some(text);
                    ui.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                        egui::UserAttentionType::Informational,
                    ));
                }

                Message::NotificationServerEvent(event) => {
                    if let msnp11_sdk::Event::Disconnected = event {
                        self.screen = Screen::SignIn(sign_in::SignIn::new(
                            self.handle.clone(),
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some("Lost connection to the server".to_string());
                        ui.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
                    } else if let msnp11_sdk::Event::LoggedInAnotherDevice = event {
                        self.screen = Screen::SignIn(sign_in::SignIn::new(
                            self.handle.clone(),
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some(
                            "Disconnected as you have signed in on another computer".to_string(),
                        );

                        ui.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::NotificationServerEvent(event.clone()),
                            ui,
                            &mut self.conversations,
                        );

                        for conversation in self.conversations.values_mut() {
                            conversation
                                .handle_event(Message::NotificationServerEvent(event.clone()), ui);
                        }
                    }

                    ui.request_repaint();
                }

                Message::SwitchboardEvent(session_id, event) => {
                    if let msnp11_sdk::Event::DisplayPicture { email, data } = event {
                        let data = Arc::from(data);
                        let _ = self
                            .sender
                            .send(Message::ContactDisplayPictureEvent { email, data });
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::SwitchboardEvent(session_id.clone(), event.clone()),
                            ui,
                            &mut self.conversations,
                        );

                        for conversation in self.conversations.values_mut() {
                            conversation.handle_event(
                                Message::SwitchboardEvent(session_id.clone(), event.clone()),
                                ui,
                            );
                        }
                    }

                    ui.request_repaint();
                }

                Message::UserDisplayPictureChanged(picture) => {
                    for conversation in self.conversations.values_mut() {
                        conversation
                            .handle_event(Message::UserDisplayPictureChanged(picture.clone()), ui);
                    }
                }

                Message::UserStatusChanged(status) => {
                    for conversation in self.conversations.values_mut() {
                        conversation.handle_event(Message::UserStatusChanged(status.clone()), ui);
                    }
                }

                Message::ContactDisplayPictureEvent { email, data } => {
                    if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::ContactDisplayPictureEvent {
                                email: email.clone(),
                                data: data.clone(),
                            },
                            ui,
                            &mut self.conversations,
                        );
                    }

                    for conversation in self.conversations.values_mut() {
                        conversation.handle_event(
                            Message::ContactDisplayPictureEvent {
                                email: email.clone(),
                                data: data.clone(),
                            },
                            ui,
                        );
                    }
                }

                Message::DisplayNameChangeResult(display_name, result) => {
                    if let Err(error) = result {
                        self.dialog_window_text =
                            Some(format!("Error setting display name: {error}"));

                        ui.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::NotificationServerEvent(msnp11_sdk::Event::DisplayName(
                                display_name,
                            )),
                            ui,
                            &mut self.conversations,
                        );
                    }
                }

                Message::OpenConversation {
                    user_email,
                    user_display_name,
                    user_display_picture,
                    user_status,
                    contact_repository,
                    contact,
                    client,
                } => {
                    if let Some((id, _)) = self.conversations.iter().find(|(_, conversation)| {
                        conversation.get_participants().contains_key(&contact.email)
                            || conversation.get_participants().is_empty()
                                && conversation
                                    .get_last_participant()
                                    .as_ref()
                                    .is_some_and(|participant| participant.email == contact.email)
                    }) {
                        ui.send_viewport_cmd_to(*id, egui::ViewportCommand::Focus);
                        let _ = self
                            .sender
                            .send(Message::ContactChatWindowFocused(contact.email.clone()));
                    } else {
                        let viewport_id = egui::ViewportId::from_hash_of(contact.guid.clone());
                        self.conversations.insert(
                            viewport_id,
                            conversation::Conversation::new(
                                user_email,
                                user_display_name,
                                user_display_picture,
                                user_status,
                                contact,
                                contact_repository,
                                client,
                                self.sender.clone(),
                                self.sqlite.clone(),
                                self.handle.clone(),
                                viewport_id,
                            ),
                        );
                    }
                }

                Message::CloseConversation(id) => {
                    self.conversations.remove(&id);
                }

                Message::ContactChatWindowFocused(email) => {
                    if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::ContactChatWindowFocused(email),
                            ui,
                            &mut self.conversations,
                        );
                    }
                }
            }
        }

        match &mut self.screen {
            Screen::SignIn(sign_in) => sign_in.ui(ui, frame),
            Screen::Contacts(contacts) => contacts.ui(ui, frame),
        }

        if self.dialog_window_text.is_some() {
            ui.show_viewport_immediate(
                egui::ViewportId::from_hash_of("dialog"),
                egui::ViewportBuilder::default()
                    .with_title("meowsn")
                    .with_inner_size([300., 120.])
                    .with_maximize_button(false)
                    .with_minimize_button(false)
                    .with_resizable(false),
                |ui, _| {
                    ui.send_viewport_cmd(egui::ViewportCommand::Focus);
                    egui::CentralPanel::default().show_inside(ui, |ui| {
                        ui.add_space(18.);
                        ui.vertical_centered(|ui| {
                            ui.add(
                                egui::Label::new(
                                    self.dialog_window_text.as_ref().unwrap_or(&"".to_string()),
                                )
                                .halign(egui::Align::Center),
                            );

                            ui.add_space(5.);
                            if ui.button("Ok").clicked() {
                                self.dialog_window_text = None;
                            }
                        });
                    });

                    if ui.input(|i| i.viewport().close_requested()) {
                        self.dialog_window_text = None;
                    }
                },
            );
        }

        for (id, conversation) in &mut self.conversations {
            // Immediate might waste more CPU cycles but deferred is a real PITA in this type of application
            ui.show_viewport_immediate(
                *id,
                egui::ViewportBuilder::default()
                    .with_title(conversation.get_title())
                    .with_inner_size([1000., 650.])
                    .with_min_inner_size([800., 500.]),
                |ui, _| {
                    conversation.conversation(ui);
                    if ui.input(|input| input.viewport().close_requested()) {
                        conversation.leave_switchboards();
                        let _ = self.sender.send(Message::CloseConversation(*id));
                    }
                },
            );
        }

        if let Some(personal_settings_window) = &mut self.personal_settings_window {
            let sender = self.sender.clone();
            let main_ui = ui.clone();

            ui.show_viewport_immediate(
                egui::ViewportId::from_hash_of("personal-settings"),
                egui::ViewportBuilder::default()
                    .with_title("Personal settings")
                    .with_inner_size([600., 500.])
                    .with_maximize_button(false)
                    .with_resizable(false),
                move |ui, _| {
                    personal_settings_window.personal_settings(ui);
                    if ui.input(|input| input.viewport().close_requested()) {
                        let _ = sender.send(Message::ClosePersonalSettings);
                        main_ui.request_repaint();
                    }
                },
            );
        }
    }
}
