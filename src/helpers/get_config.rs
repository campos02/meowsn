use crate::models::config::Config;
use crate::models::tab::Tab;
use msnp11_sdk::Client;
use std::sync::Arc;

pub async fn get_config(client: Arc<Client>, config_url: String) -> anyhow::Result<Config> {
    let config = client.get_config(&config_url).await?;
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

    Ok(Config {
        tabs,
        today_url: config.msn_today_url,
    })
}
