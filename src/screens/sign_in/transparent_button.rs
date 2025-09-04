use crate::screens::sign_in::sign_in::Message;
use iced::widget::{Button, button};
use iced::{Color, Element, Renderer, Theme};

pub fn transparent_button<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message, Theme> {
    button(content).style(|theme: &Theme, status| match status {
        button::Status::Hovered | button::Status::Pressed => button::primary(theme, status),
        button::Status::Active | button::Status::Disabled => {
            button::secondary(theme, status).with_background(Color::TRANSPARENT)
        }
    })
}
