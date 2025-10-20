use crate::helpers::run_future::run_future;
use crate::settings;
use crate::settings::Settings;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use msnp11_sdk::Client;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct PersonalSettings {
    display_name: Option<String>,
    server: String,
    nexus_url: String,
    check_for_updates: bool,
    client: Option<Arc<Client>>,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    handle: Handle,
}

impl PersonalSettings {
    pub fn new(
        display_name: Option<String>,
        client: Option<Arc<Client>>,
        main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
        handle: Handle,
    ) -> Self {
        let settings = settings::get_settings().unwrap_or_default();
        Self {
            display_name,
            server: settings.server,
            nexus_url: settings.nexus_url,
            check_for_updates: settings.check_for_updates,
            client,
            main_window_sender,
            handle,
        }
    }

    pub fn personal_settings(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.window_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                tui(ui, ui.id().with("personal-settings-screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: Some(taffy::AlignItems::Stretch),
                        size: taffy::Size {
                            width: percent(1.),
                            height: auto(),
                        },
                        padding: length(25.),
                        gap: length(15.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.ui(|ui| {
                            let label = ui.label("Display name:");
                            ui.add_space(3.);

                            if let Some(display_name) = &mut self.display_name {
                                ui.add_enabled(
                                    true,
                                    egui::text_edit::TextEdit::singleline(display_name)
                                        .hint_text("Display name")
                                        .min_size(egui::Vec2::new(340., 5.)),
                                )
                                .labelled_by(label.id)
                            } else {
                                let mut buffer = "";
                                ui.add_enabled(
                                    false,
                                    egui::text_edit::TextEdit::singleline(&mut buffer)
                                        .hint_text("Display name")
                                        .min_size(egui::Vec2::new(340., 5.)),
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
                                    .min_size(egui::Vec2::new(340., 5.)),
                            )
                            .labelled_by(label.id);
                        });

                        tui.ui(|ui| {
                            let label = ui.label("Nexus URL:");
                            ui.add_space(3.);
                            ui.add(
                                egui::text_edit::TextEdit::singleline(&mut self.nexus_url)
                                    .hint_text("Nexus URL")
                                    .min_size(egui::Vec2::new(340., 5.)),
                            )
                            .labelled_by(label.id);
                        });

                        tui.ui(|ui| {
                            ui.checkbox(
                                &mut self.check_for_updates,
                                "Check for updates on startup",
                            );
                        });

                        tui.style(taffy::Style {
                            align_self: Some(taffy::AlignItems::Center),
                            size: taffy::Size {
                                width: percent(0.15),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
                            ui.style_mut().spacing.button_padding = egui::Vec2::new(8., 5.);
                            if ui.button("Save").clicked() {
                                self.display_name
                                    .as_mut()
                                    .map(|display_name| display_name.trim().to_string());

                                self.server = self.server.trim().to_string();
                                self.nexus_url = self.nexus_url.trim().to_string();

                                let settings = Settings {
                                    server: self.server.clone(),
                                    nexus_url: self.nexus_url.clone(),
                                    check_for_updates: self.check_for_updates,
                                };

                                let _ = settings::save_settings(&settings);
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);

                                if let Some(display_name) = self.display_name.clone()
                                    && let Some(client) = self.client.clone()
                                {
                                    let new_display_name = display_name.clone();
                                    run_future(
                                        self.handle.clone(),
                                        async move { client.set_display_name(&display_name).await },
                                        self.main_window_sender.clone(),
                                        move |result| {
                                            crate::main_window::Message::DisplayNameChangeResult(
                                                new_display_name.clone(),
                                                result,
                                            )
                                        },
                                    );
                                }
                            }
                        });

                        tui.style(taffy::Style {
                            align_self: Some(taffy::AlignItems::Center),
                            size: taffy::Size {
                                width: percent(0.42),
                                height: auto(),
                            },
                            padding: percent(0.07),
                            ..Default::default()
                        })
                        .label(format!("meowsn v{}", env!("CARGO_PKG_VERSION")));
                    });
            });
    }
}
