use crate::contact_repository::ContactRepository;
use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::display_picture::DisplayPicture;
use crate::models::sign_in_return::SignInReturn;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::screens::contacts::category_collapsing_header::category_collapsing_header;
use crate::screens::contacts::status_selector::{Status, status_selector};
use crate::sqlite::Sqlite;
use crate::svg;
use eframe::egui;
use egui_taffy::taffy::prelude::length;
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{Client, MsnpList, MsnpStatus, PersonalMessage, SdkError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

pub enum Message {
    DisplayPictureResult(Result<DisplayPicture, Box<dyn std::error::Error + Sync + Send>>),
    StatusResult(Result<(), SdkError>),
    PersonalMessageResult(Result<(), SdkError>),
    BlockResult(Arc<String>, Result<(), SdkError>),
    UnblockResult(Arc<String>, Result<(), SdkError>),
    DeleteResult(Arc<String>, Result<(), SdkError>),
    AddContactResult(Box<Result<msnp11_sdk::Event, SdkError>>),
    CloseAddContact,
}

pub struct Contacts {
    user_email: Arc<String>,
    display_name: Arc<String>,
    personal_message: String,
    display_picture: Option<DisplayPicture>,
    selected_status: Status,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    show_personal_message_frame: bool,
    online_contacts: HashMap<Arc<String>, Contact>,
    offline_contacts: HashMap<Arc<String>, Contact>,
    contact_repository: ContactRepository,
    selected_contact: Option<Arc<String>>,
    client: Arc<Client>,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
    sqlite: Sqlite,
    add_contact_window: Option<crate::screens::add_contact::AddContact>,
    orphan_switchboards: HashMap<Arc<String>, SwitchboardAndParticipants>,
    handle: Handle,
}

impl Contacts {
    pub fn new(
        sign_in_return: SignInReturn,
        main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
        sqlite: Sqlite,
        handle: Handle,
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
            display_name: Arc::new(String::from("")),
            personal_message: sign_in_return.personal_message,
            display_picture: sign_in_return.display_picture,
            main_window_sender,
            selected_status,
            show_personal_message_frame: false,
            online_contacts: HashMap::new(),
            offline_contacts: HashMap::new(),
            contact_repository: ContactRepository::new(),
            selected_contact: None,
            client: sign_in_return.client,
            sender,
            receiver,
            sqlite,
            add_contact_window: None,
            orphan_switchboards: HashMap::new(),
            handle,
        }
    }

    pub fn handle_event(&mut self, message: crate::main_window::Message, ctx: &egui::Context) {
        match message {
            crate::main_window::Message::NotificationServerEvent(event) => match event {
                msnp11_sdk::Event::DisplayName(display_name) => {
                    self.display_name = Arc::new(display_name);
                }

                msnp11_sdk::Event::ContactInForwardList {
                    email,
                    display_name,
                    guid,
                    lists,
                    ..
                } => {
                    let email = Arc::new(email);
                    let contact = Contact {
                        email: email.clone(),
                        display_name: Arc::new(display_name),
                        guid: Arc::new(guid),
                        lists,
                        ..Default::default()
                    };

                    self.contact_repository
                        .add_contacts(std::slice::from_ref(&contact));

                    self.offline_contacts.insert(email, contact);
                }

                msnp11_sdk::Event::PresenceUpdate {
                    email,
                    display_name,
                    presence,
                } => {
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    let mut previous_status = None;
                    if let Some(contact) = &mut contact {
                        if let Some(msn_object) = &presence.msn_object
                            && msn_object.object_type == 3
                        {
                            contact.display_picture = if let Ok(picture) =
                                self.sqlite.select_display_picture_data(&msn_object.sha1d)
                            {
                                Some(DisplayPicture {
                                    data: picture,
                                    hash: Arc::new(msn_object.sha1d.clone()),
                                })
                            } else {
                                None
                            }
                        }

                        if let Some(presence) = &contact.status {
                            previous_status = Some(presence.status.clone());
                        }

                        contact.display_name = Arc::new(display_name);
                        contact.status = Some(Arc::new(presence));

                        self.contact_repository
                            .update_contacts(std::slice::from_ref(contact));
                    }

                    if let Some(contact) = contact.cloned()
                        && previous_status.is_none()
                    {
                        self.contact_repository
                            .update_contacts(std::slice::from_ref(&contact));

                        self.offline_contacts.remove(&email);
                        self.online_contacts.insert(contact.email.clone(), contact);
                    }
                }

                msnp11_sdk::Event::PersonalMessageUpdate {
                    email,
                    personal_message,
                } => {
                    let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    if let Some(contact) = contact {
                        contact.personal_message = Some(Arc::new(personal_message.psm));
                        self.contact_repository
                            .update_contacts(std::slice::from_ref(contact));
                    }
                }

                msnp11_sdk::Event::ContactOffline { email } => {
                    let mut contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                        Some(contact)
                    } else {
                        self.offline_contacts.get_mut(&email)
                    };

                    if let Some(contact) = &mut contact {
                        contact.status = None;
                        self.contact_repository
                            .update_contacts(std::slice::from_ref(contact));
                    }

                    if let Some(contact) = contact.cloned() {
                        self.contact_repository
                            .update_contacts(std::slice::from_ref(&contact));

                        self.online_contacts.remove(&email);
                        self.offline_contacts.insert(contact.email.clone(), contact);
                    }
                }

                msnp11_sdk::Event::SessionAnswered(switchboard) => {
                    if let Ok(session_id) = self.handle.block_on(switchboard.get_session_id()) {
                        let session_id = Arc::new(session_id);
                        self.orphan_switchboards.insert(
                            session_id.clone(),
                            SwitchboardAndParticipants {
                                switchboard: switchboard.clone(),
                                participants: Vec::new(),
                            },
                        );

                        let sender = self.main_window_sender.clone();
                        let ctx = ctx.clone();

                        self.handle.block_on(async {
                            switchboard.add_event_handler_closure(move |event| {
                                let sender = sender.clone();
                                let session_id = session_id.clone();
                                let ctx = ctx.clone();

                                async move {
                                    let _ =
                                        sender.send(crate::main_window::Message::SwitchboardEvent(
                                            session_id, event,
                                        ));

                                    ctx.request_repaint();
                                }
                            });
                        });
                    }
                }

                _ => (),
            },

            crate::main_window::Message::SwitchboardEvent(session_id, event) => {
                if let msnp11_sdk::Event::ParticipantInSwitchboard { email } = event
                    && let Some(mut switchboard) = self.orphan_switchboards.remove(&session_id)
                {
                    switchboard.participants.push(Arc::from(email));
                    let _ = self.main_window_sender.send(
                        crate::main_window::Message::OpenConversationWithSwitchboard {
                            user_email: self.user_email.clone(),
                            user_display_name: self.display_name.clone(),
                            user_display_picture: self.display_picture.clone(),
                            contact_repository: self.contact_repository.clone(),
                            session_id,
                            switchboard,
                        },
                    );

                    ctx.request_repaint();
                }
            }

            crate::main_window::Message::ContactDisplayPictureEvent { email, data } => {
                let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                    Some(contact)
                } else {
                    self.offline_contacts.get_mut(&email)
                };

                if let Some(contact) = contact {
                    if let Some(status) = &contact.status
                        && let Some(msn_object) = &status.msn_object
                    {
                        let _ = self.sqlite.insert_display_picture(&data, &msn_object.sha1d);
                        contact.display_picture = Some(DisplayPicture {
                            data,
                            hash: Arc::new(msn_object.sha1d.clone()),
                        });
                    }

                    self.contact_repository
                        .update_contacts(std::slice::from_ref(contact));
                }
            }

            crate::main_window::Message::ContactChatWindowFocused(email) => {
                let contact = if let Some(contact) = self.online_contacts.get_mut(&email) {
                    Some(contact)
                } else {
                    self.offline_contacts.get_mut(&email)
                };

                if let Some(contact) = contact {
                    contact.opening_conversation = false;
                }
            }

            _ => (),
        }
    }
}

