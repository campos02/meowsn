#[derive(Clone, Default, PartialEq)]
pub enum SignInStatus {
    #[default]
    Online,
    Busy,
    Away,
    AppearOffline,
    PersonalSettings,
}

impl SignInStatus {
    pub const ALL: [SignInStatus; 5] = [
        SignInStatus::Online,
        SignInStatus::Busy,
        SignInStatus::Away,
        SignInStatus::AppearOffline,
        SignInStatus::PersonalSettings,
    ];
}

impl std::fmt::Display for SignInStatus {
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
