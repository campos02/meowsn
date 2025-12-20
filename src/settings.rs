use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub server: String,
    pub nexus_url: String,
    pub check_for_updates: bool,
    pub notify_sign_ins: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: "ms.msgrsvcs.ctsrv.gay".to_string(),
            nexus_url: "https://pp.login.ugnet.gay/rdr/pprdr.asp".to_string(),
            check_for_updates: true,
            notify_sign_ins: true,
        }
    }
}

#[allow(dead_code)]
pub enum SettingsError {
    GetLocalDir,
    CreateSettingsDirectory(std::io::Error),
    GetSettings,
    SerializeSettings(toml::ser::Error),
    DeserializeSettings(toml::de::Error),
    WriteSettings(std::io::Error),
}

pub fn get_settings() -> Result<Settings, SettingsError> {
    // Compatibility with previous name
    let mut old_settings_local = dirs::data_local_dir().ok_or(SettingsError::GetLocalDir)?;
    old_settings_local.push("icedm");

    let mut settings_local = dirs::data_local_dir().ok_or(SettingsError::GetLocalDir)?;
    settings_local.push("meowsn");

    if old_settings_local.exists() {
        std::fs::rename(old_settings_local.clone(), settings_local.clone())
            .map_err(SettingsError::CreateSettingsDirectory)?;
    }

    std::fs::create_dir_all(&settings_local).map_err(SettingsError::CreateSettingsDirectory)?;

    let mut old_settings_local = settings_local.clone();
    old_settings_local.push("icedm");
    old_settings_local.set_extension("toml");

    settings_local.push("meowsn");
    settings_local.set_extension("toml");

    if old_settings_local.exists() {
        std::fs::rename(old_settings_local, settings_local.clone())
            .map_err(SettingsError::CreateSettingsDirectory)?;
    }

    toml::from_str(&std::fs::read_to_string(settings_local).or(Err(SettingsError::GetSettings))?)
        .map_err(SettingsError::DeserializeSettings)
}

pub fn save_settings(settings: &Settings) -> Result<(), SettingsError> {
    let mut settings_local = dirs::data_local_dir().ok_or(SettingsError::GetLocalDir)?;

    settings_local.push("meowsn");
    std::fs::create_dir_all(&settings_local).or(Err(SettingsError::GetLocalDir))?;

    settings_local.push("meowsn");
    settings_local.set_extension("toml");

    std::fs::write(
        settings_local,
        toml::to_string(&settings).map_err(SettingsError::SerializeSettings)?,
    )
    .map_err(SettingsError::WriteSettings)
}
