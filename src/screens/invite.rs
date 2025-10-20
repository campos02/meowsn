use crate::helpers::run_future::run_future;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::Switchboard;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct Invite {
    email: String,
    switchboard: Arc<Switchboard>,
    conversation_sender: std::sync::mpsc::Sender<crate::screens::conversation::Message>,
    handle: Handle,
}

impl Invite {
    pub fn new(
        switchboard: Arc<Switchboard>,
        conversation_sender: std::sync::mpsc::Sender<crate::screens::conversation::Message>,
        handle: Handle,
    ) -> Self {
        Self {
            email: String::default(),
            switchboard,
            conversation_sender,
            handle,
        }
    }

    pub fn invite(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame {
                    fill: ctx.style().visuals.window_fill,
                    ..Default::default()
                }
                .inner_margin(5.),
            )
            .show(ctx, |ui| {
                tui(ui, ui.id().with("invite-screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: Some(taffy::AlignItems::Stretch),
                        size: taffy::Size {
                            width: percent(0.9),
                            height: auto(),
                        },
                        padding: length(20.),
                        gap: length(15.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.ui(|ui| {
                            let label = ui.label("Enter e-mail address:");
                            ui.add_space(3.);
                            ui.add(
                                egui::text_edit::TextEdit::singleline(&mut self.email)
                                    .hint_text("E-mail address")
                                    .min_size(egui::Vec2::new(340., 5.)),
                            )
                            .labelled_by(label.id);
                        });

                        tui.style(taffy::Style {
                            align_self: Some(taffy::AlignItems::Center),
                            size: taffy::Size {
                                width: percent(0.2),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Ok").clicked() {
                                    let conversation_sender = self.conversation_sender.clone();
                                    let switchboard = self.switchboard.clone();
                                    let email = self.email.clone();

                                    if !email.trim().is_empty() {
                                        run_future(
                                            self.handle.clone(),
                                            async move { switchboard.invite(&email).await },
                                            conversation_sender,
                                            crate::screens::conversation::Message::InviteResult,
                                        );
                                    }

                                    let _ = self
                                        .conversation_sender
                                        .send(crate::screens::conversation::Message::CloseInvite);
                                }

                                if ui.button("Cancel").clicked() {
                                    let _ = self
                                        .conversation_sender
                                        .send(crate::screens::conversation::Message::CloseInvite);
                                }
                            })
                        })
                    })
            });

        if ctx.input(|i| i.viewport().close_requested()) {
            let _ = self
                .conversation_sender
                .send(crate::screens::conversation::Message::CloseInvite);
        }
    }
}
