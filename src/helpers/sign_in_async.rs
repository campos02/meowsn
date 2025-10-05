use crate::models::sign_in_return::SignInReturn;
use crate::settings;
use crate::sqlite::Sqlite;
use msnp11_sdk::{Client, MsnpStatus, PersonalMessage, SdkError};
use std::sync::Arc;

pub async fn sign_in_async(
    email: Arc<String>,
    password: Arc<String>,
    status: MsnpStatus,
    sqlite: Sqlite,
) -> Result<SignInReturn, SdkError> {
    let settings = settings::get_settings().unwrap_or_default();
    let version = env!("CARGO_PKG_VERSION");
    let mut client = Client::new(&settings.server, 1863).await?;

    if let msnp11_sdk::Event::RedirectedTo { server, port } = client
        .login(
            (*email).clone(),
            &password,
            &settings.nexus_url,
            "meowsn",
            version,
        )
        .await?
    {
        client = Client::new(&server, port).await?;
        client
            .login(
                (*email).clone(),
                &password,
                &settings.nexus_url,
                "meowsn",
                version,
            )
            .await?;
    }

    let mut psm = None;
    let mut display_picture = None;

    if let Ok(user) = sqlite.select_user(&email) {
        psm = user.personal_message;
        display_picture = user.display_picture;

        if let Some(display_picture) = &display_picture {
            client
                .set_display_picture(display_picture.data.to_vec())
                .await?;
        }
    }

    client.set_presence(status.clone()).await?;
    let personal_message = PersonalMessage {
        psm: psm.unwrap_or_default(),
        current_media: "".to_string(),
    };

    client.set_personal_message(&personal_message).await?;

    Ok(SignInReturn {
        email,
        status,
        personal_message: personal_message.psm,
        display_picture,
        client: Arc::new(client),
    })
}
