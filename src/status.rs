#[derive(Debug, Clone, Default, PartialEq)]
pub enum Status {
    #[default]
    Online,
    Busy,
    Away,
    AppearOffline,
    PersonalSettings,
}

impl Status {
    pub const ALL: [Status; 5] = [
        Status::Online,
        Status::Busy,
        Status::Away,
        Status::AppearOffline,
        Status::PersonalSettings,
    ];
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Online => "Online",
            Self::Busy => "Busy",
            Self::Away => "Away",
            Self::AppearOffline => "Appear Offline",
            Self::PersonalSettings => "Personal Settings",
        })
    }
}
