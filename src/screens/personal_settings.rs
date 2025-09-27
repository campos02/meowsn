use crate::settings;
use crate::settings::Settings;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};

pub struct PersonalSettings {
    display_name: Option<String>,
    server: String,
    nexus_url: String,
    check_for_updates: bool,
}

impl PersonalSettings {
    pub fn new(display_name: Option<String>) -> Self {
        let settings = settings::get_settings().unwrap_or_default();
        Self {
            display_name,
            server: settings.server,
            nexus_url: settings.nexus_url,
            check_for_updates: settings.check_for_updates,
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
                        padding: length(30.),
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
                                width: length(35.),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
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
                            }
                        })
                    });
            });
    }
}
