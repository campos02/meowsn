#[derive(Clone, Default, PartialEq)]
pub enum ContactListStatus {
    #[default]
    Online,
    Busy,
    Away,
    AppearOffline,
    ChangeDisplayPicture,
    PersonalSettings,
    SignOut,
}

impl ContactListStatus {
    pub const ALL: [ContactListStatus; 7] = [
        ContactListStatus::Online,
        ContactListStatus::Busy,
        ContactListStatus::Away,
        ContactListStatus::AppearOffline,
        ContactListStatus::ChangeDisplayPicture,
        ContactListStatus::PersonalSettings,
        ContactListStatus::SignOut,
    ];
}

impl std::fmt::Display for ContactListStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Online => "(Online)",
            Self::Busy => "(Busy)",
            Self::Away => "(Away)",
            Self::AppearOffline => "(Appear Offline)",
            Self::ChangeDisplayPicture => "Change Display Picture",
            Self::PersonalSettings => "Personal Settings",
            Self::SignOut => "Sign Out",
        })
    }
}
