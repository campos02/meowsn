use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub server: String,
    pub nexus_url: String,
    pub check_for_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: "ms.msgrsvcs.ctsrv.xyz".to_string(),
            nexus_url: "https://pp.login.ugnet.xyz/rdr/pprdr.asp".to_string(),
            check_for_updates: true,
        }
    }
}

pub enum SettingsError {
    CreateSettingsDirectory,
    GetSettings,
    ParseSettings,
    WriteSettings,
}

pub fn get_settings() -> Result<Settings, SettingsError> {
    let mut settings_local =
        dirs::data_local_dir().ok_or(SettingsError::CreateSettingsDirectory)?;

    settings_local.push("icedm");
    std::fs::create_dir_all(&settings_local).or(Err(SettingsError::CreateSettingsDirectory))?;

    settings_local.push("icedm");
    settings_local.set_extension("toml");

    toml::from_str(&std::fs::read_to_string(settings_local).or(Err(SettingsError::GetSettings))?)
        .or(Err(SettingsError::ParseSettings))
}

pub fn save_settings(settings: &Settings) -> Result<(), SettingsError> {
    let mut settings_local =
        dirs::data_local_dir().ok_or(SettingsError::CreateSettingsDirectory)?;

    settings_local.push("icedm");
    std::fs::create_dir_all(&settings_local).or(Err(SettingsError::CreateSettingsDirectory))?;

    settings_local.push("icedm");
    settings_local.set_extension("toml");

    std::fs::write(
        settings_local,
        toml::to_string(&settings).or(Err(SettingsError::WriteSettings))?,
    )
    .or(Err(SettingsError::WriteSettings))
}
