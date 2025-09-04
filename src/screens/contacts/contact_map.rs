use crate::models::contact::Contact;
use crate::screens::contacts::contacts::Message;
use crate::screens::contacts::transparent_button::transparent_button;
use iced::border::radius;
use iced::font::Weight;
use iced::widget::{button, container, row, svg, text};
use iced::{Background, Border, Center, Color, Element, Fill, Font, Theme};
use iced_aw::ContextMenu;
use msnp11_sdk::{MsnpList, MsnpStatus};

pub fn contact_map(contact: &Contact) -> Element<'_, Message> {
    let personal_message_color = |theme: &Theme| text::Style {
        color: Some(theme.extended_palette().secondary.weak.color),
    };

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
                button(row![
                    if contact.new_messages {
                        text(&*contact.display_name).font(Font {
                            weight: Weight::Bold,
                            ..Font::default()
                        })
                    } else {
                        text(&*contact.display_name)
                    },
                    if let Some(personal_message) = &contact.personal_message
                        && !personal_message.is_empty()
                    {
                        row![
                            text(" - ").style(personal_message_color),
                            text(&**personal_message).style(personal_message_color)
                        ]
                    } else {
                        row![]
                    }
                ])
                .on_press(Message::Conversation(contact.clone()))
                .style(|theme: &Theme, status| match status {
                    button::Status::Hovered | button::Status::Pressed => {
                        button::secondary(theme, status)
                    }

                    button::Status::Active | button::Status::Disabled => {
                        button::secondary(theme, status).with_background(Color::TRANSPARENT)
                    }
                })
                .width(Fill)
            ]
            .align_y(Center)
        ]
        .align_y(Center)
        .spacing(10)
        .width(Fill),
        || {
            container(iced::widget::column![
                if !contact.lists.contains(&MsnpList::BlockList) {
                    transparent_button(text("Block").size(15))
                        .width(Fill)
                        .on_press(Message::BlockContact(contact.email.clone()))
                } else {
                    transparent_button(text("Unblock").size(15))
                        .width(Fill)
                        .on_press(Message::UnblockContact(contact.email.clone()))
                },
                transparent_button(text("Delete").size(15))
                    .width(Fill)
                    .on_press(Message::RemoveContact(contact.email.clone()))
            ])
            .style(|theme: &Theme| container::Style {
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: radius(2.0),
                },
                background: Some(Background::Color(
                    theme.extended_palette().secondary.base.color,
                )),
                ..container::Style::default()
            })
            .width(150)
            .into()
        },
    )
    .into()
}
