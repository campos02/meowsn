use crate::contact_repository::ContactRepository;
use crate::helpers::get_config::get_config;
use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::display_picture::DisplayPicture;
use crate::models::sign_in_return::SignInReturn;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::models::tab::Tab;
use crate::screens::contacts::category_collapsing_header::category_collapsing_header;
use crate::screens::contacts::status_selector::{Status, status_selector};
use crate::screens::conversation::conversation;
use crate::sqlite::Sqlite;
use crate::{models, settings, svg};
use eframe::egui;
use eframe::egui::OpenUrl;
use egui_taffy::taffy::prelude::{length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{Client, ContactError, MsnpList, MsnpStatus, PersonalMessage, SdkError};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::runtime::Handle;

pub enum Message {
    DisplayPictureResult(Result<DisplayPicture, Box<dyn std::error::Error + Sync + Send>>),
    StatusResult(MsnpStatus, Result<(), SdkError>),
    PersonalMessageResult(Result<(), SdkError>),
    BlockResult(Arc<String>, Result<(), ContactError>),
    UnblockResult(Arc<String>, Result<(), ContactError>),
    DeleteResult(Arc<String>, Result<(), ContactError>),
    AddContactResult(Box<Result<msnp11_sdk::Event, ContactError>>),
    GetConfigResult(Result<models::config::Config, Box<dyn std::error::Error + Sync + Send>>),
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
    online_contacts: BTreeMap<Arc<String>, Contact>,
    offline_contacts: BTreeMap<Arc<String>, Contact>,
    contact_repository: ContactRepository,
    selected_contact: Option<Arc<String>>,
    client: Arc<Client>,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
    sqlite: Sqlite,
    tabs: Vec<Tab>,
    msn_today_url: Option<String>,
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

        let settings = settings::get_settings().unwrap_or_default();
        run_future(
            handle.clone(),
            get_config(sign_in_return.client.clone(), settings.config_server),
            sender.clone(),
            Message::GetConfigResult,
        );

        Self {
            user_email: sign_in_return.email,
            display_name: Arc::new(String::from("")),
            personal_message: sign_in_return.personal_message,
            display_picture: sign_in_return.display_picture,
            main_window_sender,
            selected_status,
            show_personal_message_frame: false,
            online_contacts: BTreeMap::new(),
            offline_contacts: BTreeMap::new(),
            contact_repository: ContactRepository::new(),
            selected_contact: None,
            client: sign_in_return.client,
            sender,
            receiver,
            sqlite,
            tabs: Vec::new(),
            msn_today_url: None,
            add_contact_window: None,
            orphan_switchboards: HashMap::new(),
            handle,
        }
    }

    pub fn handle_event(
        &mut self,
        message: crate::main_window::Message,
        ctx: &egui::Context,
        conversations: &mut HashMap<egui::ViewportId, conversation::Conversation>,
    ) {
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

                msnp11_sdk::Event::InitialPresenceUpdate {
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
                        let settings = settings::get_settings().unwrap_or_default();
                        if settings.notify_sign_ins && self.selected_status != Status::Busy {
                            let _ = notify_rust::Notification::new()
                                .summary("New sign in")
                                .body(&format!("{} has just signed in", contact.display_name))
                                .show();
                        }

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

                msnp11_sdk::Event::ServerMaintenanceScheduled { time_remaining } => {
                    let _ = self
                        .main_window_sender
                        .send(crate::main_window::Message::OpenDialog(format!(
                            "The server will shut down for maintenance in {} minutes",
                            time_remaining
                        )));

                    ctx.request_repaint();
                }

                _ => (),
            },

            crate::main_window::Message::SwitchboardEvent(session_id, event) => match event {
                msnp11_sdk::Event::ParticipantInSwitchboard { email } => {
                    if let Some(switchboard) = self.orphan_switchboards.get_mut(&session_id) {
                        switchboard.participants.push(Arc::from(email));
                    }
                }

                msnp11_sdk::Event::ParticipantLeftSwitchboard { email } => {
                    if let Some(switchboard) = self.orphan_switchboards.get_mut(&session_id) {
                        switchboard
                            .participants
                            .retain(|participant| **participant != email);
                    }
                }

                msnp11_sdk::Event::TextMessage { .. } | msnp11_sdk::Event::Nudge { .. } => {
                    if let Some(switchboard) = self.orphan_switchboards.remove(&session_id) {
                        if let Some(conversation) =
                            conversations.values_mut().find(|conversation| {
                                conversation.get_participants().len() == 1
                                    && switchboard.participants.iter().all(|participant| {
                                        conversation.get_participants().contains_key(participant)
                                    })
                                    || conversation.get_participants().is_empty()
                                        && switchboard.participants.iter().all(|sb_participant| {
                                            conversation
                                                .get_last_participant()
                                                .as_ref()
                                                .is_some_and(|participant| {
                                                    *sb_participant == participant.email
                                                })
                                        })
                            })
                        {
                            conversation.add_switchboard(session_id.clone(), switchboard);
                        } else {
                            let user_status = match self.selected_status {
                                Status::Busy => MsnpStatus::Busy,
                                Status::Away => MsnpStatus::Away,
                                Status::AppearOffline => MsnpStatus::AppearOffline,
                                _ => MsnpStatus::Online,
                            };

                            let viewport_id = egui::ViewportId::from_hash_of(&session_id);
                            conversations.insert(
                                viewport_id,
                                conversation::Conversation::new_with_switchboard(
                                    self.user_email.clone(),
                                    self.display_name.clone(),
                                    self.display_picture.clone(),
                                    user_status,
                                    self.contact_repository.clone(),
                                    session_id.clone(),
                                    switchboard,
                                    self.main_window_sender.clone(),
                                    self.sqlite.clone(),
                                    self.handle.clone(),
                                    viewport_id,
                                ),
                            );

                            ctx.send_viewport_cmd_to(
                                viewport_id,
                                egui::ViewportCommand::Minimized(true),
                            );

                            ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                                egui::UserAttentionType::Informational,
                            ));
                        };

                        ctx.request_repaint();
                    }
                }

                _ => (),
            },

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

                Message::StatusResult(status, result) => {
                    if let Err(error) = result {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::OpenDialog(error.to_string()));

                        ctx.request_repaint();
                    } else {
                        let _ = self
                            .main_window_sender
                            .send(crate::main_window::Message::UserStatusChanged(status));
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

                Message::GetConfigResult(result) => {
                    if let Ok(config) = result {
                        self.tabs = config.tabs;
                        self.msn_today_url = Some(config.msn_today_url);
                    }
                }

                Message::CloseAddContact => self.add_contact_window = None,
            }
        }

        egui::TopBottomPanel::top("user_info")
            .frame(egui::Frame {
                inner_margin: egui::Margin {
                    top: 15,
                    bottom: 0,
                    left: 15,
                    right: 15,
                },
                fill: ctx.style().visuals.window_fill,
                ..Default::default()
            })
            .show_separator_line(false)
            .show(ctx, |ui| {
                tui(ui, ui.id().with("user_info"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        size: percent(1.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Row,
                            gap: length(10.),
                            ..Default::default()
                        })
                        .add(|tui| {
                            tui.style(taffy::Style {
                                size: length(62.),
                                ..Default::default()
                            })
                            .add_with_border(|tui| {
                                tui.ui(|ui| {
                                    ui.add(if let Some(picture) = self.display_picture.clone() {
                                        egui::Image::from_bytes(
                                            format!("bytes://{}", picture.hash),
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
                                });
                            });

                            tui.ui(|ui| {
                                ui.vertical(|ui| {
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

                                    let personal_message_edit = ui
                                        .add(
                                            egui::text_edit::TextEdit::singleline(
                                                &mut self.personal_message,
                                            )
                                            .hint_text("<Type a personal message>")
                                            .min_size(egui::vec2(180., 5.))
                                            .frame(self.show_personal_message_frame),
                                        )
                                        .on_hover_text("Type a personal message");

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

                                    ui.add_space(3.);
                                    if let Some(msn_today_url) = &self.msn_today_url {
                                        ui.hyperlink_to(" MSN Today", msn_today_url)
                                            .on_hover_text("MSN Today");
                                    }
                                });
                            });
                        });

                        tui.ui(|ui| {
                            ui.add_space(5.);
                            ui.separator();
                            ui.add_space(2.);

                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Image::new(svg::add_contact())
                                        .fit_to_exact_size(egui::Vec2::splat(20.))
                                        .alt_text("Add a contact"),
                                );

                                if ui
                                    .link("Add a Contact")
                                    .on_hover_text("Add a contact to your list")
                                    .clicked()
                                {
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
                            });
                        });
                    });
            });

        if !self.tabs.is_empty() {
            egui::SidePanel::left("tabs")
                .default_width(45.)
                .resizable(false)
                .show_separator_line(false)
                .frame(egui::Frame {
                    inner_margin: egui::Margin {
                        top: 10,
                        bottom: 15,
                        left: 15,
                        right: 0,
                    },
                    fill: ctx.style().visuals.window_fill,
                    ..Default::default()
                })
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            for tab in &self.tabs {
                                if ui
                                    .button(
                                        egui::Image::from_bytes(
                                            format!("bytes://{}", tab.msn_tab.name),
                                            tab.image.clone(),
                                        )
                                        .fit_to_exact_size(egui::Vec2::splat(25.))
                                        .alt_text(&tab.msn_tab.tooltip),
                                    )
                                    .on_hover_text(&tab.msn_tab.tooltip)
                                    .clicked()
                                {
                                    ui.ctx().open_url(OpenUrl {
                                        url: tab.msn_tab.content_url.clone(),
                                        new_tab: true,
                                    });
                                }

                                ui.add_space(3.);
                            }
                        });
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame {
                inner_margin: egui::Margin {
                    top: 0,
                    bottom: 15,
                    left: if !self.tabs.is_empty() { 0 } else { 15 },
                    right: 15,
                },
                fill: ctx.style().visuals.window_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                tui(ui, ui.id().with("contacts_screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        size: percent(1.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.style(taffy::Style {
                            size: percent(1.),
                            ..Default::default()
                        })
                        .ui(|ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink(false)
                                .show(ui, |ui| {
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
                        });
                    })
            });

        if let Some(add_contact) = &mut self.add_contact_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("add-contact"),
                egui::ViewportBuilder::default()
                    .with_title("Add contact")
                    .with_inner_size([400., 220.])
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
