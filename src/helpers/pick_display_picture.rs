use crate::models::display_picture::DisplayPicture;
use crate::sqlite::Sqlite;
use anyhow::Context;
use image::imageops::FilterType;
use msnp11_sdk::Client;
use rfd::FileHandle;
use std::io::Cursor;
use std::sync::Arc;

pub async fn pick_display_picture(
    picture_future: impl Future<Output = Option<FileHandle>>,
    email: Arc<String>,
    client: Arc<Client>,
    sqlite: Sqlite,
) -> anyhow::Result<DisplayPicture> {
    let picture = picture_future
        .await
        .context("Could not find display picture")?;

    let mut bytes = Vec::new();
    let picture = image::open(picture.path())?;
    picture
        .resize_to_fill(200, 200, FilterType::Triangle)
        .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

    let hash = client.set_display_picture(bytes.clone()).await?;
    let _ = sqlite.insert_display_picture(&bytes, &hash);
    let _ = sqlite.update_user_display_picture(&email, &hash);

    Ok(DisplayPicture {
        data: Arc::from(bytes),
        hash: Arc::new(hash),
    })
}
