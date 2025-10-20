use crate::contact_repository::ContactRepository;
use crate::helpers::run_future::run_future;
use crate::models::contact::Contact;
use crate::models::display_picture::DisplayPicture;
use crate::svg;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, TextFormat, Ui};
use msnp11_sdk::{Client, MsnpList, MsnpStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

#[allow(clippy::too_many_arguments)]
pub fn category_collapsing_header(
    ui: &mut Ui,
    name: &str,
    selected_contact: &mut Option<Arc<String>>,
    contacts: &mut HashMap<Arc<String>, Contact>,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    contacts_sender: std::sync::mpsc::Sender<crate::screens::contacts::contacts::Message>,
    handle: Handle,
    user_email: Arc<String>,
    user_display_name: Arc<String>,
    user_display_picture: Option<DisplayPicture>,
    user_status: crate::screens::contacts::status_selector::Status,
    contact_repository: ContactRepository,
    client: Arc<Client>,
) {
    if egui::CollapsingHeader::new(name)
        .default_open(true)
        .show(ui, |ui| {
            if contacts.is_empty() {
                ui.label("No contacts in this category");
            } else {
                for contact in contacts.values_mut() {
                    let user_email = user_email.clone();
                    let user_display_name = user_display_name.clone();
                    let user_display_picture = user_display_picture.clone();
                    let contact_repository = contact_repository.clone();
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
                            &contact.display_name,
                            0.,
                            TextFormat {
                                font_id: FontId::proportional(14.),
                                color: ui.visuals().text_color(),
                                ..Default::default()
                            },
                        );

                        if let Some(personal_message) = contact.personal_message.as_ref()
                            && !personal_message.is_empty()
                        {
                            contact_job.append(" - ", 0., TextFormat {
                                font_id: FontId::proportional(14.),
                                color: ui.visuals().weak_text_color(),
                                ..Default::default()
                            });

                            contact_job.append(
                                personal_message,
                                0.,
                                TextFormat {
                                    font_id: FontId::proportional(14.),
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

                        if label.clicked() || label.secondary_clicked() {
                            *selected_contact = Some(contact.email.clone());
                        }

                        if label.double_clicked()
                            && contact.status.is_some()
                            && !contact.opening_conversation
                            && user_status != crate::screens::contacts::status_selector::Status::AppearOffline {
                            contact.opening_conversation = true;

                            let _ = main_window_sender.send(crate::main_window::Message::OpenConversation {
                                user_email: user_email.clone(),
                                user_display_name: user_display_name.clone(),
                                user_display_picture: user_display_picture.clone(),
                                contact_repository: contact_repository.clone(),
                                contact: contact.clone(),
                                client: client.clone(),
                            });
                        }

                        ui.style_mut().spacing.button_padding = egui::Vec2::splat(5.);
                        label.context_menu(|ui| {
                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::LEFT),
                                |ui| {
                                    if ui.button("Send an Instant Message").clicked() 
                                        && contact.status.is_some()
                                        && !contact.opening_conversation
                                        && user_status != crate::screens::contacts::status_selector::Status::AppearOffline {
                                        contact.opening_conversation = true;

                                        let _ = main_window_sender.send(crate::main_window::Message::OpenConversation {
                                            user_email,
                                            user_display_name,
                                            user_display_picture,
                                            contact_repository,
                                            contact: contact.clone(),
                                            client: client.clone(),
                                        });
                                    }

                                    ui.separator();

                                    if contact.lists.contains(&MsnpList::BlockList) {
                                        let email = contact.email.clone();
                                        let contact = contact.email.clone();
                                        let client = client.clone();

                                        if ui.button("Unblock").clicked() {
                                            run_future(
                                                handle.clone(),
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
                                                handle.clone(),
                                                async move { client.block_contact(&email).await },
                                                contacts_sender.clone(),
                                                move |result| crate::screens::contacts::contacts::Message::BlockResult(contact.clone(), result),
                                            );
                                        }
                                    }

                                    if ui.button("Delete Contact").clicked() {
                                        let guid = contact.guid.clone();
                                        let contact = contact.email.clone();

                                        run_future(
                                            handle.clone(),
                                            async move { client.remove_contact_from_forward_list(&guid).await },
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
        }).header_response.clicked()
        && let Some(contact) = selected_contact && contacts.contains_key(contact) {
            *selected_contact = None;
        }
}
