use crate::helpers::run_future::run_future;
use crate::helpers::sign_in_async::sign_in_async;
use crate::models::display_picture::DisplayPicture;
use crate::models::sign_in_return::SignInReturn;
use crate::screens::sign_in::status_selector::{Status, status_selector};
use crate::sqlite::Sqlite;
use crate::svg;
use crate::widgets::custom_fill_combo_box::CustomFillComboBox;
use eframe::egui;
use eframe::egui::{FontFamily, FontId};
use egui_taffy::taffy::prelude::{auto, length, percent};
use egui_taffy::{TuiBuilderLogic, taffy, tui};
use keyring::Entry;
use msnp11_sdk::{MsnpStatus, SdkError};
use std::sync::Arc;
use tokio::runtime::Handle;

pub enum Message {
    SignInResult(Result<SignInReturn, SdkError>),
}

pub struct SignIn {
    display_picture: Option<DisplayPicture>,
    emails: Vec<String>,
    email: String,
    password: String,
    remember_me: bool,
    remember_my_password: bool,
    selected_status: Status,
    signing_in: bool,
    main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    handle: Handle,
    sqlite: Sqlite,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
}

impl SignIn {
    pub fn new(
        handle: Handle,
        sqlite: Sqlite,
        main_window_sender: std::sync::mpsc::Sender<crate::main_window::Message>,
    ) -> Self {
        let mut display_picture = None;
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

            if let Ok(user) = sqlite.select_user(&email)
                && let Some(picture) = user.display_picture
            {
                display_picture = Some(picture)
            }
        }

        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            display_picture,
            emails,
            email,
            password,
            remember_me,
            remember_my_password,
            selected_status: Status::Online,
            signing_in: false,
            main_window_sender,
            handle,
            sqlite,
            sender,
            receiver,
        }
    }
}

impl eframe::App for SignIn {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Ok(message) = self.receiver.try_recv() {
            let Message::SignInResult(result) = message;
            match result {
                Ok(sign_in_return) => {
                    if self.remember_me {
                        let _ = self.sqlite.insert_user_if_not_in_db(&self.email);
                    }

                    if self.remember_my_password
                        && let Ok(entry) = Entry::new("meowsn", &self.email)
                    {
                        let _ = entry.set_password(&self.password);
                    }

                    let _ = self
                        .main_window_sender
                        .send(crate::main_window::Message::SignIn(sign_in_return));

                    ctx.request_repaint();
                }

                Err(error) => {
                    let _ = self
                        .main_window_sender
                        .send(crate::main_window::Message::OpenDialog(error.to_string()));

                    self.signing_in = false;
                    ctx.request_repaint();
                }
            }
        }

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
                        gap: length(15.),
                        ..Default::default()
                    })
                    .show(|tui| {
                        tui.add_with_border(|tui| {
                            tui.ui(|ui| {
                                ui.add(if let Some(picture) = self.display_picture.clone() {
                                    egui::Image::from_bytes(
                                        format!("bytes://{}.png", picture.hash),
                                        picture.data,
                                    )
                                    .fit_to_exact_size(egui::Vec2::splat(100.))
                                    .corner_radius(
                                        ui.visuals().widgets.noninteractive.corner_radius,
                                    )
                                    .alt_text("User display picture")
                                } else {
                                    egui::Image::new(svg::default_display_picture())
                                        .fit_to_exact_size(egui::Vec2::splat(100.))
                                        .alt_text("Default display picture")
                                })
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
                            ui.add_enabled_ui(!self.signing_in, |ui| {
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

                                                    if let Ok(user) = self.sqlite.select_user(email)
                                                        && let Some(picture) = user.display_picture
                                                    {
                                                        self.display_picture = Some(picture);
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
                                                self.display_picture = None;
                                                self.email.clear();
                                                self.password.clear();

                                                self.remember_me = false;
                                                self.remember_my_password = false;
                                            };
                                        });
                                });
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
                            ui.add_enabled_ui(!self.signing_in, |ui| {
                                let label = ui.label("Password:");
                                ui.add_space(3.);
                                ui.add(
                                    egui::text_edit::TextEdit::singleline(&mut self.password)
                                        .hint_text("Password")
                                        .password(true),
                                )
                                .labelled_by(label.id);
                            })
                        });

                        tui.ui(|ui| {
                            ui.add_enabled_ui(!self.signing_in, |ui| {
                                status_selector(
                                    ui,
                                    &mut self.selected_status,
                                    self.main_window_sender.clone(),
                                );
                            })
                        });

                        tui.ui(|ui| {
                            ui.add_enabled_ui(!self.signing_in, |ui| {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut self.remember_me, "Remember Me");
                                    ui.scope(|ui| {
                                        ui.style_mut().text_styles.insert(
                                            egui::TextStyle::Body,
                                            FontId::new(12.8, FontFamily::Proportional),
                                        );

                                        if ui.link("(Forget Me)").clicked() {
                                            let _ = self.sqlite.delete_user(&self.email);
                                            if let Ok(entry) = Entry::new("meowsn", &self.email) {
                                                let _ = entry.delete_credential();
                                            }

                                            self.emails = self
                                                .sqlite
                                                .select_user_emails()
                                                .unwrap_or_default();

                                            self.display_picture = None;
                                            self.email.clear();
                                            self.password.clear();

                                            self.remember_me = false;
                                            self.remember_my_password = false;
                                        }
                                    })
                                });

                                if ui
                                    .checkbox(
                                        &mut self.remember_my_password,
                                        "Remember My Password",
                                    )
                                    .changed()
                                    && self.remember_my_password
                                {
                                    self.remember_me = true;
                                }
                            })
                        });

                        tui.style(taffy::Style {
                            size: taffy::Size {
                                width: if !self.signing_in {
                                    percent(0.2)
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
                                    if self.email.is_empty() || self.password.is_empty() {
                                        let _ = self.main_window_sender.send(
                                            crate::main_window::Message::OpenDialog(
                                                "Please type your e-mail address and \
                                            password in their corresponding forms."
                                                    .to_string(),
                                            ),
                                        );

                                        ctx.request_repaint();
                                    } else {
                                        self.signing_in = true;

                                        let email = Arc::new(self.email.trim().to_string());
                                        let password = Arc::new(self.password.clone());
                                        let sqlite = self.sqlite.clone();

                                        let status = match self.selected_status {
                                            Status::Busy => MsnpStatus::Busy,
                                            Status::Away => MsnpStatus::Away,
                                            Status::AppearOffline => MsnpStatus::AppearOffline,
                                            _ => MsnpStatus::Online,
                                        };

                                        run_future(
                                            self.handle.clone(),
                                            async move {
                                                sign_in_async(email, password, status, sqlite).await
                                            },
                                            self.sender.clone(),
                                            Message::SignInResult,
                                        );
                                    }
                                }
                            } else {
                                ui.spinner();
                            }
                        });
                    })
            });
    }
}
