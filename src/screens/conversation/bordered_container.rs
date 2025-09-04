use crate::screens::conversation::conversation::Message;
use iced::border::radius;
use iced::widget::container;
use iced::{Border, Element, Renderer, Theme};

pub fn bordered_container<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    radius_len: f32,
) -> container::Container<'a, Message> {
    container(content)
        .style(move |theme: &Theme| container::Style {
            border: Border {
                color: theme.palette().text,
                width: 1.0,
                radius: radius(radius_len),
            },
            ..container::Style::default()
        })
        .padding(3)
}
