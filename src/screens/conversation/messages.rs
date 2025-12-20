use crate::models::contact::Contact;
use crate::models::message;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, FontSelection, TextFormat};
use egui_taffy::taffy::prelude::{auto, percent, span};
use egui_taffy::{Tui, TuiBuilderLogic, taffy};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::Arc;

pub fn messages(
    tui: &mut Tui,
    participants: &BTreeMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
    user_email: Arc<String>,
    user_display_name: Arc<String>,
    messages: &[message::Message],
) {
    let url_regex = Regex::new(
        r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_+.~#?&/=]*)",
    );

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
                                    Some(if ui.visuals().dark_mode {
                                        egui::Color32::GRAY
                                    } else {
                                        egui::Color32::from_gray(120)
                                    });
                            }

                            if !message.is_nudge && !message.errored {
                                let id = ui
                                    .label(format!("{} said:", display_name))
                                    .id;

                                ui.indent(id, |ui| {
                                    display_text_message(ui, message, &url_regex, ui.visuals().text_color());
                                });
                            } else if message.errored {
                                ui.add_sized([20., 10.], egui::Separator::default());
                                let id = ui
                                    .label("The following message could not be delivered to all recipients:")
                                    .id;

                                ui.indent(id, |ui| {
                                    display_text_message(ui, message, &url_regex, if ui.visuals().dark_mode {
                                        egui::Color32::GRAY
                                    } else {
                                        egui::Color32::from_gray(120)
                                    });
                                });

                                ui.add_sized([20., 10.], egui::Separator::default());
                            } else {
                                // Nudge
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

fn display_text_message(
    ui: &mut egui::Ui,
    message: &message::Message,
    url_regex: &Result<Regex, regex::Error>,
    text_color: egui::Color32,
) {
    ui.style_mut().spacing.item_spacing.x = 0.;
    ui.horizontal_wrapped(|ui| {
        for word in message.text.split(" ") {
            let is_url = if let Ok(url_regex) = &url_regex
                && url_regex.is_match(word)
            {
                true
            } else {
                false
            };

            let mut job = LayoutJob::default();
            job.append(
                word,
                0.,
                TextFormat {
                    font_id: if message.bold {
                        FontId::new(
                            FontSelection::Default.resolve(ui.style()).size,
                            egui::FontFamily::Name("Bold".into()),
                        )
                    } else {
                        FontSelection::Default.resolve(ui.style())
                    },
                    color: if is_url {
                        ui.style().visuals.hyperlink_color
                    } else {
                        text_color
                    },
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

            if is_url {
                ui.hyperlink_to(job, word).on_hover_text(word);
            } else {
                ui.label(job);
            }

            ui.label(" ");
        }
    });
}
