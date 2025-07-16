use crate::Message;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};

pub fn listen(key: Key, modifiers: Modifiers) -> Option<Message> {
    if let Key::Named(Named::Tab) = key {
        return if modifiers.shift() {
            Some(Message::FocusPrevious)
        } else {
            Some(Message::FocusNext)
        };
    }

    None
}
