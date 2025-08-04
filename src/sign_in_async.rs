use crate::enums::sign_in_status::SignInStatus;
use crate::settings;
use crate::sqlite::Sqlite;
use msnp11_sdk::sdk_error::SdkError;
use msnp11_sdk::{Client, MsnpStatus, PersonalMessage};
use std::sync::Arc;

pub async fn sign_in_async(
    email: Arc<String>,
    password: Arc<String>,
    status: Option<SignInStatus>,
    sqlite: Sqlite,
) -> Result<(String, MsnpStatus, Arc<Client>), SdkError> {
    let settings = settings::get_settings().unwrap_or_default();
    let mut client = Client::new(&settings.server, 1863).await?;

    if let msnp11_sdk::Event::RedirectedTo { server, port } = client
        .login(
            (*email).clone(),
            &password,
            &settings.nexus_url,
            "icedm",
            "0.3.0",
        )
        .await?
    {
        client = Client::new(&server, port).await?;
        client
            .login(
                (*email).clone(),
                &password,
                &settings.nexus_url,
                "icedm",
                "0.3.0",
            )
            .await?;
    }

    let mut psm = None;
    if let Ok(user) = sqlite.select_user(&email) {
        psm = user.personal_message;
        if let Some(display_picture) = user.display_picture {
            client.set_display_picture(display_picture)?;
        }
    }

    let status = match status {
        Some(status) => match status {
            SignInStatus::Busy => MsnpStatus::Busy,
            SignInStatus::Away => MsnpStatus::Away,
            SignInStatus::AppearOffline => MsnpStatus::AppearOffline,
            _ => MsnpStatus::Online,
        },
        None => MsnpStatus::Online,
    };

    client.set_presence(status.clone()).await?;

    let personal_message = PersonalMessage {
        psm: psm.unwrap_or_default(),
        current_media: "".to_string(),
    };

    client.set_personal_message(&personal_message).await?;
    Ok((personal_message.psm, status, Arc::new(client)))
}
