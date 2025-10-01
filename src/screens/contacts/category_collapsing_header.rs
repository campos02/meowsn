use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::svg;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{TextFormat, Ui};
use msnp11_sdk::{Client, MsnpList, MsnpStatus};
use std::collections::HashMap;
use std::sync::Arc;

pub fn category_collapsing_header(
    ui: &mut Ui,
    name: &str,
    selected_contact: &mut Option<Arc<String>>,
    contacts: &HashMap<Arc<String>, Contact>,
    contacts_sender: std::sync::mpsc::Sender<crate::screens::contacts::contacts::Message>,
    client: Arc<Client>,
) {
    egui::CollapsingHeader::new(name)
        .default_open(true)
        .show(ui, |ui| {
            if contacts.is_empty() {
                ui.label("No contacts in this category");
            } else {
                for contact in contacts.values() {
                    let client = client.clone();
                    ui.horizontal(|ui| {
                        let mut alt_text = "Contact is offline";
                        ui.add(
                            egui::Image::new(if let Some(status) = &contact.status {
                                if contact.lists.contains(&MsnpList::BlockList) {
                                    alt_text = "Contact is blocked";
                                    svg::default_display_picture_blocked()
                                } else {
                                    match status.status {
                                        MsnpStatus::Busy | MsnpStatus::OnThePhone => {
                                            alt_text = "Contact is busy";
                                            svg::default_display_picture_busy()
                                        }

                                        MsnpStatus::Away
                                        | MsnpStatus::Idle
                                        | MsnpStatus::BeRightBack
                                        | MsnpStatus::OutToLunch => {
                                            alt_text = "Contact is away";
                                            svg::default_display_picture_away()
                                        }

                                        _ => {
                                            alt_text = "Contact is online";
                                            svg::default_display_picture()
                                        }
                                    }
                                }
                            } else if contact.lists.contains(&MsnpList::BlockList) {
                                alt_text = "Contact is offline and blocked";
                                svg::default_display_picture_offline_blocked()
                            } else {
                                svg::default_display_picture_offline()
                            })
                            .fit_to_exact_size(egui::Vec2::splat(25.))
                            .alt_text(alt_text),
                        );

                        let mut contact_job = LayoutJob::default();
                        contact_job.append(
                            contact.display_name.as_str(),
                            0.,
                            TextFormat {
                                font_id: egui::FontId::proportional(13.),
                                ..Default::default()
                            },
                        );

                        if let Some(personal_message) = contact.personal_message.as_ref()
                            && !personal_message.is_empty()
                        {
                            contact_job.append(" - ", 0., TextFormat {
                                font_id: egui::FontId::proportional(13.),
                                color: ui.visuals().weak_text_color(),
                                ..Default::default()
                            });

                            contact_job.append(
                                personal_message,
                                0.,
                                TextFormat {
                                    font_id: egui::FontId::proportional(13.),
                                    color: ui.visuals().weak_text_color(),
                                    ..Default::default()
                                },
                            );
                        }

                        ui.style_mut().spacing.button_padding = egui::Vec2::new(5., 3.);

                        let label = ui.add(egui::Button::selectable(
                            selected_contact
                            .as_ref()
                            .is_some_and(|selected_contact| {
                                *selected_contact == contact.email
                            }),
                            contact_job
                        )
                        .truncate());

                        if label.clicked() {
                            *selected_contact = Some(contact.email.clone());
                        }

                        ui.style_mut().spacing.button_padding = egui::Vec2::splat(5.);
                        label.context_menu(|ui| {
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    ui.button("Send an Instant Message");
                                    ui.separator();

                                    if contact.lists.contains(&MsnpList::BlockList) {
                                        let email = contact.email.clone();
                                        let contact = contact.email.clone();
                                        let client = client.clone();

                                        if ui.button("Unblock").clicked() {
                                            run_future(
                                                async move { client.unblock_contact(&email).await },
                                                contacts_sender.clone(),
                                                move |result| crate::screens::contacts::contacts::Message::UnblockResult(contact.clone(), result),
                                            );
                                        }
                                    } else {
                                        let email = contact.email.clone();
                                        let contact = contact.email.clone();
                                        let client = client.clone();

                                        if ui.button("Block").clicked() {
                                            run_future(
                                                async move { client.block_contact(&email).await },
                                                contacts_sender.clone(),
                                                move |result| crate::screens::contacts::contacts::Message::BlockResult(contact.clone(), result),
                                            );
                                        }
                                    }

                                    if ui.button("Delete Contact").clicked() {
                                        let email = contact.email.clone();
                                        let contact = contact.email.clone();

                                        run_future(
                                            async move { client.remove_contact(&email, MsnpList::ForwardList).await },
                                            contacts_sender.clone(),
                                            move |result| crate::screens::contacts::contacts::Message::DeleteResult(contact.clone(), result),
                                        );
                                    }
                                },
                            );
                        });
                    });
                }
            }
        });
}
