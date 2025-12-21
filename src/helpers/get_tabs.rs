use crate::models::tab::Tab;
use msnp11_sdk::Client;
use std::sync::Arc;

pub async fn get_tabs(
    client: Arc<Client>,
    config_url: &str,
) -> Result<Vec<Tab>, Box<dyn std::error::Error + Sync + Send>> {
    let config = client.get_config(config_url).await?;
    let client = reqwest::Client::new();

    let mut tabs = Vec::with_capacity(config.tabs.len());
    for tab in config.tabs {
        let response = client.get(&tab.image).send().await?;
        let image = response.bytes().await?;

        tabs.push(Tab {
            msn_tab: tab,
            image: Arc::from(&*image),
        });
    }

    Ok(tabs)
}
