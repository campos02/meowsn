use crate::models::contact::Contact;
use crate::models::message;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, FontSelection, TextFormat};
use egui_taffy::taffy::prelude::{auto, percent, span};
use egui_taffy::{Tui, TuiBuilderLogic, taffy};
use std::collections::HashMap;
use std::sync::Arc;

pub fn messages(
    tui: &mut Tui,
    participants: &HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    user_email: Arc<String>,
    user_display_name: Arc<String>,
    messages: &Vec<message::Message>,
) {
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
            egui::ScrollArea::vertical().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
                for message in messages.iter() {
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::LEFT),
                        |ui| {
                            let display_name = if let Some(participant) =
                                participants.get(&message.sender)
                            {
                                &participant.display_name
                            } else if let Some(participant) = &last_participant
                                && participant.email == message.sender
                            {
                                &*participant.display_name
                            } else if message.sender == user_email {
                                &user_display_name
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
}
