use crate::helpers::run_future::run_future;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{Client, MsnpList};
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct AddContact {
    email: String,
    display_name: String,
    client: Arc<Client>,
    contacts_sender: std::sync::mpsc::Sender<crate::screens::contacts::contacts::Message>,
    handle: Handle,
}

impl AddContact {
    pub fn new(
        client: Arc<Client>,
        contacts_sender: std::sync::mpsc::Sender<crate::screens::contacts::contacts::Message>,
        handle: Handle,
    ) -> Self {
        Self {
            email: String::default(),
            display_name: String::default(),
            client,
            contacts_sender,
            handle,
        }
    }

    pub fn add_contact(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame {
                    fill: ctx.style().visuals.window_fill,
                    ..Default::default()
                }
                .inner_margin(5.),
            )
            .show(ctx, |ui| {
                tui(ui, ui.id().with("add-contact-screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: Some(taffy::AlignItems::Stretch),
                        size: taffy::Size {
                            width: percent(1.),
                            height: auto(),
                        },
                        padding: length(20.),
                        gap: length(15.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.ui(|ui| {
                            let label = ui.label("Contact e-mail:");
                            ui.add_space(3.);
                            ui.add(
                                egui::text_edit::TextEdit::singleline(&mut self.email)
                                    .hint_text("Contact e-mail")
                                    .min_size(egui::Vec2::new(340., 5.)),
                            )
                            .labelled_by(label.id);
                        });

                        tui.ui(|ui| {
                            let label = ui.label("Contact display name:");
                            ui.add_space(3.);
                            ui.add(
                                egui::text_edit::TextEdit::singleline(&mut self.display_name)
                                    .hint_text("Contact display name")
                                    .min_size(egui::Vec2::new(340., 5.)),
                            )
                            .labelled_by(label.id);
                        });

                        tui.style(taffy::Style {
                            align_self: Some(taffy::AlignItems::Center),
                            ..Default::default()
                        })
                        .ui(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Ok").clicked() {
                                    let contacts_sender = self.contacts_sender.clone();
                                    let client = self.client.clone();
                                    let email = self.email.clone();

                                    let display_name = if !self.display_name.is_empty() {
                                        self.display_name.clone()
                                    } else {
                                        self.email.clone()
                                    };

                                    run_future(self.handle.clone(),
                                               async move { client.add_contact(&email, &display_name, MsnpList::ForwardList).await },
                                               contacts_sender,
                                               crate::screens::contacts::contacts::Message::AddContactResult);

                                    let _ = self.contacts_sender.send(crate::screens::contacts::contacts::Message::CloseAddContact);
                                }

                                if ui.button("Cancel").clicked() {
                                    let _ = self.contacts_sender.send(crate::screens::contacts::contacts::Message::CloseAddContact);
                                }
                            })
                        })
                    })
            });

        if ctx.input(|i| i.viewport().close_requested()) {
            let _ = self
                .contacts_sender
                .send(crate::screens::contacts::contacts::Message::CloseAddContact);
        }
    }
}
