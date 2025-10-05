use eframe::egui;
use eframe::egui::ImageSource;
use std::sync::LazyLock;

static DEFAULT_DISPLAY_PICTURE: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/default_display_picture.svg"));

static DEFAULT_DISPLAY_PICTURE_BUSY: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/default_display_picture_busy.svg"));

static DEFAULT_DISPLAY_PICTURE_AWAY: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/default_display_picture_away.svg"));

static DEFAULT_DISPLAY_PICTURE_BLOCKED: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/default_display_picture_blocked.svg"));

static DEFAULT_DISPLAY_PICTURE_OFFLINE: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/default_display_picture_offline.svg"));

static DEFAULT_DISPLAY_PICTURE_OFFLINE_BLOCKED: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/default_display_picture_offline_blocked.svg"));

static ADD_CONTACT: LazyLock<ImageSource> =
    LazyLock::new(|| egui::include_image!("../assets/add_contact.svg"));

pub fn default_display_picture() -> ImageSource<'static> {
    DEFAULT_DISPLAY_PICTURE.to_owned()
}

pub fn default_display_picture_busy() -> ImageSource<'static> {
    DEFAULT_DISPLAY_PICTURE_BUSY.to_owned()
}

pub fn default_display_picture_away() -> ImageSource<'static> {
    DEFAULT_DISPLAY_PICTURE_AWAY.to_owned()
}

pub fn default_display_picture_blocked() -> ImageSource<'static> {
    DEFAULT_DISPLAY_PICTURE_BLOCKED.to_owned()
}

pub fn default_display_picture_offline() -> ImageSource<'static> {
    DEFAULT_DISPLAY_PICTURE_OFFLINE.to_owned()
}

pub fn default_display_picture_offline_blocked() -> ImageSource<'static> {
    DEFAULT_DISPLAY_PICTURE_OFFLINE_BLOCKED.to_owned()
}

pub fn add_contact() -> ImageSource<'static> {
    ADD_CONTACT.to_owned()
}
