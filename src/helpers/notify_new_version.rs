use crate::settings;
use notify_rust::Notification;
use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub async fn notify_new_version() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = settings::get_settings().unwrap_or_default();
    if !settings.check_for_updates {
        return Ok(());
    }

    let cargo_version = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::builder().user_agent("meowsn").build()?;

    let response = client
        .get("https://api.github.com/repos/campos02/meowsn/releases/latest")
        .send()
        .await?
        .json::<Release>()
        .await?;

    if Version::parse(&response.tag_name.replace("v", ""))? > Version::parse(cargo_version)? {
        Notification::new()
            .summary("New release")
            .body("A new version of meowsn is available at GitHub!")
            .show()?;
    }

    Ok(())
}
