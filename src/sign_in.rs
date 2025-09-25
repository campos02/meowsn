use crate::widgets::left_label_combo_box::LeftLabelComboBox;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};

#[derive(Debug, PartialEq, Copy, Clone)]
enum Status {
    Online,
    Busy,
    Away,
    AppearOffline,
    PersonalSettings,
}

pub struct SignIn {
    email: String,
    password: String,
    remember_me: bool,
    remember_my_password: bool,
    selected_status: Status,
    signing_in: bool,
}

impl Default for SignIn {
    fn default() -> Self {
        Self {
            email: String::default(),
            password: String::default(),
            remember_me: false,
            remember_my_password: false,
            selected_status: Status::Online,
            signing_in: false,
        }
    }
}

impl eframe::App for SignIn {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame {
                    fill: ctx.style().visuals.window_fill,
                    ..Default::default()
                }
                .inner_margin(30.),
            )
            .show(ctx, |ui| {
                tui(ui, ui.id().with("meowsn"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: Some(taffy::AlignItems::Center),
                        justify_content: Some(taffy::AlignContent::Center),
                        size: taffy::Size {
                            width: percent(1.),
                            height: auto(),
                        },
                        padding: length(8.),
                        gap: length(20.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.add_with_border(|tui| {
                            tui.ui(|ui| {
                                ui.add(
                                    egui::Image::new(egui::include_image!(
                                        "../assets/default_display_picture.svg"
                                    ))
                                    .fit_to_exact_size(egui::Vec2::splat(105.)),
                                )
                            })
                        });

                        tui.style(taffy::Style {
                            size: taffy::Size {
                                width: length(250.),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
                            let label = ui.label("E-mail address:");
                            ui.add_space(3.);
                            ui.add(
                                egui::text_edit::TextEdit::singleline(&mut self.email)
                                    .hint_text("E-mail address"),
                            )
                            .labelled_by(label.id);
                        });

                        tui.style(taffy::Style {
                            size: taffy::Size {
                                width: length(250.),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
                            let label = ui.label("Password:");
                            ui.add_space(3.);
                            ui.add(
                                egui::text_edit::TextEdit::singleline(&mut self.password)
                                    .hint_text("Password")
                                    .password(true),
                            )
                            .labelled_by(label.id);
                        });

                        let old_status = self.selected_status;
                        tui.ui(|ui| {
                            LeftLabelComboBox::from_label("Status:")
                                .selected_text(format!("{:?}", self.selected_status))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.selected_status,
                                        Status::Online,
                                        "Online",
                                    );
                                    ui.selectable_value(
                                        &mut self.selected_status,
                                        Status::Busy,
                                        "Busy",
                                    );
                                    ui.selectable_value(
                                        &mut self.selected_status,
                                        Status::Away,
                                        "Away",
                                    );
                                    ui.selectable_value(
                                        &mut self.selected_status,
                                        Status::AppearOffline,
                                        "Appear Offline",
                                    );
                                    ui.separator();
                                    ui.selectable_value(
                                        &mut self.selected_status,
                                        Status::PersonalSettings,
                                        "Personal Settings",
                                    );
                                });
                        });

                        if self.selected_status == Status::PersonalSettings {
                            self.selected_status = old_status;
                        }

                        tui.ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.remember_me, "Remember Me");
                                ui.scope(|ui| {
                                    ui.style_mut().text_styles.insert(
                                        egui::TextStyle::Body,
                                        egui::FontId::new(12., egui::FontFamily::Proportional),
                                    );
                                    ui.link("(Forget Me)")
                                })
                            });
                            ui.checkbox(&mut self.remember_my_password, "Remember My Password");
                        });

                        tui.style(taffy::Style {
                            size: taffy::Size {
                                width: length(50.),
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
                            if ui
                                .add_enabled(!self.signing_in, egui::Button::new("Sign In"))
                                .clicked()
                            {
                                self.signing_in = true;
                            }
                        });
                    })
            });
    }
}
