use crate::contact_repository::ContactRepository;
use crate::helpers::run_future::run_future;
use crate::screens::contacts::contacts;
use crate::settings::Settings;
use crate::{main_window, settings};
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::{Client, MsnpList};
use std::sync::{Arc, mpsc};
use tokio::runtime::Handle;

#[derive(PartialEq)]
enum SelectedTab {
    General,
    Privacy,
}

pub struct PersonalSettings {
    display_name: Option<String>,
    server: String,
    nexus_url: String,
    config_server: String,
    check_for_updates: bool,
    notify_sign_ins: bool,
    notify_added_by: bool,
    only_in_contact_list: bool,
    client: Option<Arc<Client>>,
    main_window_sender: mpsc::Sender<main_window::Message>,
    contacts_sender: Option<mpsc::Sender<contacts::Message>>,
    handle: Handle,
    selected_tab: SelectedTab,
    contact_repository: Option<ContactRepository>,
    selected_contact: Option<Arc<String>>,
}

impl PersonalSettings {
    pub fn new(
        display_name: Option<String>,
        client: Option<Arc<Client>>,
        contact_repository: Option<ContactRepository>,
        main_window_sender: mpsc::Sender<main_window::Message>,
        contacts_sender: Option<mpsc::Sender<contacts::Message>>,
        blp_bl: Option<bool>,
        handle: Handle,
    ) -> Self {
        let settings = settings::get_settings().unwrap_or_default();
        Self {
            display_name,
            server: settings.server,
            nexus_url: settings.nexus_url,
            config_server: settings.config_server,
            check_for_updates: settings.check_for_updates,
            notify_sign_ins: settings.notify_sign_ins,
            notify_added_by: settings.notify_added_by,
            only_in_contact_list: blp_bl.unwrap_or_default(),
            client,
            main_window_sender,
            contacts_sender,
            handle,
            selected_tab: SelectedTab::General,
            contact_repository,
            selected_contact: None,
        }
    }

