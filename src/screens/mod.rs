use regex::Regex;
use std::sync::LazyLock;

mod add_contact;
pub mod contacts;
pub mod conversation;
mod invite;
pub mod personal_settings;
pub mod sign_in;

pub static PLUS_TAGS_REGEX: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\[/?[abcius]=.*?]|\[/?[abcius]]").ok());

pub static URL_REGEX: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{2,256}\.[a-z]{2,4}\b([-a-zA-Z0-9@:%_+.~#?&/=]*)",
    )
    .ok()
});
