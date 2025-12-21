use crate::models::contact::Contact;
use crate::svg;
use eframe::egui;
use egui_taffy::taffy::prelude::{fr, length, line, repeat};
use egui_taffy::{Tui, TuiBuilderLogic, taffy};
use std::collections::BTreeMap;
use std::sync::Arc;

pub fn contacts_display_pictures(
    tui: &mut Tui,
    participants: &BTreeMap<Arc<String>, Contact>,
    last_participant: Option<Contact>,
) {
    if participants.len() < 2 {
        tui.style(taffy::Style {
            size: length(92.),
            ..Default::default()
        })
        .add_with_border(|tui| {
            tui.ui(|ui| {
                if let Some(participant) = participants.values().next() {
                    ui.add(if let Some(picture) = participant.display_picture.clone() {
                        egui::Image::from_bytes(format!("bytes://{}", picture.hash), picture.data)
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
                            .alt_text(format!("Display picture for {}", participant.display_name))
                    } else {
                        egui::Image::new(svg::default_display_picture())
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .alt_text(format!("Display picture for {}", participant.display_name))
                    })
                    .on_hover_text(format!("Display picture for {}", participant.display_name))
                } else if let Some(participant) = &last_participant {
                    ui.add(if let Some(picture) = participant.display_picture.clone() {
                        egui::Image::from_bytes(format!("bytes://{}", picture.hash), picture.data)
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
                            .alt_text(format!("Display picture for {}", participant.display_name))
                    } else {
                        egui::Image::new(svg::default_display_picture())
                            .fit_to_exact_size(egui::Vec2::splat(90.))
                            .alt_text(format!("Display picture for {}", participant.display_name))
                    })
                    .on_hover_text(format!("Display picture for {}", participant.display_name))
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
            display: taffy::Display::Grid,
            grid_template_columns: vec![length(45.), fr(1.)],
            grid_template_rows: vec![repeat("auto-fill", vec![length(43.)])],
            align_items: Some(taffy::AlignItems::Center),
            gap: taffy::Size {
                width: length(5.),
                height: length(10.),
            },
            ..Default::default()
        })
        .add(|tui| {
            for participant in participants.values() {
                tui.style(taffy::Style {
                    justify_self: Some(taffy::JustifySelf::Start),
                    ..Default::default()
                })
                .add_with_border(|tui| {
                    tui.ui(|ui| {
                        ui.add(if let Some(picture) = participant.display_picture.clone() {
                            egui::Image::from_bytes(
                                format!("bytes://{}", picture.hash),
                                picture.data,
                            )
                            .fit_to_exact_size(egui::Vec2::splat(44.))
                            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
                            .alt_text(format!("Display picture for {}", participant.display_name))
                        } else {
                            egui::Image::new(svg::default_display_picture())
                                .fit_to_exact_size(egui::Vec2::splat(44.))
                                .alt_text(format!(
                                    "Display picture for {}",
                                    participant.display_name
                                ))
                        })
                        .on_hover_text(format!("Display picture for {}", participant.display_name))
                    });
                });

                tui.style(taffy::Style {
                    grid_column: line(2),
                    ..Default::default()
                })
                .ui_add(egui::Label::new(participant.display_name.as_str()).truncate());
            }
        })
    }
}
