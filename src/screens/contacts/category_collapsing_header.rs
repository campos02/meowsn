use crate::models::contact::Contact;
use crate::svg;
use eframe::egui;
use eframe::egui::Ui;
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
                                        alt_text = "Contact is offline";
                                        svg::default_display_picture()
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

                        if ui
                            .selectable_label(
                                if let Some(selected_contact) = selected_contact {
                                    contact.email == *selected_contact
                                } else {
                                    false
                                },
                                contact.display_name.as_str(),
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
