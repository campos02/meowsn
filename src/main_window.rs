use crate::contact_repository::ContactRepository;
use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::display_picture::DisplayPicture;
use crate::models::sign_in_return::SignInReturn;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::screens::conversation::conversation;
use crate::screens::{contacts, personal_settings, sign_in};
use crate::sqlite::Sqlite;
use crate::visuals;
use eframe::egui;
use eframe::egui::CornerRadius;
use msnp11_sdk::{Client, MsnpStatus, SdkError, Switchboard};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

enum Screen {
    SignIn(sign_in::sign_in::SignIn),
    Contacts(Box<contacts::contacts::Contacts>),
}

pub enum Message {
    SignIn(SignInReturn),
    SignOut,
    OpenPersonalSettings(Option<String>, Option<Arc<Client>>),
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

    CreateSessionResult {
        user_email: Arc<String>,
        user_display_name: Arc<String>,
        user_display_picture: Option<DisplayPicture>,
        user_status: MsnpStatus,
        contact_repository: ContactRepository,
        result: Result<Arc<Switchboard>, SdkError>,
    },

    CloseConversation(egui::ViewportId),
    ContactChatWindowFocused(Arc<String>),
}

pub struct MainWindow {
    screen: Screen,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
    personal_settings_window: Option<Arc<Mutex<personal_settings::PersonalSettings>>>,
    dialog_window_text: Option<String>,
    conversations: HashMap<egui::ViewportId, conversation::Conversation>,
    handle: Handle,
    sqlite: Sqlite,
}

impl MainWindow {
    pub fn new(handle: Handle) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let sqlite = Sqlite::new().expect("Could not create database");

