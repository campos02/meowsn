use crate::models::contact::Contact;
use crate::screens::contacts::contacts::Message;
use crate::screens::contacts::transparent_button::transparent_button;
use iced::border::radius;
use iced::widget::{column, mouse_area};
use iced::widget::{container, row, rule, svg, text};
use iced::{Background, Border, Center, Color, Element, Fill, Theme};
use iced_aw::ContextMenu;
use msnp11_sdk::{MsnpList, MsnpStatus};
use std::sync::Arc;

pub fn contact_map(
    contact: &Contact,
    selected_contact: Option<Arc<String>>,
) -> Element<'_, Message> {
    ContextMenu::new(
        row![
            row![
                svg(if let Some(status) = &contact.status {
                    if contact.lists.contains(&MsnpList::BlockList) {
                        crate::svg::default_display_picture_blocked()
                    } else {
                        match status.status {
                            MsnpStatus::Busy | MsnpStatus::OnThePhone => {
                                crate::svg::default_display_picture_busy()
                            }

                            MsnpStatus::Away
                            | MsnpStatus::Idle
                            | MsnpStatus::BeRightBack
                            | MsnpStatus::OutToLunch => crate::svg::default_display_picture_away(),

                            _ => crate::svg::default_display_picture(),
                        }
                    }
                } else if contact.lists.contains(&MsnpList::BlockList) {
                    crate::svg::default_display_picture_offline_blocked()
                } else {
                    crate::svg::default_display_picture_offline()
                })
                .width(30),
                mouse_area(
                    container(row![
                        text(&*contact.display_name).size(15),
                        if let Some(personal_message) = &contact.personal_message
                            && !personal_message.is_empty()
                        {
                            let personal_message_color = |theme: &Theme| text::Style {
                                color: Some(theme.extended_palette().secondary.weak.color),
                            };

                            row![
                                text(" - ").style(personal_message_color).size(15),
                                text(&**personal_message)
                                    .style(personal_message_color)
                                    .size(15)
                            ]
                        } else {
                            row![]
                        }
                    ])
                    .padding(5)
                    .style(move |theme: &Theme| container::Style {
                        background: if selected_contact
                            .as_ref()
                            .is_some_and(|selected_contact| contact.email == *selected_contact)
                        {
                            Some(Background::from(
                                theme.extended_palette().secondary.strong.color,
                            ))
                        } else {
                            None
                        },
                        ..Default::default()
                    })
                )
                .on_press(Message::SelectContact(contact.email.clone()))
                .on_double_click(Message::Conversation(contact.clone()))
            ]
            .align_y(Center)
        ]
        .align_y(Center)
        .spacing(10)
        .width(Fill),
        || {
            container(column![
                transparent_button(text("Send an Instant Message").size(15))
                    .width(Fill)
                    .on_press(Message::Conversation(contact.clone())),
                rule::horizontal(10),
                if !contact.lists.contains(&MsnpList::BlockList) {
                    transparent_button(text("Block").size(15))
                        .width(Fill)
                        .on_press(Message::BlockContact(contact.email.clone()))
                } else {
                    transparent_button(text("Unblock").size(15))
                        .width(Fill)
                        .on_press(Message::UnblockContact(contact.email.clone()))
                },
                transparent_button(text("Delete Contact").size(15))
                    .width(Fill)
                    .on_press(Message::RemoveContact {
                        contact: contact.email.clone(),
                        guid: contact.guid.clone(),
                    })
            ])
            .style(|theme: &Theme| container::Style {
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: radius(2.0),
                },
                background: Some(Background::Color(
                    theme.extended_palette().secondary.strong.color,
                )),
                ..container::Style::default()
            })
            .width(160)
            .into()
        },
    )
    .into()
}
