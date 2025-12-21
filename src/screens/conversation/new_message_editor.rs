use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::message;
use crate::screens::conversation::conversation::Message;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, FontSelection, TextFormat};
use egui_taffy::taffy::prelude::line;
use egui_taffy::{Tui, TuiBuilderLogic, taffy};
use msnp11_sdk::Switchboard;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::runtime::Handle;

#[allow(clippy::too_many_arguments)]
pub fn new_message_editor(
    tui: &mut Tui,
    participants: &BTreeMap<Arc<String>, Contact>,
    last_participant: &Option<Contact>,
    user_email: Arc<String>,
    switchboards: &HashMap<Arc<String>, Arc<Switchboard>>,
    sender: std::sync::mpsc::Sender<Message>,
    handle: Handle,
    message_buffer: &mut Vec<message::Message>,
    user_typing: &mut bool,
    bold: &mut bool,
    italic: &mut bool,
    underline: &mut bool,
    strikethrough: &mut bool,
    new_message: &mut String,
) {
    tui.style(taffy::Style {
        grid_row: line(5),
        ..Default::default()
    })
    .ui(|ui| {
        ui.style_mut().spacing.button_padding = egui::Vec2::new(10., 6.5);
        ui.horizontal(|ui| {
            if ui
                .button("Nudge")
                .on_hover_text("Send a nudge to this contact")
                .clicked()
            {
                let mut message = message::Message {
                    sender: user_email.clone(),
                    receiver: if participants.len() == 1 {
                        participants
                            .values()
                            .next()
                            .map(|participant| participant.email.clone())
                    } else if participants.is_empty() {
                        last_participant
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

                if let Some(switchboard) = switchboards.values().next() {
                    if !participants.is_empty() {
                        let switchboard = switchboard.clone();
                        run_future(
                            handle.clone(),
                            async move { switchboard.send_nudge().await },
                            sender.clone(),
                            move |result| {
                                Message::SendMessageResult(std::mem::take(&mut message), result)
                            },
                        );
                    } else {
                        message_buffer.push(message);
                        if let Some(last_participant) = last_participant.clone() {
                            let switchboard = switchboard.clone();
                            handle.spawn(async move {
                                switchboard.invite(&last_participant.email).await
                            });
                        }
                    }
                } else {
                    message_buffer.push(message);
                }
            }

            ui.add_space(5.);

            ui.style_mut().spacing.button_padding = egui::Vec2::new(10., 5.);
            ui.style_mut()
                .text_styles
                .insert(egui::TextStyle::Button, FontId::monospace(16.));

            ui.toggle_value(bold, "B").on_hover_text("Toggle bold");
            ui.toggle_value(italic, "I").on_hover_text("Toggle italic");

            ui.toggle_value(underline, "U")
                .on_hover_text("Toggle underline");

            ui.toggle_value(strikethrough, "S")
                .on_hover_text("Toggle strikethrough");
        });
    });

    tui.style(taffy::Style {
        grid_row: line(6),
        ..Default::default()
    })
    .ui(|ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut layout_job = LayoutJob::default();
                layout_job.append(
                    buf.as_str(),
                    0.,
                    TextFormat {
                        font_id: if *bold {
                            FontId::new(
                                FontSelection::Default.resolve(ui.style()).size,
                                egui::FontFamily::Name("Bold".into()),
                            )
                        } else {
                            FontSelection::Default.resolve(ui.style())
                        },
                        color: ui.visuals().text_color(),
                        italics: *italic,
                        underline: if *underline {
                            ui.visuals().window_stroke
                        } else {
                            Default::default()
                        },
                        strikethrough: if *strikethrough {
                            ui.visuals().window_stroke
                        } else {
                            Default::default()
                        },
                        ..Default::default()
                    },
                );

                layout_job.wrap.max_width = wrap_width;
                ui.fonts_mut(|f| f.layout_job(layout_job))
            };

            let multiline = ui
                .add(
                    egui::TextEdit::multiline(new_message)
                        .desired_rows(5)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter)
                        .return_key(Some(egui::KeyboardShortcut::new(
                            egui::Modifiers::SHIFT,
                            egui::Key::Enter,
                        ))),
                )
                .on_hover_text_at_pointer("Enter your message here and press Enter to send it");

            if multiline.changed()
                && !*user_typing
                && let Some(switchboard) = switchboards.values().next()
            {
                *user_typing = true;
                handle.block_on(async {
                    let _ = switchboard.send_typing_user(&user_email).await;
                });

                run_future(
                    handle.clone(),
                    async {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    },
                    sender.clone(),
                    |_| Message::ClearUserTyping,
                );
            }

            if multiline.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !new_message.trim().is_empty()
            {
                let mut message = message::Message {
                    sender: user_email.clone(),
                    receiver: if participants.len() == 1 {
                        participants
                            .values()
                            .next()
                            .map(|participant| participant.email.clone())
                    } else if participants.is_empty() {
                        last_participant
                            .as_ref()
                            .map(|participant| participant.email.clone())
                    } else {
                        None
                    },
                    is_nudge: false,
                    text: new_message.replace("\n", "\r\n"),
                    bold: *bold,
                    italic: *italic,
                    underline: *underline,
                    strikethrough: *strikethrough,
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

                *new_message = "".to_string();
                if let Some(switchboard) = switchboards.values().next() {
                    if !participants.is_empty() {
                        let switchboard = switchboard.clone();
                        run_future(
                            handle.clone(),
                            async move { switchboard.send_text_message(&plain_text).await },
                            sender.clone(),
                            move |result| {
                                Message::SendMessageResult(std::mem::take(&mut message), result)
                            },
                        );
                    } else {
                        message_buffer.push(message);
                        if let Some(last_participant) = last_participant.clone() {
                            let switchboard = switchboard.clone();
                            handle.spawn(async move {
                                switchboard.invite(&last_participant.email).await
                            });
                        }
                    }
                } else {
                    message_buffer.push(message);
                }
            }
        });
    });
}
