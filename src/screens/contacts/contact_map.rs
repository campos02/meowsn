use crate::models::contact::Contact;
use crate::screens::contacts::contacts::Message;
use crate::screens::contacts::transparent_button::transparent_button;
use crate::widgets::simpler_drop_down::SimplerDropDown;
use iced::border::radius;
use iced::widget::rule::FillMode;
use iced::widget::{column, mouse_area};
use iced::widget::{container, row, rule, svg, text};
use iced::{Background, Border, Center, Color, Element, Fill, Shrink, Theme};
use msnp11_sdk::{MsnpList, MsnpStatus};
use std::sync::Arc;

pub fn contact_map(
    contact: &Contact,
    selected_contact: Option<Arc<String>>,
    contact_menu_opened: bool,
) -> Element<'_, Message> {
    SimplerDropDown::new(
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
                            color: Some(theme.extended_palette().secondary.strong.color),
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
                            theme.extended_palette().background.neutral.color,
                        ))
                    } else {
                        None
                    },
                    ..Default::default()
                })
            )
            .on_press(Message::SelectContact(contact.email.clone()))
            .on_right_press(Message::OpenContactMenu(contact.email.clone()))
            .on_double_click(Message::Conversation(contact.clone()))
        ]
        .align_y(Center)
        .spacing(10)
        .width(Shrink),
        container(column![
            transparent_button(text("Send an Instant Message"))
                .width(Fill)
                .on_press(Message::Conversation(contact.clone())),
            rule::horizontal(1).style(|theme: &Theme| rule::Style {
                color: theme.extended_palette().secondary.weak.color,
                radius: Default::default(),
                fill_mode: FillMode::Padded(3),
                snap: Default::default(),
            }),
            if !contact.lists.contains(&MsnpList::BlockList) {
                transparent_button(text("Block"))
                    .width(Fill)
                    .on_press(Message::BlockContact(contact.email.clone()))
            } else {
                transparent_button(text("Unblock"))
                    .width(Fill)
                    .on_press(Message::UnblockContact(contact.email.clone()))
            },
            transparent_button(text("Delete Contact"))
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
                theme.extended_palette().background.strong.color,
            )),
            ..container::Style::default()
        })
        .width(200),
        contact_menu_opened,
    )
    .on_dismiss(Message::CloseContactMenu)
    .into()
}