impl eframe::App for Contacts {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::DisplayPictureResult(result) => {
                    if let Ok(picture) = result {
                        self.display_picture = Some(picture.clone());
                        let _ = self.main_window_sender.send(
                            crate::main_window::Message::UserDisplayPictureChanged(picture),
                        );
                    }
                }

                Message::StatusResult(result) => {
                    if let Err(error) = result {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    }
                }

                Message::PersonalMessageResult(result) => {
                    if let Err(error) = result {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    } else {
                        let _ = self
                            .sqlite
                            .update_personal_message(&self.user_email, &self.personal_message);
                    }
                }

                Message::BlockResult(contact, result) => {
                    if let Err(error) = result {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    } else {
                        let contact = if let Some(contact) = self.online_contacts.get_mut(&contact)
                        {
                            Some(contact)
                        } else {
                            self.offline_contacts.get_mut(&contact)
                        };

                        if let Some(contact) = contact {
                            contact.lists.push(MsnpList::BlockList);
                            contact.lists.retain(|list| list != &MsnpList::AllowList);
                            self.contact_repository
                                .update_contacts(std::slice::from_ref(contact));
                        }
                    }
                }

                Message::UnblockResult(contact, result) => {
                    if let Err(error) = result {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    } else {
                        let contact = if let Some(contact) = self.online_contacts.get_mut(&contact)
                        {
                            Some(contact)
                        } else {
                            self.offline_contacts.get_mut(&contact)
                        };

                        if let Some(contact) = contact {
                            contact.lists.retain(|list| list != &MsnpList::BlockList);
                            contact.lists.push(MsnpList::AllowList);
                            self.contact_repository
                                .update_contacts(std::slice::from_ref(contact));
                        }
                    }
                }

