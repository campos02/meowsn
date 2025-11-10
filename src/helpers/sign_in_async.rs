use crate::models::sign_in_return::SignInReturn;
use crate::settings;
use crate::sqlite::Sqlite;
use msnp11_sdk::{Client, MsnpStatus, PersonalMessage, SdkError};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub enum SignInError {
    SdkError(SdkError),
    Cancelled,
}

pub async fn sign_in_async(
    email: Arc<String>,
    password: Arc<String>,
    status: MsnpStatus,
    sqlite: Sqlite,
    cancellation_token: CancellationToken,
) -> Result<SignInReturn, SignInError> {
    let settings = settings::get_settings().unwrap_or_default();
    let version = env!("CARGO_PKG_VERSION");
    let mut client = tokio::select! {
        client = Client::new(&settings.server, 1863) => {
            client.map_err(SignInError::SdkError)?
        }

        _ = cancellation_token.cancelled() => {
            return Err(SignInError::Cancelled);
        }
    };

    if let msnp11_sdk::Event::RedirectedTo { server, port } = tokio::select! {
        result = client.login(
            (*email).clone(),
            &password,
            &settings.nexus_url,
            "meowsn",
            version,
        ) => {
            match result {
                Ok(result) => result,
                Err(error) => {
                    let _ = client.disconnect().await;
                    return Err(SignInError::SdkError(error));
                }
            }
        }

        _ = cancellation_token.cancelled() => {
            let _ = client.disconnect().await;
            return Err(SignInError::Cancelled);
        }
    } {
        let _ = client.disconnect().await;
        client = tokio::select! {
            client = Client::new(&server, port) => {
                client.map_err(SignInError::SdkError)?
            }

            _ = cancellation_token.cancelled() => {
                return Err(SignInError::Cancelled);
            }
        };

        tokio::select! {
            result = client.login(
                (*email).clone(),
                &password,
                &settings.nexus_url,
                "meowsn",
                version,
            ) => {
                if let Err(error) = result {
                    let _ = client.disconnect().await;
                    return Err(SignInError::SdkError(error));
                }
            }

            _ = cancellation_token.cancelled() => {
                let _ = client.disconnect().await;
                return Err(SignInError::Cancelled);
            }
        }
    }

    let mut psm = None;
    let mut display_picture = None;

    if let Ok(user) = sqlite.select_user(&email) {
        psm = user.personal_message;
        display_picture = user.display_picture;

        if let Some(display_picture) = &display_picture {
            tokio::select! {
                result = client.set_display_picture(display_picture.data.to_vec()) => {
                    if let Err(error) = result {
                        let _ = client.disconnect().await;
                        return Err(SignInError::SdkError(error));
                    }
                }

                _ = cancellation_token.cancelled() => {
                    let _ = client.disconnect().await;
                    return Err(SignInError::Cancelled);
                }
            }
        }
    }

    tokio::select! {
        result = client.set_presence(status.clone()) => {
            if let Err(error) = result {
                let _ = client.disconnect().await;
                return Err(SignInError::SdkError(error));
            }
        }

        _ = cancellation_token.cancelled() => {
            let _ = client.disconnect().await;
            return Err(SignInError::Cancelled);
        }
    }

    let personal_message = PersonalMessage {
        psm: psm.unwrap_or_default(),
        current_media: "".to_string(),
    };

    tokio::select! {
        result = client.set_personal_message(&personal_message) => {
            if let Err(error) = result {
                let _ = client.disconnect().await;
                return Err(SignInError::SdkError(error));
            }
        }

        _ = cancellation_token.cancelled() => {
            let _ = client.disconnect().await;
            return Err(SignInError::Cancelled);
        }
    }

    Ok(SignInReturn {
        email,
        status,
        personal_message: personal_message.psm,
        display_picture,
        client: Arc::new(client),
    })
}