        Self {
            screen: Screen::SignIn(sign_in::sign_in::SignIn::new(
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let old = ctx.style().visuals.clone();
        if ctx.style().visuals.dark_mode {
            ctx.set_visuals(visuals::dark_mode(old));
        } else {
            ctx.set_visuals(visuals::light_mode(old));
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
                    self.screen = Screen::Contacts(Box::new(contacts::contacts::Contacts::new(
                        sign_in_return,
                        self.sender.clone(),
                        self.sqlite.clone(),
                        self.handle.clone(),
                    )));

                    let sender = self.sender.clone();
                    let main_ctx = ctx.clone();

                    self.handle.block_on(async {
                        client.add_event_handler_closure(move |event| {
                            let sender = sender.clone();
                            let ctx = main_ctx.clone();

                            async move {
                                let _ = sender.send(Message::NotificationServerEvent(event));
                                ctx.request_repaint();
                            }
                        });
                    });

                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                        egui::UserAttentionType::Informational,
                    ));
                }

                Message::SignOut => {
                    self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                        self.handle.clone(),
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
                            personal_settings::PersonalSettings::new(
                                display_name,
                                client,
                                self.sender.clone(),
                                self.handle.clone(),
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
                            self.handle.clone(),
                            self.sqlite.clone(),
                            self.sender.clone(),
                        ));

                        self.dialog_window_text = Some("Lost connection to the server".to_string());
                        ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                            egui::UserAttentionType::Informational,
                        ));
                    } else if let msnp11_sdk::Event::LoggedInAnotherDevice = event {
                        self.screen = Screen::SignIn(sign_in::sign_in::SignIn::new(
                            self.handle.clone(),
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
                        contacts.handle_event(
                            Message::NotificationServerEvent(event.clone()),
                            ctx,
                            &mut self.conversations,
                        );

                        for conversation in self.conversations.values_mut() {
                            conversation
                                .handle_event(Message::NotificationServerEvent(event.clone()), ctx);
                        }
                    }

                    ctx.request_repaint();
                }

                Message::SwitchboardEvent(session_id, event) => {
                    if let msnp11_sdk::Event::DisplayPicture { email, data } = event {
                        let data = data.into_boxed_slice();
                        let data = Arc::from(data);

                        let _ = self
                            .sender
                            .send(Message::ContactDisplayPictureEvent { email, data });
                    } else if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::SwitchboardEvent(session_id.clone(), event.clone()),
                            ctx,
                            &mut self.conversations,
                        );

                        for conversation in self.conversations.values_mut() {
                            conversation.handle_event(
                                Message::SwitchboardEvent(session_id.clone(), event.clone()),
                                ctx,
                            );
                        }
                    }

                    ctx.request_repaint();
                }

                Message::UserDisplayPictureChanged(picture) => {
                    for conversation in self.conversations.values_mut() {
                        conversation
                            .handle_event(Message::UserDisplayPictureChanged(picture.clone()), ctx);
                    }
                }

                Message::UserStatusChanged(status) => {
                    for conversation in self.conversations.values_mut() {
                        conversation.handle_event(Message::UserStatusChanged(status.clone()), ctx);
                    }
                }

                Message::ContactDisplayPictureEvent { email, data } => {
                    if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::ContactDisplayPictureEvent {
                                email: email.clone(),
                                data: data.clone(),
                            },
                            ctx,
                            &mut self.conversations,
                        );
                    }

                    for conversation in self.conversations.values_mut() {
                        conversation.handle_event(
                            Message::ContactDisplayPictureEvent {
                                email: email.clone(),
                                data: data.clone(),
                            },
                            ctx,
                        );
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
                        contacts.handle_event(
                            Message::NotificationServerEvent(msnp11_sdk::Event::DisplayName(
                                display_name,
                            )),
                            ctx,
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
                        ctx.send_viewport_cmd_to(*id, egui::ViewportCommand::Focus);
                        let _ = self
                            .sender
                            .send(Message::ContactChatWindowFocused(contact.email.clone()));
                    } else {
                        let contact_email = contact.email.clone();
                        let sender = self.sender.clone();

                        run_future(
                            self.handle.clone(),
                            async move { client.create_session(&contact_email).await },
                            sender,
                            move |result| Message::CreateSessionResult {
                                user_email: user_email.clone(),
                                user_display_name: user_display_name.clone(),
                                user_display_picture: user_display_picture.clone(),
                                user_status: user_status.clone(),
                                contact_repository: contact_repository.clone(),
                                result: result.map(Arc::from),
                            },
                        );
                    }
                }

                Message::CreateSessionResult {
                    user_email,
                    user_display_name,
                    user_display_picture,
                    user_status,
                    contact_repository,
                    result,
                } => {
                    if let Ok(switchboard) = result
                        && let Ok(session_id) = self.handle.block_on(switchboard.get_session_id())
                    {
                        let switchboard = SwitchboardAndParticipants {
                            switchboard,
                            participants: Vec::new(),
                        };

                        let session_id = Arc::new(session_id);
                        let viewport_id = egui::ViewportId::from_hash_of(&session_id);
                        let inner_switchboard = switchboard.switchboard.clone();

                        self.conversations.insert(
                            viewport_id,
                            conversation::Conversation::new(
                                user_email,
                                user_display_name,
                                user_display_picture,
                                user_status,
                                contact_repository,
                                session_id.clone(),
                                switchboard,
                                self.sender.clone(),
                                self.sqlite.clone(),
                                self.handle.clone(),
                                viewport_id,
                            ),
                        );

                        let sender = self.sender.clone();
                        let ctx = ctx.clone();

                        self.handle.block_on(async {
                            inner_switchboard.add_event_handler_closure(move |event| {
                                let sender = sender.clone();
                                let session_id = session_id.clone();
                                let ctx = ctx.clone();

                                async move {
                                    let _ =
                                        sender.send(Message::SwitchboardEvent(session_id, event));

                                    ctx.request_repaint();
                                }
                            });
                        });
                    }
                }

                Message::CloseConversation(id) => {
                    self.conversations.remove(&id);
                }

                Message::ContactChatWindowFocused(email) => {
                    if let Screen::Contacts(contacts) = &mut self.screen {
                        contacts.handle_event(
                            Message::ContactChatWindowFocused(email),
                            ctx,
                            &mut self.conversations,
                        );
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
                    .with_inner_size([300., 120.])
                    .with_maximize_button(false)
                    .with_minimize_button(false)
                    .with_resizable(false),
                |ctx, _| {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    egui::CentralPanel::default().show(ctx, |ui| {
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

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.dialog_window_text = None;
                    }
                },
            );
        }

        for (id, conversation) in &mut self.conversations {
            // Immediate might waste more CPU cycles but deferred is a real PITA in this type of application
            ctx.show_viewport_immediate(
                *id,
                egui::ViewportBuilder::default()
                    .with_title(conversation.get_title())
                    .with_inner_size([1000., 650.])
                    .with_min_inner_size([800., 500.]),
                |ctx, _| {
                    conversation.conversation(ctx);
                    if ctx.input(|input| input.viewport().close_requested()) {
                        conversation.leave_switchboards();
                        let _ = self.sender.send(Message::CloseConversation(*id));
                    }
                },
            );
        }

        if let Some(personal_settings_window) = self.personal_settings_window.clone() {
            let sender = self.sender.clone();
            let main_ctx = ctx.clone();

            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of("personal-settings"),
                egui::ViewportBuilder::default()
                    .with_title("Personal settings")
                    .with_inner_size([400., 400.])
                    .with_maximize_button(false)
                    .with_resizable(false),
                move |ctx, _| {
                    personal_settings_window
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .personal_settings(ctx);

                    if ctx.input(|input| input.viewport().close_requested()) {
                        let _ = sender.send(Message::ClosePersonalSettings);
                        main_ctx.request_repaint();
                    }
                },
            );
        }
    }
}
