use crate::screens::sign_in::sign_in::Message;
use iced::border::radius;
use iced::widget::container;
use iced::{Border, Element, Renderer, Theme};

pub fn bordered_container<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> container::Container<'a, Message> {
    container(content)
        .width(110)
        .style(|theme: &Theme| container::Style {
            border: Border {
                color: theme.palette().text,
                width: 1.0,
                radius: radius(10.0),
            },
            ..container::Style::default()
        })
        .padding(3)
}