                Message::DeleteResult(contact, result) => {
                    if let Err(error) = result {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    } else {
                        self.online_contacts.remove(&contact);
                        self.offline_contacts.remove(&contact);
                        self.contact_repository.remove_contact(&contact);
                    }
                }

                Message::AddContactResult(result) => match *result {
                    Ok(event) => {
                        if let msnp11_sdk::Event::ContactInForwardList {
                            email,
                            display_name,
                            guid,
                            lists,
                            ..
                        } = event
                        {
                            let email = Arc::new(email);
                            let display_name = Arc::new(display_name);
                            let guid = Arc::new(guid);

                            let contact = Contact {
                                email: email.clone(),
                                display_name,
                                guid,
                                lists,
                                ..Default::default()
                            };

                            self.contact_repository
                                .add_contacts(std::slice::from_ref(&contact));

                            self.offline_contacts.insert(email, contact);
                        }
                    }

                    Err(error) => {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    }
                },

                Message::CloseAddContact => self.add_contact_window = None,
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
                                        egui::Image::from_bytes(
                                            format!("bytes://{}.png", picture.hash),
                                            picture.data,
                                        )
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
                                        &self.display_name,
                                        &mut self.selected_status,
                                        self.sender.clone(),
                                        self.main_window_sender.clone(),
                                        self.handle.clone(),
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
                                            self.handle.clone(),
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

                                if ui.link("Add a Contact").clicked() {
                                    if self.add_contact_window.is_some() {
                                        ctx.send_viewport_cmd_to(
                                            egui::ViewportId::from_hash_of("add-contact"),
                                            egui::ViewportCommand::Focus,
                                        );
                                    } else {
                                        self.add_contact_window =
                                            Some(crate::screens::add_contact::AddContact::new(
                                                self.client.clone(),
                                                self.sender.clone(),
                                                self.handle.clone(),
                                            ));
                                    }
                                }
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
                                &mut self.online_contacts,
                                self.main_window_sender.clone(),
                                self.sender.clone(),
                                self.handle.clone(),
                                self.user_email.clone(),
                                self.display_name.clone(),
                                self.display_picture.clone(),
                                self.selected_status,
                                self.contact_repository.clone(),
                                self.client.clone(),
                            );

                            category_collapsing_header(
                                ui,
                                "Offline",
                                &mut self.selected_contact,
                                &mut self.offline_contacts,
                                self.main_window_sender.clone(),
                                self.sender.clone(),
                                self.handle.clone(),
                                self.user_email.clone(),
                                self.display_name.clone(),
                                self.display_picture.clone(),
                                self.selected_status,
                                self.contact_repository.clone(),
                                self.client.clone(),
                            );
                        });
                    })
            });

        if let Some(add_contact) = &mut self.add_contact_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("add-contact"),
                egui::ViewportBuilder::default()
                    .with_title("Add contact")
                    .with_inner_size([390., 180.])
                    .with_maximize_button(false)
                    .with_minimize_button(false)
                    .with_resizable(false),
                |ctx, _| {
                    add_contact.add_contact(ctx);
                },
            );
        }

        if ctx.input(|input| input.viewport().close_requested()) {
            let _ = self
                .handle
                .block_on(async { self.client.disconnect().await });
        }
    }
}
