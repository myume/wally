use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::providers::WallpaperProvider;

const WALLHAVEN_API_URL: &str = "https://wallhaven.cc/api/v1/search";

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenData {
    id: String,
    dimension_x: u32,
    dimension_y: u32,
    resolution: String,
    file_size: u32,
    file_type: String,
    path: Url,
}

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenMeta {
    current_page: u32,
    last_page: u32,
    per_page: u32,
    total: u32,
    query: Option<String>,
    seed: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WallhavenResponse {
    data: Vec<WallhavenData>,
    meta: WallhavenMeta,
}

pub struct Wallhaven {}

impl WallpaperProvider for Wallhaven {
    async fn list(&self) -> Vec<Url> {
        todo!()
    }

    async fn random(&self) -> anyhow::Result<Url> {
        let mut handles = Vec::new();

        // fire off requests for 4 pages each of (usually) 24 wallpapers
        // for a larger selection of wallpapers to choose from for increased randomness.
        for page in 1..=4 {
            let handle: JoinHandle<anyhow::Result<WallhavenResponse>> = tokio::spawn(async move {
                reqwest::get(format!("{WALLHAVEN_API_URL}?page={page}"))
                    .await?
                    .json()
                    .await
                    .context("Unable to parse wallhaven response into json")
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            let response = handle.await.context("Failed to fire request")??;
            results.extend(response.data.into_iter().map(|data| data.path))
        }
        let selected = rand::random_range(0..results.len());

        Ok(results
            .get(selected)
            .context("selected item does not exist")?
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
        assert!(url.is_ok(), "{:?}", url);
    }
}
