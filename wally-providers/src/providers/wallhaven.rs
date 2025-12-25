use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::providers::WallpaperProvider;

const WALLHAVEN_API_URL: &str = "https://wallhaven.cc/api/v1/search";

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenData {
    path: Url,
}

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenResponse {
    data: Vec<WallhavenData>,
}

pub struct Wallhaven {}

impl WallpaperProvider for Wallhaven {
    async fn list(&self) -> Vec<Url> {
        todo!()
    }

    async fn random(&self) -> anyhow::Result<Url> {
        let response: WallhavenResponse = reqwest::get(WALLHAVEN_API_URL)
            .await?
            .json()
            .await
            .context("Unable to parse wallhaven response into json")?;

        let selected = rand::random_range(0..response.data.len());

        Ok(response
            .data
            .get(selected)
            .context("selected item does not exist")?
            .path
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_get_random_wallpaper() {
        let provider = Wallhaven {};
        let url = provider.random().await;
        assert!(url.is_ok());
        dbg!(url.unwrap().to_string());
    }
}
