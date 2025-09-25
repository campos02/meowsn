use crate::sign_in;
use eframe::egui;

enum Screen {
    SignIn(sign_in::SignIn),
    Contacts,
}

pub struct MainWindow {
    screen: Screen,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            screen: Screen::SignIn(sign_in::SignIn::default()),
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match &mut self.screen {
            Screen::SignIn(sign_in) => sign_in.update(ctx, frame),
            Screen::Contacts => (),
        }
    }
}
