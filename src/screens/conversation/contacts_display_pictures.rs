use crate::models::contact::Contact;
use crate::svg;
use eframe::egui;
use egui_taffy::taffy::prelude::{auto, fr, length, percent, repeat, span};
use egui_taffy::{Tui, TuiBuilderLogic, taffy};
use std::collections::HashMap;
use std::sync::Arc;

pub fn contacts_display_pictures(
    tui: &mut Tui,
    participants: &HashMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
) {
    if participants.len() < 2 {
        tui.style(taffy::Style {
            justify_self: Some(taffy::JustifySelf::End),
            grid_row: span(2),
            ..Default::default()
        })
        .add_with_border(|tui| {
            tui.ui(|ui| {
                if let Some(participant) = participants.values().next() {
                    ui.add(if let Some(picture) = participant.display_picture.clone() {
                        egui::Image::from_bytes(
                            format!("bytes://{}.png", picture.hash),
                            picture.data,
                        )
                        .fit_to_exact_size(egui::Vec2::splat(90.))
                        .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
                        .alt_text("Contact display picture")
                    } else {
                        egui::Image::new(svg::default_display_picture())
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .alt_text("Default display picture")
                    })
                } else if let Some(participant) = &last_participant {
                    ui.add(if let Some(picture) = participant.display_picture.clone() {
                        egui::Image::from_bytes(
                            format!("bytes://{}.png", picture.hash),
                            picture.data,
                        )
                        .fit_to_exact_size(egui::Vec2::splat(90.))
                        .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
                        .alt_text("Contact display picture")
                    } else {
                        egui::Image::new(svg::default_display_picture())
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .alt_text("Default display picture")
                    })
                } else {
                    ui.add(
                        egui::Image::new(svg::default_display_picture())
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .alt_text("Default display picture"),
                    )
                }
            })
        });
    } else {
        tui.style(taffy::Style {
            justify_self: Some(taffy::JustifySelf::Start),
            grid_row: span(3),
            display: taffy::Display::Grid,
            grid_template_columns: vec![fr(1.), fr(1.)],
            grid_template_rows: vec![repeat("auto-fill", vec![length(43.)])],
            align_items: Some(taffy::AlignItems::Center),
            gap: length(5.),
            ..Default::default()
        })
        .add(|tui| {
            for participant in participants.values() {
                tui.style(taffy::Style {
                    justify_self: Some(taffy::JustifySelf::Start),
                    size: taffy::Size {
                        width: length(45.),
                        height: auto(),
                    },
                    margin: percent(-0.9),
                    ..Default::default()
                })
                .add_with_border(|tui| {
                    tui.ui(|ui| {
                        ui.add(if let Some(picture) = participant.display_picture.clone() {
                            egui::Image::from_bytes(
                                format!("bytes://{}.png", picture.hash),
                                picture.data,
                            )
                            .fit_to_exact_size(egui::Vec2::splat(44.))
                            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
                            .alt_text("Contact display picture")
                        } else {
                            egui::Image::new(svg::default_display_picture())
                                .fit_to_exact_size(egui::Vec2::splat(44.))
                                .alt_text("Default display picture")
                        })
                    });
                });

                tui.style(taffy::Style {
                    margin: percent(-0.9),
                    max_size: percent(1.),
                    ..Default::default()
                })
                .ui_add(egui::Label::new(participant.display_name.as_str()).truncate());
            }
        })
    }
}
