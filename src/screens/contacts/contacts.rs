use crate::models::contact::Contact;
use crate::screens::contacts;
use crate::screens::contacts::category_collapsing_header::category_collapsing_header;
use crate::screens::contacts::status_selector::status_selector;
use crate::svg;
use eframe::egui;
use egui_taffy::taffy::prelude::length;
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Contacts {
    display_name: Arc<String>,
    personal_message: String,
    selected_status: contacts::status_selector::Status,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    show_personal_message_frame: bool,
    online_contacts: HashMap<Arc<String>, Contact>,
    offline_contacts: HashMap<Arc<String>, Contact>,
    selected_contact: Option<Arc<String>>,
}

impl Contacts {
    pub fn new(main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>) -> Self {
        Self {
            display_name: Arc::new(String::from("Testing 2")),
            personal_message: String::new(),
            main_window_sender,
            selected_status: contacts::status_selector::Status::Online,
            show_personal_message_frame: false,
            online_contacts: HashMap::new(),
            offline_contacts: HashMap::new(),
            selected_contact: None,
        }
    }
}

impl eframe::App for Contacts {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                tui(ui, ui.id().with("contacts screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        padding: length(15.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.style(taffy::Style {
                            flex_direction: taffy::FlexDirection::Row,
                            gap: length(10.),
                            ..Default::default()
                        })
                        .add(|tui| {
                            tui.add_with_border(|tui| {
                                tui.ui(|ui| {
                                    ui.add(
                                        egui::Image::new(svg::default_display_picture())
                                            .fit_to_exact_size(egui::Vec2::splat(60.))
                                            .alt_text("Display picture"),
                                    )
                                })
                            });

                            tui.ui(|ui| {
                                ui.vertical(|ui| {
                                    ui.add_space(5.);
                                    status_selector(
                                        ui,
                                        self.display_name.as_str(),
                                        &mut self.selected_status,
                                        self.main_window_sender.clone(),
                                    );

                                    let personal_message = ui.add(
                                        egui::text_edit::TextEdit::singleline(
                                            &mut self.personal_message,
                                        )
                                        .hint_text("<Type a personal message>")
                                        .min_size(egui::vec2(180., 5.))
                                        .frame(self.show_personal_message_frame),
                                    );

                                    self.show_personal_message_frame =
                                        personal_message.hovered() || personal_message.has_focus();
                                });
                            })
                        });

                        tui.ui(|ui| ui.add_space(8.));
                        tui.ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::Image::new(svg::add_contact())
                                        .fit_to_exact_size(egui::Vec2::splat(20.))
                                        .alt_text("Add a contact"),
                                );
                                ui.link("Add a Contact")
                            })
                        });

                        tui.style(taffy::Style {
                            padding: length(10.),
                            ..Default::default()
                        })
                        .ui(|ui| {
                            category_collapsing_header(
                                ui,
                                "Online",
                                &mut self.selected_contact,
                                &self.online_contacts,
                            );

                            category_collapsing_header(
                                ui,
                                "Offline",
                                &mut self.selected_contact,
                                &self.offline_contacts,
                            );
                        });
                    })
            });
    }
}
