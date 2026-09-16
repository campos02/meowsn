use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub server: String,
    pub nexus_url: String,
    pub config_server: String,
    pub check_for_updates: bool,
    pub notify_sign_ins: bool,
    pub notify_added_by: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: "ms.msgrsvcs.ctsrv.gay".to_string(),
            nexus_url: "https://pp.login.ugnet.gay/rdr/pprdr.asp".to_string(),
            config_server: "https://conf.msgrsvcs.ctsrv.gay/Config/MsgrConfig.asmx".to_string(),
            check_for_updates: true,
            notify_sign_ins: true,
            notify_added_by: true,
        }
    }
}

pub fn get_settings() -> anyhow::Result<Settings> {
    let mut settings_local =
        dirs::data_local_dir().context("Could not find local data directory")?;

    settings_local.push("meowsn");
    std::fs::create_dir_all(&settings_local)?;

    settings_local.push("meowsn");
    settings_local.set_extension("toml");

    toml::from_str(&std::fs::read_to_string(settings_local)?).context("Could not read settings")
}

pub fn save_settings(settings: &Settings) -> anyhow::Result<()> {
    let mut settings_local =
        dirs::data_local_dir().context("Could not find local data directory")?;

    settings_local.push("meowsn");
    std::fs::create_dir_all(&settings_local)?;

    settings_local.push("meowsn");
    settings_local.set_extension("toml");

    std::fs::write(settings_local, toml::to_string(&settings)?).context("Could not save settings")
}
