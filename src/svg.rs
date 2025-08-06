use iced::widget::svg::Handle;
use std::sync::LazyLock;

static DEFAULT_DISPLAY_PICTURE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/default_display_picture.svg")));

static DEFAULT_DISPLAY_PICTURE_BUSY: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../assets/default_display_picture_busy.svg"))
});

static DEFAULT_DISPLAY_PICTURE_AWAY: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../assets/default_display_picture_away.svg"))
});

static DEFAULT_DISPLAY_PICTURE_BLOCKED: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!(
        "../assets/default_display_picture_blocked.svg"
    ))
});

static DEFAULT_DISPLAY_PICTURE_OFFLINE: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!(
        "../assets/default_display_picture_offline.svg"
    ))
});

static DEFAULT_DISPLAY_PICTURE_OFFLINE_BLOCKED: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!(
        "../assets/default_display_picture_offline_blocked.svg"
    ))
});

static ADD_CONTACT: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../assets/add_contact.svg")));

pub fn default_display_picture() -> Handle {
    DEFAULT_DISPLAY_PICTURE.clone()
}

pub fn default_display_picture_busy() -> Handle {
    DEFAULT_DISPLAY_PICTURE_BUSY.clone()
}

pub fn default_display_picture_away() -> Handle {
    DEFAULT_DISPLAY_PICTURE_AWAY.clone()
}

pub fn default_display_picture_blocked() -> Handle {
    DEFAULT_DISPLAY_PICTURE_BLOCKED.clone()
}

pub fn default_display_picture_offline() -> Handle {
    DEFAULT_DISPLAY_PICTURE_OFFLINE.clone()
}

pub fn default_display_picture_offline_blocked() -> Handle {
    DEFAULT_DISPLAY_PICTURE_OFFLINE_BLOCKED.clone()
}

pub fn add_contact() -> Handle {
    ADD_CONTACT.clone()
}
