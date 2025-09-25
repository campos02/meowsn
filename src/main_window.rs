use crate::screens::{contacts, sign_in};
use eframe::egui;

enum Screen {
    SignIn(sign_in::SignIn),
    Contacts(contacts::Contacts),
}

pub enum Message {
    SignIn,
    SignOut,
}

pub struct MainWindow {
    screen: Screen,
    sender: std::sync::mpsc::Sender<Message>,
    receiver: std::sync::mpsc::Receiver<Message>,
}

impl Default for MainWindow {
    fn default() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            screen: Screen::SignIn(sign_in::SignIn::new(sender.clone())),
            sender,
            receiver,
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                Message::SignIn => {
                    self.screen = Screen::Contacts(contacts::Contacts::new(self.sender.clone()))
                }

                Message::SignOut => {
                    self.screen = Screen::SignIn(sign_in::SignIn::new(self.sender.clone()))
                }
            }
        }

        match &mut self.screen {
            Screen::SignIn(sign_in) => sign_in.update(ctx, frame),
            Screen::Contacts(contacts) => contacts.update(ctx, frame),
        }
    }
}