    pub fn personal_settings(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("tabs")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    let label = ui.selectable_label(
                        self.selected_tab == SelectedTab::General,
                        "General     ",
                    );

                    if label.clicked() || label.secondary_clicked() {
                        self.selected_tab = SelectedTab::General;
                    }

                    let label = ui.selectable_label(
                        self.selected_tab == SelectedTab::Privacy,
                        "Privacy      ",
                    );

                    if label.clicked() || label.secondary_clicked() {
                        self.selected_tab = SelectedTab::Privacy;
                    }
                })
            });

        egui::Panel::bottom("version")
            .frame(egui::Frame {
                inner_margin: egui::Margin {
                    top: 15,
                    bottom: 15,
                    left: 15,
                    right: 15,
                },
                fill: ui.visuals().window_fill,
                ..Default::default()
            })
            .show_separator_line(false)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(format!("meowsn v{}", env!("CARGO_PKG_VERSION")));
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            tui(ui, ui.id().with("personal-settings-screen"))
                .reserve_available_space()
                .style(taffy::Style {
                    flex_direction: taffy::FlexDirection::Column,
                    align_items: Some(taffy::AlignItems::Stretch),
                    size: taffy::Size {
                        width: percent(1.),
                        height: auto(),
                    },
                    padding: length(15.),
                    gap: length(15.),
                    ..Default::default()
                })
                .show(|tui| {
                    match self.selected_tab {
                        SelectedTab::General => {
                            tui.ui(|ui| {
                                let label = ui.label("Display name:");
                                ui.add_space(3.);

                                if let Some(display_name) = &mut self.display_name {
                                    ui.add_enabled(
                                        true,
                                        egui::text_edit::TextEdit::singleline(display_name)
                                            .hint_text("Display name")
                                            .min_size(egui::Vec2::new(ui.available_width(), 5.)),
                                    )
                                    .labelled_by(label.id)
                                    .on_hover_text("Change your display name")
                                } else {
                                    let mut buffer = "";
                                    ui.add_enabled(
                                        false,
                                        egui::text_edit::TextEdit::singleline(&mut buffer)
                                            .hint_text("Display name")
                                            .min_size(egui::Vec2::new(ui.available_width(), 5.)),
                                    )
                                    .labelled_by(label.id)
                                }
                            });

                            tui.ui(|ui| {
                                let label = ui.label("Server:");
                                ui.add_space(3.);
                                ui.add(
                                    egui::text_edit::TextEdit::singleline(&mut self.server)
                                        .hint_text("Server")
                                        .min_size(egui::Vec2::new(ui.available_width(), 5.)),
                                )
                                .labelled_by(label.id)
                                .on_hover_text("Enter the main server address");
                            });

                            tui.ui(|ui| {
                                let label = ui.label("Nexus URL:");
                                ui.add_space(3.);
                                ui.add(
                                    egui::text_edit::TextEdit::singleline(&mut self.nexus_url)
                                        .hint_text("Nexus URL")
                                        .min_size(egui::Vec2::new(ui.available_width(), 5.)),
                                )
                                .labelled_by(label.id)
                                .on_hover_text("Enter the Nexus URL used in authentication");
                            });

                            tui.ui(|ui| {
                                let label = ui.label("Configuration server URL:");
                                ui.add_space(3.);
                                ui.add(
                                    egui::text_edit::TextEdit::singleline(&mut self.config_server)
                                        .hint_text("Configuration server URL")
                                        .min_size(egui::Vec2::new(ui.available_width(), 5.)),
                                )
                                .labelled_by(label.id)
                                .on_hover_text("Enter the configuration server URL (used to get tabs)");
                            });

                            tui.ui(|ui| {
                                ui.checkbox(&mut self.check_for_updates, "Check for updates on startup");
                                ui.checkbox(
                                    &mut self.notify_sign_ins,
                                    "Notify me when contacts come online",
                                );
                            });

                            tui.style(taffy::Style {
                                align_self: Some(taffy::AlignItems::Center),
                                ..Default::default()
                            })
                            .ui(|ui| {
                                ui.style_mut().spacing.button_padding = egui::Vec2::new(8., 5.);
                                ui.horizontal(|ui| {
                                    if ui.button("Save").on_hover_text("Save settings").clicked() {
                                        self.display_name
                                            .as_mut()
                                            .map(|display_name| display_name.trim().to_string());

                                        self.server = self.server.trim().to_string();
                                        self.nexus_url = self.nexus_url.trim().to_string();

                                        let settings = Settings {
                                            server: self.server.clone(),
                                            nexus_url: self.nexus_url.clone(),
                                            config_server: self.config_server.clone(),
                                            check_for_updates: self.check_for_updates,
                                            notify_sign_ins: self.notify_sign_ins,
                                            notify_added_by: self.notify_added_by,
                                        };

                                        let _ = settings::save_settings(&settings);
                                        ui.send_viewport_cmd(egui::ViewportCommand::Close);

                                        if let Some(display_name) = self.display_name.clone()
                                            && let Some(client) = self.client.clone()
                                        {
                                            let new_display_name = display_name.clone();
                                            run_future(
                                                self.handle.clone(),
                                                async move { client.set_display_name(&display_name).await },
                                                self.main_window_sender.clone(),
                                                move |result| {
                                                    main_window::Message::DisplayNameChangeResult(
                                                        new_display_name.clone(),
                                                        result,
                                                    )
                                                },
                                            );
                                        }
                                    }

                                    if ui
                                        .button("Restore Defaults")
                                        .on_hover_text("Restore default settings")
                                        .clicked()
                                    {
                                        let defaults = Settings::default();
                                        self.server = defaults.server;
                                        self.nexus_url = defaults.nexus_url;
                                        self.config_server = defaults.config_server;
                                        self.check_for_updates = defaults.check_for_updates;
                                        self.notify_sign_ins = defaults.notify_sign_ins;
                                    }
                                });
                            });
                        }

                        SelectedTab::Privacy => {
                            tui.label("Allow and block lists");
                            tui.ui(|ui| {
                                ui.checkbox(
                                    &mut self.only_in_contact_list,
                                    "Only people in my Allow List can see my status and send me messages",
                                );
                            });

                            if let Some(contact_repository) = &self.contact_repository {
                                tui.ui(|ui| {
                                    egui::Grid::new("privacy_grid")
                                        .max_col_width(ui.available_width() / 2.41)
                                        .show(ui, |ui| {
                                        let allowed_contacts = contact_repository
                                            .get_contacts_in_list(MsnpList::AllowList);

                                        let blocked_contacts = contact_repository
                                            .get_contacts_in_list(MsnpList::BlockList);

                                        ui.vertical(|ui| {
                                            ui.label("Allow list:");
                                            ui.add_space(3.);
                                            ui.push_id(0, |ui| {
                                                egui::Frame::new()
                                                    .fill(ui.visuals().text_edit_bg_color())
                                                    .show(ui, |ui| {
                                                    egui::ScrollArea::vertical()
                                                        .min_scrolled_height(120.)
                                                        .auto_shrink(false)
                                                        .show(ui, |ui| {
                                                            ui.vertical(|ui| {
                                                                if let Some(contacts) = &allowed_contacts {
                                                                    for contact in contacts {
                                                                        let label = ui.add(egui::Button::selectable(
                                                                            self.selected_contact
                                                                                .as_ref()
                                                                                .is_some_and(|selected_contact| {
                                                                                    *selected_contact == contact.email
                                                                                }),
                                                                            &*contact.email
                                                                        )
                                                                        .truncate())
                                                                        .on_hover_text(
                                                                            &*contact.email
                                                                        );

                                                                        if label.clicked() || label.secondary_clicked() {
                                                                            self.selected_contact = Some(contact.email.clone());
                                                                        }
                                                                    }
                                                                }
                                                            });
                                                        });
                                                });
                                            });
                                        });

                                        ui.vertical(|ui| {
                                            ui.add_space(24.);
                                            if ui.add_enabled(blocked_contacts.as_ref().is_some_and(|contacts| {
                                                !contacts.is_empty()
                                            }), egui::Button::new("<< Allow")).clicked()
                                                && let Some(contact) = self.selected_contact.clone()
                                                && let Some(client) = self.client.clone()
                                                && let Some(contacts_sender) = self.contacts_sender.clone() {
                                                let email = contact.clone();
                                                run_future(
                                                    self.handle.clone(),
                                                    async move { client.unblock_contact(&email).await },
                                                    contacts_sender,
                                                    move |result| contacts::Message::UnblockResult(
                                                        contact.clone(), result
                                                    ),
                                                );
                                            }

                                            if ui.add_enabled(allowed_contacts.is_some_and(|contacts| {
                                                !contacts.is_empty()
                                            }), egui::Button::new(">> Block")).clicked()
                                                && let Some(contact) = self.selected_contact.clone()
                                                && let Some(client) = self.client.clone()
                                                && let Some(contacts_sender) = self.contacts_sender.clone() {
                                                let email = contact.clone();
                                                run_future(
                                                    self.handle.clone(),
                                                    async move { client.block_contact(&email).await },
                                                    contacts_sender,
                                                    move |result| contacts::Message::BlockResult(contact.clone(), result),
                                                );
                                            }
                                        });

                                        ui.vertical(|ui| {
                                            ui.label("Block list:");
                                            ui.add_space(3.);
                                            ui.push_id(2, |ui| {
                                                egui::Frame::new()
                                                    .fill(ui.visuals().text_edit_bg_color())
                                                    .show(ui, |ui| {
                                                    egui::ScrollArea::vertical()
                                                        .min_scrolled_height(120.)
                                                        .auto_shrink(false)
                                                        .show(ui, |ui| {
                                                            ui.vertical(|ui| {
                                                                if let Some(contacts) = blocked_contacts {
                                                                    for contact in contacts {
                                                                        let label = ui.add(egui::Button::selectable(
                                                                            self.selected_contact
                                                                                .as_ref()
                                                                                .is_some_and(|selected_contact| {
                                                                                    *selected_contact == contact.email
                                                                                }),
                                                                            &*contact.email
                                                                        )
                                                                        .truncate())
                                                                        .on_hover_text(
                                                                            &*contact.email
                                                                        );

                                                                        if label.clicked() || label.secondary_clicked() {
                                                                            self.selected_contact = Some(contact.email.clone());
                                                                        }
                                                                    }
                                                                }
                                                            });
                                                        });
                                                });
                                            });
                                        });
                                    });
                                });
                            }

                            if let Some(contact_repository) = &self.contact_repository
                                && let Some(contacts) = contact_repository.get_contacts_in_list(MsnpList::ReverseList) {
                                tui.ui(|ui| {
                                    ui.label("The following people have added you to their contact list:");
                                    ui.add_space(3.);

                                    egui::Frame::new()
                                        .fill(ui.visuals().text_edit_bg_color())
                                        .inner_margin(5.)
                                        .show(ui, |ui| {
                                        egui::ScrollArea::vertical()
                                            .min_scrolled_height(90.)
                                            .auto_shrink(false)
                                            .show(ui, |ui| {
                                                for contact in contacts {
                                                    ui.label(format!("{}", contact.email));
                                                }
                                            });
                                    });
                                });
                            }

                            tui.ui(|ui| {
                                ui.checkbox(
                                    &mut self.notify_added_by,
                                    "Notify me when other people add me to their contact list",
                                );
                            });

                            tui.style(taffy::Style {
                                align_self: Some(taffy::AlignItems::Center),
                                ..Default::default()
                            })
                                .ui(|ui| {
                                    ui.style_mut().spacing.button_padding = egui::Vec2::new(8., 5.);
                                    ui.horizontal(|ui| {
                                        if ui.button("Save").on_hover_text("Save settings").clicked() {
                                            let settings = Settings {
                                                server: self.server.clone(),
                                                nexus_url: self.nexus_url.clone(),
                                                config_server: self.config_server.clone(),
                                                check_for_updates: self.check_for_updates,
                                                notify_sign_ins: self.notify_sign_ins,
                                                notify_added_by: self.notify_added_by,
                                            };

                                            let _ = settings::save_settings(&settings);
                                            ui.send_viewport_cmd(egui::ViewportCommand::Close);

                                            if let Some(client) = self.client.clone()
                                                && let Some(sender) = self.contacts_sender.clone() {
                                                let only_in_contact_list = self.only_in_contact_list;
                                                run_future(
                                                    self.handle.clone(),
                                                    async move { client.set_blp(if only_in_contact_list {
                                                        "BL"
                                                    } else {
                                                        "AL"
                                                    }).await },
                                                    sender,
                                                    move |result| contacts::Message::BlpResult {
                                                        blp_bl: only_in_contact_list, result
                                                    },
                                                );
                                            }
                                        }

                                        if ui
                                            .button("Restore Defaults")
                                            .on_hover_text("Restore default settings")
                                            .clicked()
                                        {
                                            let defaults = Settings::default();
                                            self.notify_added_by = defaults.notify_added_by;
                                        }
                                    });
                                });
                        }
                    }
                });
        });
    }
}
