use crate::contact_repository::ContactRepository;
use crate::models::contact::Contact;
use crate::models::message;
use crate::models::switchboard_and_participants::SwitchboardAndParticipants;
use crate::sqlite::Sqlite;
use crate::svg;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, FontSelection, TextFormat};
use egui_taffy::taffy::prelude::{auto, fr, length, line, percent, repeat, span};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::Switchboard;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct Conversation {
    user_email: Arc<String>,
    user_display_name: Arc<String>,
    switchboards: HashMap<Arc<String>, Arc<Switchboard>>,
    participants: HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    messages: Vec<message::Message>,
    message_buffer: Vec<message::Message>,
    new_message: String,
    user_display_picture: Option<Arc<[u8]>>,
    contact_repository: ContactRepository,
    sqlite: Sqlite,
    participant_typing: Option<Arc<String>>,
    user_typing: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    focused: bool,
    handle: Handle,
}

impl Conversation {
    pub fn new(
        user_email: Arc<String>,
        user_display_name: Arc<String>,
        user_display_picture: Option<Arc<[u8]>>,
        contact_repository: ContactRepository,
        session_id: Arc<String>,
        switchboard: SwitchboardAndParticipants,
        sqlite: Sqlite,
        handle: Handle,
    ) -> Self {
        let mut messages = Vec::new();
        if switchboard.participants.len() > 1
            && let Ok(message_history) = sqlite.select_messages_by_session_id(&session_id)
        {
            messages = message_history;
        }

        let mut participants = HashMap::with_capacity(switchboard.participants.len());
        for participant in &switchboard.participants {
            participants.insert(
                participant,
                contact_repository
                    .get_contact(participant)
                    .unwrap_or(Contact {
                        email: participant.clone(),
                        display_name: participant.clone(),
                        ..Contact::default()
                    }),
            );

            if switchboard.participants.len() == 1
                && let Ok(message_history) = sqlite.select_messages(&user_email, &participant)
            {
                messages = message_history;
            }
        }

        let mut switchboards = HashMap::new();
        switchboards.insert(session_id, switchboard.switchboard);

        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            user_email,
            user_display_name,
            switchboards,
            participants: HashMap::new(),
            last_participant: None,
            messages,
            message_buffer: Vec::new(),
            new_message: "".to_string(),
            user_display_picture,
            contact_repository,
            sqlite,
            participant_typing: None,
            sender,
            receiver,
            user_typing: false,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            focused: false,
            handle,
        }
    }

    pub fn handle_event(&mut self, message: crate::main_window::Message) {
        match message {
            crate::main_window::Message::NotificationServerEvent(event) => match event {
                msnp11_sdk::Event::DisplayName(display_name) => {
                    self.user_display_name = Arc::new(display_name);
                }

                msnp11_sdk::Event::PresenceUpdate {
                    email,
                    display_name,
                    presence,
                } => {
                    if let Some(contact) = self.participants.get_mut(&email) {
                        if let Some(msn_object) = &presence.msn_object
                            && msn_object.object_type == 3
                        {
                            contact.display_picture = self
                                .sqlite
                                .select_display_picture(&msn_object.sha1d)
                                .ok()
                                .map(|picture| {
                                    let picture = picture.into_boxed_slice();
                                    Arc::from(picture)
                                });
                        }

                        contact.display_name = Arc::new(display_name);
                    } else if let Some(contact) = &mut self.last_participant {
                        if *contact.email == email {
                            if let Some(msn_object) = &presence.msn_object
                                && msn_object.object_type == 3
                            {
                                contact.display_picture = self
                                    .sqlite
                                    .select_display_picture(&msn_object.sha1d)
                                    .ok()
                                    .map(|picture| {
                                        let picture = picture.into_boxed_slice();
                                        Arc::from(picture)
                                    });
                            }

                            contact.display_name = Arc::new(display_name);
                        }
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
                                && let Ok(message_history) =
                                    self.sqlite.select_messages(&self.user_email, &email)
                            {
                                self.messages = message_history;
                            }
                        }

                        msnp11_sdk::Event::ParticipantLeftSwitchboard { email } => {
                            let email = Arc::new(email);
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
                                    .summary(
                                        format!(
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
                                        )
                                        .as_str(),
                                    )
                                    .body(&message.text)
                                    .show();
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
                if let Some(contact) = self.participants.get_mut(&email) {
                    contact.display_picture = Some(data);
                } else if let Some(contact) = &mut self.last_participant {
                    contact.display_picture = Some(data);
                }
            }

            _ => (),
        }
    }

    pub fn conversation(&mut self, ctx: &egui::Context) {
        self.focused = ctx.input(|input| input.viewport().focused.is_some_and(|focused| focused));
        if let Ok(Message::SendMessageResult(mut message, result)) = self.receiver.try_recv() {
            if result.is_err() {
                message.errored = true;
            } else {
                let _ = self.sqlite.insert_message(&message);
            }

            self.messages.push(message);
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
                            contact.display_name.as_str(),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );

                        job.append(
                            format!(" <{}>", contact.email).as_str(),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );
                    } else if self.participants.len() > 1 {
                        job.append(
                            format!("{} participants", self.participants.len()).as_str(),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );
                    } else if let Some(contact) = &self.last_participant {
                        job.append(
                            contact.display_name.as_str(),
                            0.,
                            TextFormat {
                                ..Default::default()
                            },
                        );

                        job.append(
                            format!(" <{}>", contact.email).as_str(),
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

                        if self.participants.len() < 2 {
                            ui.link("Load your entire conversation history with this contact");
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
                                            egui::Image::from_bytes("bytes://picture.png", picture)
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
                                            egui::Image::from_bytes("bytes://picture.png", picture)
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
                                    size: taffy::Size {
                                        width: length(43.5),
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
                                                    "bytes://picture.png",
                                                    picture,
                                                )
                                                .fit_to_exact_size(egui::Vec2::splat(40.))
                                                .corner_radius(
                                                    ui.visuals()
                                                        .widgets
                                                        .noninteractive
                                                        .corner_radius,
                                                )
                                                .alt_text("Contact display picture")
                                            } else {
                                                egui::Image::new(svg::default_display_picture())
                                                    .fit_to_exact_size(egui::Vec2::splat(40.))
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
                        egui::ScrollArea::vertical().show(ui, |ui| {
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
                                                .label(format!("{} said:", display_name).as_str())
                                                .id;

                                            ui.indent(id, |ui| {
                                                let mut job = LayoutJob::default();
                                                job.append(
                                                    message.text.replace("\r\n", "\n").as_str(),
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
                                                    message.text.replace("\r\n", "\n").as_str(),
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
                                            ui.label(
                                                format!("{} sent you a nudge!", display_name)
                                                    .as_str(),
                                            );
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
                            .stick_to_bottom(true)
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
                                    && let Some(switchboard) = self.switchboards.values().next()
                                {
                                    self.handle.block_on(async {
                                        let _ =
                                            switchboard.send_typing_user(&self.user_email).await;
                                    });
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
                                egui::Image::from_bytes("bytes://picture.png", picture)
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

    pub fn add_switchboard(&mut self, session_id: Arc<String>, switchboard: Arc<Switchboard>) {
        self.switchboards.insert(session_id, switchboard);
    }

    pub fn leave_switchboards(&self) {
        for switchboard in self.switchboards.values() {
            let _ = self
                .handle
                .block_on(async { switchboard.disconnect().await });
        }
    }
}
