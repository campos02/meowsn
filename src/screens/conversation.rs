use crate::contact_repository::ContactRepository;
use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::display_picture::DisplayPicture;
use crate::models::message;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::sqlite::Sqlite;
use crate::svg;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, FontSelection, TextFormat};
use egui_taffy::taffy::prelude::{auto, fr, length, line, percent, repeat, span};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{SdkError, Switchboard};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

const INITIAL_HISTORY_LIMIT: u32 = 3;

pub enum Message {
    SendMessageResult(message::Message, Result<(), SdkError>),
    ClearUserTyping,
    ClearParticipantTyping,
}

pub struct Conversation {
    user_email: Arc<String>,
    user_display_name: Arc<String>,
    switchboards: HashMap<Arc<String>, Arc<Switchboard>>,
    participants: HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    messages: Vec<message::Message>,
    message_buffer: Vec<message::Message>,
    new_message: String,
    user_display_picture: Option<DisplayPicture>,
    contact_repository: ContactRepository,
    sqlite: Sqlite,
    participant_typing: Option<Arc<String>>,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
    user_typing: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    focused: bool,
    handle: Handle,
    visible: bool,
}

impl Conversation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_email: Arc<String>,
        user_display_name: Arc<String>,
        user_display_picture: Option<DisplayPicture>,
        contact_repository: ContactRepository,
        session_id: Arc<String>,
        switchboard: SwitchboardAndParticipants,
        main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
        sqlite: Sqlite,
        handle: Handle,
        visible: bool,
    ) -> Self {
        let messages = if switchboard.participants.len() > 1
            && let Ok(mut message_history) =
                sqlite.select_messages_by_session_id(&session_id, INITIAL_HISTORY_LIMIT)
        {
            message_history.reverse();
            message_history
        } else if switchboard.participants.len() == 1
            && let Some(participant) = switchboard.participants.first()
            && let Ok(mut message_history) =
                sqlite.select_messages(&user_email, participant, INITIAL_HISTORY_LIMIT)
        {
            message_history.reverse();
            message_history
        } else {
            Vec::new()
        };

        let mut participants = HashMap::with_capacity(switchboard.participants.len());
        for participant in &switchboard.participants {
            participants.insert(
                participant.clone(),
                contact_repository
                    .get_contact(participant)
                    .unwrap_or(Contact {
                        email: participant.clone(),
                        display_name: participant.clone(),
                        ..Contact::default()
                    }),
            );
        }

        let mut switchboards = HashMap::new();
        switchboards.insert(session_id, switchboard.switchboard);

        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            user_email,
            user_display_name,
            switchboards,
            participants,
            last_participant: None,
            messages,
            message_buffer: Vec::new(),
            new_message: "".to_string(),
            user_display_picture,
            contact_repository,
            sqlite,
            participant_typing: None,
            main_window_sender,
            sender,
            receiver,
            user_typing: false,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            focused: false,
            handle,
            visible,
        }
    }

    pub fn handle_event(&mut self, message: crate::main_window::Message, ctx: &egui::Context) {
        match message {
            crate::main_window::Message::NotificationServerEvent(event) => match event {
                msnp11_sdk::Event::DisplayName(display_name) => {
                    self.user_display_name = Arc::new(display_name);
                }

                msnp11_sdk::Event::PresenceUpdate { email, .. } => {
                    if let Some(contact) = self.participants.get_mut(&email) {
                        let email = Arc::new(email);
                        *contact = self
                            .contact_repository
                            .get_contact(&email)
                            .unwrap_or(Contact {
                                email: email.clone(),
                                display_name: email.clone(),
                                ..Contact::default()
                            });
                    } else if let Some(contact) = &mut self.last_participant
                        && *contact.email == email
                    {
                        let email = Arc::new(email);
                        *contact = self
                            .contact_repository
                            .get_contact(&email)
                            .unwrap_or(Contact {
                                email: email.clone(),
                                display_name: email.clone(),
                                ..Contact::default()
                            });
                    }
                }

                _ => (),
            },

            crate::main_window::Message::SwitchboardEvent(session_id, event) => {
                if self.switchboards.contains_key(&session_id) {
                    match event {
                        msnp11_sdk::Event::ParticipantInSwitchboard { email } => {
                            let email = Arc::new(email);
                            self.participants.insert(
                                email.clone(),
                                self.contact_repository
                                    .get_contact(&email)
                                    .unwrap_or(Contact {
                                        email: email.clone(),
                                        display_name: email.clone(),
                                        ..Contact::default()
                                    }),
                            );

                            if self.last_participant.is_none()
                                && self.participants.len() == 1
                                && let Ok(mut message_history) = self.sqlite.select_messages(
                                    &self.user_email,
                                    &email,
                                    INITIAL_HISTORY_LIMIT,
                                )
                            {
                                message_history.reverse();
                                self.messages = message_history;
                            }

                            if !self.message_buffer.is_empty()
                                && let Some(switchboard) = self.switchboards.get(&session_id)
                            {
                                for mut message in self.message_buffer.drain(..) {
                                    let switchboard = switchboard.clone();
                                    let sender = self.sender.clone();

                                    if !message.is_nudge {
                                        let plain_text = msnp11_sdk::PlainText {
                                            bold: message.bold,
                                            italic: message.italic,
                                            underline: message.underline,
                                            strikethrough: message.strikethrough,
                                            color: message.color.clone(),
                                            text: message.text.clone(),
                                        };

                                        run_future(
                                            self.handle.clone(),
                                            async move {
                                                switchboard.send_text_message(&plain_text).await
                                            },
                                            sender,
                                            move |result| {
                                                Message::SendMessageResult(
                                                    std::mem::take(&mut message),
                                                    result,
                                                )
                                            },
                                        );
                                    } else {
                                        run_future(
                                            self.handle.clone(),
                                            async move { switchboard.send_nudge().await },
                                            sender,
                                            move |result| {
                                                Message::SendMessageResult(
                                                    std::mem::take(&mut message),
                                                    result,
                                                )
                                            },
                                        );
                                    }
                                }
                            }
                        }

                        msnp11_sdk::Event::ParticipantLeftSwitchboard { email } => {
                            let participant = self.participants.remove(&email);
                            if self.participants.is_empty() && participant.is_some() {
                                self.last_participant = participant;
                            }
                        }

                        msnp11_sdk::Event::TypingNotification { email } => {
                            if self.participant_typing.is_none() {
                                self.participant_typing =
                                    if let Some(participant) = self.participants.get(&email) {
                                        Some(participant.display_name.clone())
                                    } else {
                                        Some(Arc::new(email))
                                    };

                                run_future(
                                    self.handle.clone(),
                                    async {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(5))
                                            .await
                                    },
                                    self.sender.clone(),
                                    |_| Message::ClearParticipantTyping,
                                );
                            }
                        }

                        msnp11_sdk::Event::TextMessage { email, message } => {
                            let message = message::Message {
                                sender: Arc::new(email),
                                receiver: Some(self.user_email.clone()),
                                is_nudge: false,
                                text: message.text,
                                bold: message.bold,
                                italic: message.italic,
                                underline: message.underline,
                                strikethrough: message.strikethrough,
                                session_id: None,
                                color: message.color,
                                is_history: false,
                                errored: false,
                            };

                            let _ = self.sqlite.insert_message(&message);
                            if !self.focused {
                                let _ = notify_rust::Notification::new()
                                    .summary(&format!(
                                        "{} said:",
                                        if let Some(participant) =
                                            self.participants.get(&message.sender)
                                        {
                                            &participant.display_name
                                        } else if let Some(participant) = &self.last_participant
                                            && participant.email == message.sender
                                        {
                                            &*participant.display_name
                                        } else {
                                            &message.sender
                                        }
                                    ))
                                    .body(&message.text)
                                    .show();

                                if let Some(session_id) = self.switchboards.keys().next() {
                                    if !self.visible {
                                        ctx.send_viewport_cmd_to(
                                            egui::ViewportId::from_hash_of(session_id),
                                            egui::ViewportCommand::Visible(true),
                                        );

                                        self.visible = true;
                                        ctx.send_viewport_cmd(
                                            egui::ViewportCommand::RequestUserAttention(
                                                egui::UserAttentionType::Informational,
                                            ),
                                        );
                                    } else {
                                        ctx.send_viewport_cmd_to(
                                            egui::ViewportId::from_hash_of(session_id),
                                            egui::ViewportCommand::RequestUserAttention(
                                                egui::UserAttentionType::Informational,
                                            ),
                                        );
                                    }
                                }
                            }

                            self.messages.push(message);
                            self.participant_typing = None;
                        }

                        msnp11_sdk::Event::Nudge { email } => {
                            let sender = Arc::new(email);
                            let message = message::Message {
                                sender: sender.clone(),
                                receiver: Some(self.user_email.clone()),
                                is_nudge: true,
                                text: format!(
                                    "{} just sent you a nudge!",
                                    if let Some(participant) = self.participants.get(&sender) {
                                        &participant.display_name
                                    } else if let Some(participant) = &self.last_participant
                                        && participant.email == sender
                                    {
                                        &*participant.display_name
                                    } else {
                                        &sender
                                    }
                                ),
                                bold: false,
                                italic: false,
                                underline: false,
                                strikethrough: false,
                                session_id: None,
                                color: "0".to_string(),
                                is_history: false,
                                errored: false,
                            };

                            let _ = self.sqlite.insert_message(&message);
                            if !self.focused {
                                let _ = notify_rust::Notification::new()
                                    .summary("New message")
                                    .body(&message.text)
                                    .show();

                                if let Some(session_id) = self.switchboards.keys().next() {
                                    if !self.visible {
                                        ctx.send_viewport_cmd_to(
                                            egui::ViewportId::from_hash_of(session_id),
                                            egui::ViewportCommand::Visible(true),
                                        );

                                        self.visible = true;
                                        ctx.send_viewport_cmd(
                                            egui::ViewportCommand::RequestUserAttention(
                                                egui::UserAttentionType::Informational,
                                            ),
                                        );
                                    } else {
                                        ctx.send_viewport_cmd_to(
                                            egui::ViewportId::from_hash_of(session_id),
                                            egui::ViewportCommand::RequestUserAttention(
                                                egui::UserAttentionType::Informational,
                                            ),
                                        );
                                    }
                                }
                            }

                            self.messages.push(message);
                            self.participant_typing = None;
                        }

                        _ => (),
                    }
                }
            }

            crate::main_window::Message::UserDisplayPictureChanged(picture) => {
                self.user_display_picture = Some(picture);
            }

            crate::main_window::Message::ContactDisplayPictureEvent { email, data } => {
                if let Some(contact) = self.participants.get_mut(&email)
                    && let Some(presence) = &contact.status
                    && let Some(msn_object) = &presence.msn_object
                {
                    contact.display_picture = Some(DisplayPicture {
                        data,
                        hash: Arc::new(msn_object.sha1d.clone()),
                    });
                } else if let Some(contact) = &mut self.last_participant
                    && let Some(presence) = &contact.status
                    && let Some(msn_object) = &presence.msn_object
                    && *contact.email == email
                {
                    contact.display_picture = Some(DisplayPicture {
                        data,
                        hash: Arc::new(msn_object.sha1d.clone()),
                    });
                }
            }

            _ => (),
        }
    }

    pub fn conversation(&mut self, ctx: &egui::Context) {
        let previous_focus = self.focused;
        self.focused = ctx.input(|input| input.viewport().focused.is_some_and(|focused| focused));

        if !previous_focus && self.focused {
            for participant in self.participants.values() {
                if let Some(status) = &participant.status
                    && let Some(msn_object_string) = status.msn_object_string.clone()
                    && let Some(switchboard) = self.switchboards.values().next().cloned()
                    && participant.display_picture.is_none()
                {
                    let email = participant.email.clone();
                    self.handle.spawn(async move {
                        switchboard
                            .request_contact_display_picture(&email, &msn_object_string)
                            .await
                    });
                }

                let _ = self.main_window_sender.send(
                    crate::main_window::Message::ContactChatWindowFocused(
                        participant.email.clone(),
                    ),
                );
            }
        }

        if let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::SendMessageResult(mut message, result) => {
                    if result.is_err() {
                        message.errored = true;
                    } else {
                        let _ = self.sqlite.insert_message(&message);
                    }

                    self.messages.push(message);
                }

                Message::ClearUserTyping => self.user_typing = false,
                Message::ClearParticipantTyping => self.participant_typing = None,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            tui(ui, ui.id().with("conversation-screen"))
                .reserve_available_space()
                .style(taffy::Style {
                    display: taffy::Display::Grid,
                    grid_template_columns: vec![fr(1.), length(50.)],
                    grid_template_rows: vec![
                        length(50.),
                        length(27.),
                        fr(1.),
                        length(10.),
                        length(25.),
                        length(92.),
                    ],
                    align_items: Some(taffy::AlignItems::Stretch),
                    size: percent(1.),
                    padding: length(15.),
                    gap: taffy::Size {
                        width: length(0.),
                        height: length(15.),
                    },
                    ..Default::default()
                })
                .show(|tui| {
                    let mut job = LayoutJob::default();
                    job.append("To: ", 0., TextFormat::default());

                    if self.participants.len() == 1
                        && let Some(contact) = self.participants.values().next()
                    {
                        job.append(
                            &contact.display_name,
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );

                        job.append(
                            &format!(" <{}>", contact.email),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );
                    } else if self.participants.len() > 1 {
                        job.append(
                            &format!("{} participants", self.participants.len()),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );
                    } else if let Some(contact) = &self.last_participant {
                        job.append(
                            &contact.display_name,
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );

                        job.append(
                            &format!(" <{}>", contact.email),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );
                    }

                    tui.style(taffy::Style {
                        justify_self: Some(taffy::JustifySelf::Start),
                        size: taffy::Size {
                            width: percent(0.93),
                            height: auto(),
                        },
                        ..Default::default()
                    })
                    .ui(|ui| {
                        ui.label(job);
                        ui.add_space(10.);

                        if self.participants.len() < 2
                            && ui.link("Load your entire conversation history with this contact").clicked()
                            && let Some(participant) = self.participants.values().next()
                            && let Ok(message_history) = self.sqlite.select_all_messages(&self.user_email, &participant.email)
                        {
                            self.messages = message_history;
                        }

                        ui.separator();
                    });

                    if self.participants.len() < 2 {
                        tui.style(taffy::Style {
                            justify_self: Some(taffy::JustifySelf::End),
                            grid_row: span(2),
                            ..Default::default()
                        })
                        .add_with_border(|tui| {
                            tui.ui(|ui| {
                                if let Some(participant) = self.participants.values().next() {
                                    ui.add(
                                        if let Some(picture) = participant.display_picture.clone() {
                                            egui::Image::from_bytes(format!("bytes://{}.png", picture.hash), picture.data)
                                                .fit_to_exact_size(egui::Vec2::splat(90.))
                                                .corner_radius(
                                                    ui.visuals()
                                                        .widgets
                                                        .noninteractive
                                                        .corner_radius,
                                                )
                                                .alt_text("Contact display picture")
                                        } else {
                                            egui::Image::new(svg::default_display_picture())
                                                .fit_to_exact_size(egui::Vec2::splat(90.))
                                                .alt_text("Default display picture")
                                        },
                                    )
                                } else if let Some(participant) = &self.last_participant {
                                    ui.add(
                                        if let Some(picture) = participant.display_picture.clone() {
                                            egui::Image::from_bytes(format!("bytes://{}.png", picture.hash), picture.data)
                                                .fit_to_exact_size(egui::Vec2::splat(90.))
                                                .corner_radius(
                                                    ui.visuals()
                                                        .widgets
                                                        .noninteractive
                                                        .corner_radius,
                                                )
                                                .alt_text("Contact display picture")
                                        } else {
                                            egui::Image::new(svg::default_display_picture())
                                                .fit_to_exact_size(egui::Vec2::splat(90.))
                                                .alt_text("Default display picture")
                                        },
                                    )
                                } else {
                                    ui.add(
                                        egui::Image::new(svg::default_display_picture())
                                            .fit_to_exact_size(egui::Vec2::splat(90.))
                                            .alt_text("Default display picture"),
                                    )
                                }
                            })
                        });
                    } else {
                        tui.style(taffy::Style {
                            justify_self: Some(taffy::JustifySelf::Start),
                            grid_row: span(3),
                            display: taffy::Display::Grid,
                            grid_template_columns: vec![fr(1.), fr(1.)],
                            grid_template_rows: vec![repeat("auto-fill", vec![length(43.)])],
                            align_items: Some(taffy::AlignItems::Center),
                            gap: length(5.),
                            ..Default::default()
                        })
                        .add(|tui| {
                            for participant in self.participants.values() {
                                tui.style(taffy::Style {
                                    justify_self: Some(taffy::JustifySelf::Start),
                                    size: taffy::Size {
                                        width: length(45.),
                                        height: auto(),
                                    },
                                    margin: percent(-0.9),
                                    ..Default::default()
                                })
                                .add_with_border(|tui| {
                                    tui.ui(|ui| {
                                        ui.add(
                                            if let Some(picture) =
                                                participant.display_picture.clone()
                                            {
                                                egui::Image::from_bytes(
                                                    format!("bytes://{}.png", picture.hash), picture.data
                                                )
                                                .fit_to_exact_size(egui::Vec2::splat(44.))
                                                .corner_radius(
                                                    ui.visuals()
                                                        .widgets
                                                        .noninteractive
                                                        .corner_radius,
                                                )
                                                .alt_text("Contact display picture")
                                            } else {
                                                egui::Image::new(svg::default_display_picture())
                                                    .fit_to_exact_size(egui::Vec2::splat(44.))
                                                    .alt_text("Default display picture")
                                            },
                                        )
                                    });
                                });

                                tui.style(taffy::Style {
                                    margin: percent(-0.9),
                                    max_size: percent(1.),
                                    ..Default::default()
                                })
                                .ui_add(
                                    egui::Label::new(participant.display_name.as_str()).truncate(),
                                );
                            }
                        })
                    }

                    tui.style(taffy::Style {
                        justify_self: Some(taffy::JustifySelf::Start),
                        size: taffy::Size {
                            width: percent(0.93),
                            height: auto(),
                        },
                        grid_row: span(2),
                        ..Default::default()
                    })
                    .ui(|ui| {
                        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                            for message in self.messages.iter() {
                                ui.with_layout(
                                    egui::Layout::top_down_justified(egui::Align::LEFT),
                                    |ui| {
                                        let display_name = if let Some(participant) =
                                            self.participants.get(&message.sender)
                                        {
                                            &participant.display_name
                                        } else if let Some(participant) = &self.last_participant
                                            && participant.email == message.sender
                                        {
                                            &*participant.display_name
                                        } else if message.sender == self.user_email {
                                            &self.user_display_name
                                        } else {
                                            &message.sender
                                        };

                                        if message.is_history {
                                            ui.style_mut().visuals.override_text_color =
                                                Some(egui::Color32::GRAY);
                                        }

                                        if !message.is_nudge && !message.errored {
                                            let id = ui
                                                .label(format!("{} said:", display_name))
                                                .id;

                                            ui.indent(id, |ui| {
                                                let mut job = LayoutJob::default();
                                                job.append(
                                                    &message.text.replace("\r\n", "\n"),
                                                    0.,
                                                    TextFormat {
                                                        font_id: if message.bold {
                                                            FontId::new(
                                                                FontSelection::Default
                                                                    .resolve(ui.style())
                                                                    .size,
                                                                egui::FontFamily::Name(
                                                                    "Bold".into(),
                                                                ),
                                                            )
                                                        } else {
                                                            FontSelection::Default
                                                                .resolve(ui.style())
                                                        },
                                                        color: ui.visuals().text_color(),
                                                        italics: message.italic,
                                                        underline: if message.underline {
                                                            ui.visuals().window_stroke
                                                        } else {
                                                            Default::default()
                                                        },
                                                        strikethrough: if message.strikethrough {
                                                            ui.visuals().window_stroke
                                                        } else {
                                                            Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                );

                                                ui.label(job);
                                            });
                                        } else if message.errored {
                                            ui.add_sized([20., 10.], egui::Separator::default());
                                            let id = ui
                                                .label("The following message could not be delivered to all recipients:")
                                                .id;

                                            ui.indent(id, |ui| {
                                                let mut job = LayoutJob::default();
                                                job.append(
                                                    &message.text.replace("\r\n", "\n"),
                                                    0.,
                                                    TextFormat {
                                                        font_id: if message.bold {
                                                            FontId::new(
                                                                FontSelection::Default
                                                                    .resolve(ui.style())
                                                                    .size,
                                                                egui::FontFamily::Name(
                                                                    "Bold".into(),
                                                                ),
                                                            )
                                                        } else {
                                                            FontSelection::Default
                                                                .resolve(ui.style())
                                                        },
                                                        color: egui::Color32::GRAY,
                                                        italics: message.italic,
                                                        underline: if message.underline {
                                                            ui.visuals().window_stroke
                                                        } else {
                                                            Default::default()
                                                        },
                                                        strikethrough: if message.strikethrough {
                                                            ui.visuals().window_stroke
                                                        } else {
                                                            Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                );

                                                ui.label(job);
                                                ui.add_sized([20., 10.], egui::Separator::default());
                                            });
                                        } else {
                                            ui.add_sized([20., 10.], egui::Separator::default());
                                            ui.label(&message.text);
                                            ui.add_sized([20., 10.], egui::Separator::default());
                                        }
                                    },
                                );

                                ui.add_space(5.);
                            }
                        });
                    });

                    tui.style(taffy::Style {
                        justify_self: Some(taffy::JustifySelf::Start),
                        size: taffy::Size {
                            width: percent(0.93),
                            height: auto(),
                        },
                        grid_row: line(4),
                        ..Default::default()
                    })
                    .label(
                        if let Some(typing_participant) = self.participant_typing.clone() {
                            let display_name = if let Some(participant) =
                                self.participants.get(&typing_participant)
                            {
                                &participant.display_name
                            } else if let Some(participant) = &self.last_participant
                                && participant.email == typing_participant
                            {
                                &*participant.display_name
                            } else {
                                &typing_participant
                            };

                            format!("{} is writing a message...", display_name)
                        } else {
                            "".to_string()
                        },
                    );

                    tui.style(taffy::Style {
                        justify_self: Some(taffy::JustifySelf::Stretch),
                        grid_row: line(5),
                        ..Default::default()
                    })
                    .ui(|ui| {
                        ui.style_mut().spacing.button_padding = egui::Vec2::new(10., 6.5);
                        ui.style_mut()
                            .text_styles
                            .insert(egui::TextStyle::Button, FontId::proportional(12.));

                        ui.horizontal(|ui| {
                            if ui.button("Nudge").clicked()
                                && let Some(switchboard) = self.switchboards.values().next()
                            {
                                let mut message = message::Message {
                                    sender: self.user_email.clone(),
                                    receiver: if self.participants.len() == 1 {
                                        self.participants
                                            .values()
                                            .next()
                                            .map(|participant| participant.email.clone())
                                    } else if self.participants.is_empty() {
                                        self.last_participant
                                            .as_ref()
                                            .map(|participant| participant.email.clone())
                                    } else {
                                        None
                                    },
                                    is_nudge: true,
                                    text: "You just sent a nudge!".to_string(),
                                    bold: false,
                                    italic: false,
                                    underline: false,
                                    strikethrough: false,
                                    session_id: None,
                                    color: "0".to_string(),
                                    is_history: false,
                                    errored: false,
                                };

                                if !self.participants.is_empty() {
                                    let switchboard = switchboard.clone();
                                    run_future(
                                        self.handle.clone(),
                                        async move { switchboard.send_nudge().await },
                                        self.sender.clone(),
                                        move |result| {
                                            Message::SendMessageResult(
                                                std::mem::take(&mut message),
                                                result,
                                            )
                                        },
                                    );
                                } else {
                                    self.message_buffer.push(message);
                                    if let Some(last_participant) = self.last_participant.clone() {
                                        let switchboard = switchboard.clone();
                                        self.handle.spawn(async move {
                                            switchboard.invite(&last_participant.email).await
                                        });
                                    }
                                }
                            }

                            ui.add_space(5.);

                            ui.style_mut().spacing.button_padding = egui::Vec2::new(10., 5.);
                            ui.style_mut()
                                .text_styles
                                .insert(egui::TextStyle::Button, FontId::monospace(16.));

                            ui.toggle_value(&mut self.bold, "B");
                            ui.toggle_value(&mut self.italic, "I");
                            ui.toggle_value(&mut self.underline, "U");
                            ui.toggle_value(&mut self.strikethrough, "S");
                        });
                    });

                    tui.style(taffy::Style {
                        justify_self: Some(taffy::JustifySelf::Start),
                        size: taffy::Size {
                            width: percent(0.93),
                            height: auto(),
                        },
                        grid_row: line(6),
                        ..Default::default()
                    })
                    .ui(|ui| {
                        egui::ScrollArea::vertical()
                            .show(ui, |ui| {
                                let multiline = ui.add(
                                    egui::TextEdit::multiline(&mut self.new_message)
                                        .desired_rows(5)
                                        .desired_width(f32::INFINITY)
                                        .return_key(Some(egui::KeyboardShortcut::new(
                                            egui::Modifiers::SHIFT,
                                            egui::Key::Enter,
                                        ))),
                                );

                                if multiline.changed()
                                    && !self.user_typing
                                    && let Some(switchboard) = self.switchboards.values().next()
                                {
                                    self.user_typing = true;
                                    self.handle.block_on(async {
                                        let _ =
                                            switchboard.send_typing_user(&self.user_email).await;
                                    });

                                    run_future(
                                        self.handle.clone(),
                                        async { tokio::time::sleep(tokio::time::Duration::from_secs(5)).await },
                                        self.sender.clone(),
                                        |_| Message::ClearUserTyping
                                    );
                                }

                                if multiline.has_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                    && let Some(switchboard) = self.switchboards.values().next()
                                    && !self.new_message.trim().is_empty()
                                {
                                    let mut message = message::Message {
                                        sender: self.user_email.clone(),
                                        receiver: if self.participants.len() == 1 {
                                            self.participants
                                                .values()
                                                .next()
                                                .map(|participant| participant.email.clone())
                                        } else if self.participants.is_empty() {
                                            self.last_participant
                                                .as_ref()
                                                .map(|participant| participant.email.clone())
                                        } else {
                                            None
                                        },
                                        is_nudge: false,
                                        text: self.new_message.replace("\n", "\r\n"),
                                        bold: self.bold,
                                        italic: self.italic,
                                        underline: self.underline,
                                        strikethrough: self.strikethrough,
                                        session_id: None,
                                        color: "0".to_string(),
                                        is_history: false,
                                        errored: false,
                                    };

                                    let plain_text = msnp11_sdk::PlainText {
                                        bold: message.bold,
                                        italic: message.italic,
                                        underline: message.underline,
                                        strikethrough: message.strikethrough,
                                        color: message.color.clone(),
                                        text: message.text.clone(),
                                    };

                                    self.new_message = "".to_string();
                                    if !self.participants.is_empty() {
                                        let switchboard = switchboard.clone();
                                        run_future(
                                            self.handle.clone(),
                                            async move {
                                                switchboard.send_text_message(&plain_text).await
                                            },
                                            self.sender.clone(),
                                            move |result| {
                                                Message::SendMessageResult(
                                                    std::mem::take(&mut message),
                                                    result,
                                                )
                                            },
                                        );
                                    } else {
                                        self.message_buffer.push(message);
                                        if let Some(last_participant) =
                                            self.last_participant.clone()
                                        {
                                            let switchboard = switchboard.clone();
                                            self.handle.spawn(async move {
                                                switchboard.invite(&last_participant.email).await
                                            });
                                        }
                                    }
                                }
                            });
                    });

                    tui.style(taffy::Style {
                        justify_self: Some(taffy::JustifySelf::End),
                        grid_row: line(6),
                        ..Default::default()
                    })
                    .add_with_border(|tui| {
                        tui.ui(|ui| {
                            ui.add(if let Some(picture) = self.user_display_picture.clone() {
                                egui::Image::from_bytes(format!("bytes://{}.png", picture.hash), picture.data)
                                    .fit_to_exact_size(egui::Vec2::splat(90.))
                                    .corner_radius(
                                        ui.visuals().widgets.noninteractive.corner_radius,
                                    )
                                    .alt_text("User display picture")
                            } else {
                                egui::Image::new(svg::default_display_picture())
                                    .fit_to_exact_size(egui::Vec2::splat(90.))
                                    .alt_text("Default display picture")
                            })
                        });
                    });
                });
        });
    }

    pub fn get_participants(&self) -> &HashMap<Arc<String>, Contact> {
        &self.participants
    }

    pub fn get_last_participant(&self) -> &Option<Contact> {
        &self.last_participant
    }

    pub fn add_switchboard(
        &mut self,
        session_id: Arc<String>,
        switchboard: SwitchboardAndParticipants,
    ) {
        self.switchboards
            .insert(session_id, switchboard.switchboard);
    }

    pub fn leave_switchboards(&self) {
        for switchboard in self.switchboards.values() {
            let _ = self
                .handle
                .block_on(async { switchboard.disconnect().await });
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn get_title(&self) -> String {
        if !self.participants.is_empty() {
            let mut title = "".to_string();
            for participant in self.participants.values() {
                title.push_str(&participant.display_name);
                title.push_str(", ");
            }

            title.pop();
            title.pop();
            title.push_str(" - Conversation");
            title
        } else if let Some(last_participant) = &self.last_participant {
            let mut title = (*last_participant.display_name).clone();
            title.push_str(" - Conversation");
            title
        } else {
            "Conversation".to_string()
        }
    }
}
