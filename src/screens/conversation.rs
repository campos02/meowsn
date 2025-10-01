use crate::contact_repository::ContactRepository;
use crate::models::contact::Contact;
use crate::models::message;
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

pub struct Conversation {
    user_email: Arc<String>,
    user_display_name: Arc<String>,
    switchboards: HashMap<Arc<String>, Arc<Switchboard>>,
    contact_repository: ContactRepository,
    participants: HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    messages: Vec<message::Message>,
    message_buffer: Vec<message::Message>,
    new_message: String,
    user_display_picture: Option<Arc<[u8]>>,
    sqlite: Sqlite,
    participant_typing: Option<Arc<String>>,
    user_typing: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl Conversation {
    pub fn new(
        user_email: Arc<String>,
        user_display_name: Arc<String>,
        user_display_picture: Option<Arc<[u8]>>,
        contact_repository: ContactRepository,
        sqlite: Sqlite,
    ) -> Self {
        Self {
            user_email,
            user_display_name,
            switchboards: HashMap::new(),
            contact_repository,
            participants: HashMap::new(),
            last_participant: None,
            messages: vec![],
            message_buffer: vec![],
            new_message: "".to_string(),
            user_display_picture,
            sqlite,
            participant_typing: None,
            user_typing: false,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
        }
    }
}

impl eframe::App for Conversation {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
                                        } else {
                                            &message.sender
                                        };

                                        if message.is_history {
                                            ui.style_mut().visuals.override_text_color =
                                                Some(egui::Color32::GRAY);
                                        }

                                        if !message.is_nudge {
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
                    .label("is writing a message...");

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
                            ui.button("Nudge");
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
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut self.new_message)
                                    .desired_rows(5)
                                    .desired_width(f32::INFINITY)
                                    .return_key(Some(egui::KeyboardShortcut::new(
                                        egui::Modifiers::SHIFT,
                                        egui::Key::Enter,
                                    ))),
                            );
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
}
