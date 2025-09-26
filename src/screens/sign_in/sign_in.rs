use crate::screens::sign_in::status_selector::{Status, status_selector};
use crate::svg;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};

pub struct SignIn {
    email: String,
    password: String,
    remember_me: bool,
    remember_my_password: bool,
    selected_status: Status,
    signing_in: bool,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
}

impl SignIn {
    pub fn new(main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>) -> Self {
        Self {
            email: String::default(),
            password: String::default(),
            remember_me: false,
            remember_my_password: false,
            selected_status: Status::Online,
            signing_in: false,
            main_window_sender,
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
                tui(ui, ui.id().with("sign in screen"))
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
                                    egui::Image::new(svg::default_display_picture())
                                        .fit_to_exact_size(egui::Vec2::splat(100.))
                                        .alt_text("Display picture"),
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

                        tui.ui(|ui| {
                            status_selector(ui, &mut self.selected_status);
                        });

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
                                width: if !self.signing_in {
                                    length(50.)
                                } else {
                                    auto()
                                },
                                height: auto(),
                            },
                            ..Default::default()
                        })
                        .ui(|ui| {
                            if !self.signing_in {
                                if ui.button("Sign In").clicked() {
                                    self.signing_in = true;
                                    let _ = self
                                        .main_window_sender
                                        .send(crate::main_window::Message::SignIn);
                                }
                            } else {
                                ui.spinner();
                            }
                        });
                    })
            });
    }
}
