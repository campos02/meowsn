use crate::models::contact::Contact;
use crate::svg;
use eframe::egui;
use eframe::egui::text::LayoutJob;
use eframe::egui::{TextFormat, Ui};
use msnp11_sdk::{MsnpList, MsnpStatus};
use std::collections::HashMap;
use std::sync::Arc;

pub fn category_collapsing_header(
    ui: &mut Ui,
    name: &str,
    selected_contact: &mut Option<Arc<String>>,
    contacts: &HashMap<Arc<String>, Contact>,
) {
    egui::CollapsingHeader::new(name)
        .default_open(true)
        .show(ui, |ui| {
            if contacts.is_empty() {
                ui.label("No contacts in this category");
            } else {
                for contact in contacts.values() {
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
                            TextFormat::default(),
                        );
                        contact_job.append(" - ", 0., TextFormat::default());

                        if let Some(personal_message) = contact.personal_message.as_ref() {
                            contact_job.append(
                                personal_message,
                                0.,
                                TextFormat {
                                    color: ui.visuals().weak_text_color(),
                                    ..Default::default()
                                },
                            );
                        }

                        if ui
                            .selectable_label(
                                if let Some(selected_contact) = selected_contact {
                                    contact.email == *selected_contact
                                } else {
                                    false
                                },
                                contact_job,
                            )
                            .clicked()
                        {
                            *selected_contact = Some(contact.email.clone());
                        }
                    });
                }
            }
        });
}
