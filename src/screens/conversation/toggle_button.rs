use crate::screens::conversation::conversation::Message;
use iced::widget::{Button, button};
use iced::{Color, Element, Renderer, Theme};

pub fn toggle_button<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    state: bool,
) -> Button<'a, Message, Theme> {
    button(content).style(move |theme: &Theme, status| {
        if state {
            button::primary(theme, status)
        } else {
            match status {
                button::Status::Hovered | button::Status::Pressed => button::primary(theme, status),
                button::Status::Active | button::Status::Disabled => {
                    button::background(theme, status).with_background(Color::TRANSPARENT)
                }
            }
        }
    })
}
