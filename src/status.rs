#[derive(Debug, Clone, Default, PartialEq)]
pub enum Status {
    #[default]
    Online,
    Busy,
    Away,
    AppearOffline,
}

impl Status {
    pub const ALL: [Status; 4] = [
        Status::Online,
        Status::Busy,
        Status::Away,
        Status::AppearOffline,
    ];
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Online => "Online",
            Self::Busy => "Busy",
            Self::Away => "Away",
            Self::AppearOffline => "Appear Offline",
        })
    }
}
