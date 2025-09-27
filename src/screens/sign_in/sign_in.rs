use crate::screens::sign_in::status_selector::{Status, status_selector};
use crate::sqlite::Sqlite;
use crate::svg;
use crate::widgets::custom_fill_combo_box::CustomFillComboBox;
use eframe::egui;
use eframe::egui::{FontFamily, FontId};
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use keyring::Entry;

pub struct SignIn {
    emails: Vec<String>,
    email: String,
    password: String,
    remember_me: bool,
    remember_my_password: bool,
    selected_status: Status,
    signing_in: bool,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    sqlite: Sqlite,
}

impl SignIn {
    pub fn new(
        sqlite: Sqlite,
        main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    ) -> Self {
        let mut email = String::default();
        let mut password = String::default();
        let mut remember_me = false;
        let mut remember_my_password = false;

        let emails = sqlite.select_user_emails().unwrap_or_default();
        if let Some(first_email) = emails.first() {
            email = first_email.to_owned();
            remember_me = true;

            if let Ok(entry) = Entry::new("meowsn", first_email)
                && let Ok(passwd) = entry.get_password()
            {
                password = passwd;
                remember_my_password = true;
            }
        }

        Self {
            emails,
            email,
            password,
            remember_me,
            remember_my_password,
            selected_status: Status::Online,
            signing_in: false,
            main_window_sender,
            sqlite,
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
                tui(ui, ui.id().with("sign-in-screen"))
                    .reserve_available_space()
                    .style(taffy::Style {
                        flex_direction: taffy::FlexDirection::Column,
                        align_items: Some(taffy::AlignItems::Center),
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
                            ui.horizontal(|ui| {
                                ui.style_mut().spacing.item_spacing.x = 1.;
                                ui.style_mut().spacing.button_padding = egui::Vec2::splat(2.);

                                ui.add(
                                    egui::text_edit::TextEdit::singleline(&mut self.email)
                                        .hint_text("E-mail address")
                                        .min_size(egui::vec2(227., 5.))
                                        .desired_width(219.),
                                )
                                .labelled_by(label.id);

                                CustomFillComboBox::from_label("")
                                    .selected_text("")
                                    .width(3.)
                                    .fill_color(ui.visuals().text_edit_bg_color())
                                    .show_ui(ui, |ui| {
                                        for email in &self.emails {
                                            if ui
                                                .selectable_value(
                                                    &mut self.email,
                                                    email.clone(),
                                                    email,
                                                )
                                                .clicked()
                                            {
                                                self.remember_me = true;

                                                if let Ok(entry) = Entry::new("meowsn", email)
                                                    && let Ok(passwd) = entry.get_password()
                                                {
                                                    self.password = passwd;
                                                    self.remember_my_password = true;
                                                }
                                            }
                                        }

                                        if ui
                                            .selectable_value(
                                                &mut self.email,
                                                "".to_string(),
                                                "Sign in with a different e-mail address",
                                            )
                                            .clicked()
                                        {
                                            self.email.clear();
                                            self.password.clear();

                                            self.remember_me = false;
                                            self.remember_my_password = false;
                                        };
                                    });
                            });
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
                            status_selector(
                                ui,
                                &mut self.selected_status,
                                self.main_window_sender.clone(),
                            );
                        });

                        tui.ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.remember_me, "Remember Me");
                                ui.scope(|ui| {
                                    ui.style_mut().text_styles.insert(
                                        egui::TextStyle::Body,
                                        FontId::new(12., FontFamily::Proportional),
                                    );

                                    if ui.link("(Forget Me)").clicked() {
                                        let _ = self.sqlite.delete_user(&self.email);
                                        if let Ok(entry) = Entry::new("meowsn", &self.email) {
                                            let _ = entry.delete_credential();
                                        }

                                        self.emails =
                                            self.sqlite.select_user_emails().unwrap_or_default();

                                        self.email.clear();
                                        self.password.clear();

                                        self.remember_me = false;
                                        self.remember_my_password = false;
                                    }
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
